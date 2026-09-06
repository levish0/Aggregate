use super::worker::{SimulationWorker, WorkOutcome, WorkRequest};
use aggregate_programs::{InspectionScope, InspectionSection};
use aggregate_scenario::parse_scenario;
use aggregate_simulation_core::{
    DayReport, Simulation, SimulationCommand, SimulationError, SimulationEvent,
};
use aggregate_world::{
    ContentDefinitions, CountryId, FacilityDefinitionId, ProvinceId, WorldSnapshot,
    WorldSnapshotIndex,
};
use bevy::prelude::*;

pub struct NewsEntry {
    pub day: u64,
    pub event: SimulationEvent,
}
pub enum SessionFeedback {
    Ready,
    ConstructionStarted(FacilityDefinitionId),
    Error(SimulationError),
}

/// UI keeps committed read models only. The worker owns the authoritative simulation.
#[derive(Resource)]
pub struct ManagementSession {
    worker: SimulationWorker,
    pending_jobs: usize,
    inspection: Option<(InspectionScope, u64, Result<Vec<InspectionSection>, String>)>,
    pub inspection_revision: u64,
    pub snapshot: WorldSnapshot,
    pub index: WorldSnapshotIndex,
    pub definitions: ContentDefinitions,
    pub player_country: CountryId,
    pub last_report: Option<DayReport>,
    pub news: Vec<NewsEntry>,
    pub feedback: SessionFeedback,
    pub running: bool,
    pub speed: super::SimulationSpeed,
    pub session_id: uuid::Uuid,
    pub geographic: bool,
    pub revision: u64,
    pub province_page: usize,
}

#[derive(Resource)]
pub struct ManagementViewState {
    pub selected_province: ProvinceId,
}

impl ManagementSession {
    pub fn is_busy(&self) -> bool {
        self.pending_jobs != 0
    }

    pub fn inspect_programs(
        &mut self,
        scope: &InspectionScope,
    ) -> Result<Option<Vec<InspectionSection>>, SimulationError> {
        if let Some((cached_scope, revision, result)) = &self.inspection
            && cached_scope == scope
            && *revision == self.revision
        {
            return result
                .clone()
                .map(Some)
                .map_err(SimulationError::InvalidPrograms);
        }
        if !self.is_busy() {
            self.worker
                .submit(WorkRequest::Inspect(scope.clone(), self.revision))?;
            self.pending_jobs += 1;
        }
        Ok(None)
    }

    pub fn foundation() -> Self {
        let scenario = parse_scenario(include_str!("../../../../scenarios/foundation.json"))
            .expect("validated bundled scenario");
        let country = scenario.initial_state.countries[0].id.clone();
        Self::create(
            scenario,
            country,
            vec![std::sync::Arc::new(aggregate_economy::EconomyProgram)],
            false,
        )
        .expect("bundled world")
    }

    pub fn geographic(
        scenario: aggregate_world::Scenario,
        player_country: CountryId,
        programs: Vec<std::sync::Arc<dyn aggregate_programs::SimulationProgram>>,
    ) -> Result<Self, SimulationError> {
        Self::create(scenario, player_country, programs, true)
    }

    fn create(
        scenario: aggregate_world::Scenario,
        player_country: CountryId,
        programs: Vec<std::sync::Arc<dyn aggregate_programs::SimulationProgram>>,
        geographic: bool,
    ) -> Result<Self, SimulationError> {
        if !scenario
            .initial_state
            .provinces
            .iter()
            .any(|province| province.country == player_country)
        {
            return Err(SimulationError::CommandRejected(
                "selected country has no playable land".into(),
            ));
        }
        let definitions = scenario.definitions.clone();
        let mut simulation = Simulation::from_scenario_with_programs(scenario, programs)?;
        let snapshot = simulation.snapshot();
        let index = WorldSnapshotIndex::build(&snapshot);
        let worker = SimulationWorker::spawn(simulation)?;
        Ok(Self {
            worker,
            pending_jobs: 0,
            inspection: None,
            inspection_revision: 0,
            snapshot,
            index,
            definitions,
            player_country,
            last_report: None,
            news: Vec::new(),
            feedback: SessionFeedback::Ready,
            running: false,
            speed: super::SimulationSpeed::default(),
            session_id: uuid::Uuid::now_v7(),
            geographic,
            revision: 0,
            province_page: 0,
        })
    }

    pub fn initial_view(&self) -> ManagementViewState {
        ManagementViewState {
            selected_province: self.snapshot.provinces
                [self.index.countries[&self.player_country].province_indices[0]]
                .id
                .clone(),
        }
    }

    pub fn start_construction(&mut self, province: &ProvinceId, definition: &FacilityDefinitionId) {
        let Some(recipe) = self
            .definitions
            .facilities
            .iter()
            .find(|item| item.id == *definition)
        else {
            self.feedback = SessionFeedback::Error(SimulationError::CommandRejected(format!(
                "unknown facility definition {definition}"
            )));
            self.revision += 1;
            return;
        };
        let priority = self
            .snapshot
            .facilities
            .iter()
            .filter(|facility| facility.definition == *definition)
            .map(|facility| facility.production_priority)
            .min()
            .unwrap_or(100);
        let command = SimulationCommand::StartConstruction {
            country: self.player_country.clone(),
            province: province.clone(),
            facility: uuid::Uuid::now_v7().into(),
            definition: definition.clone(),
            workers: recipe.construction.max_workers,
            production_priority: priority,
        };
        self.submit(WorkRequest::Construction(command, definition.clone()));
    }

    pub fn step(&mut self) {
        if !self.is_busy() {
            self.submit(WorkRequest::Step);
        }
    }

    fn submit(&mut self, request: WorkRequest) {
        let result = if self.pending_jobs >= 32 {
            Err(SimulationError::CommandRejected(
                "simulation request queue is full".into(),
            ))
        } else {
            self.worker.submit(request)
        };
        match result {
            Ok(()) => self.pending_jobs += 1,
            Err(error) => {
                self.running = false;
                self.feedback = SessionFeedback::Error(error);
                self.revision += 1;
            }
        }
    }

    fn accept(&mut self, completed: super::worker::CompletedWork) {
        self.pending_jobs = self.pending_jobs.saturating_sub(1);
        if let Some((snapshot, index)) = completed.snapshot {
            let retired = (
                std::mem::replace(&mut self.snapshot, snapshot),
                std::mem::replace(&mut self.index, index),
            );
            bevy::tasks::AsyncComputeTaskPool::get()
                .spawn(async move {
                    drop(retired);
                })
                .detach();
        }
        match completed.result {
            Ok(WorkOutcome::Inspection(scope, revision, result)) => {
                self.inspection = Some((scope, revision, result));
                self.inspection_revision += 1;
                return;
            }
            Ok(WorkOutcome::Day(report)) => {
                self.news.extend(
                    report
                        .events
                        .iter()
                        .filter(|event| {
                            let (SimulationEvent::ConstructionStarted { province, .. }
                            | SimulationEvent::ConstructionCompleted { province, .. }
                            | SimulationEvent::FoodShortfall { province, .. }) = event;
                            self.index.provinces.get(province).is_some_and(|position| {
                                self.snapshot.provinces[*position].country == self.player_country
                            })
                        })
                        .cloned()
                        .map(|event| NewsEntry {
                            day: report.day,
                            event,
                        }),
                );
                if let Some(retired) = self.last_report.replace(report) {
                    bevy::tasks::AsyncComputeTaskPool::get()
                        .spawn(async move {
                            drop(retired);
                        })
                        .detach();
                }
                if matches!(self.feedback, SessionFeedback::Error(_)) {
                    self.feedback = SessionFeedback::Ready;
                }
            }
            Ok(WorkOutcome::Construction(definition, outcome)) => {
                self.news.push(NewsEntry {
                    day: self.snapshot.day,
                    event: outcome.event,
                });
                self.feedback = SessionFeedback::ConstructionStarted(definition);
            }
            Err(error) => {
                self.running = false;
                self.feedback = SessionFeedback::Error(error);
            }
        }
        if self.news.len() > 200 {
            self.news.drain(..self.news.len() - 200);
        }
        self.revision += 1;
    }
}

pub fn poll_simulation(mut session: ResMut<ManagementSession>) {
    // No mutable access on idle frames: Bevy change detection must remain quiet.
    if !session.is_busy() {
        return;
    }
    if let Some(completed) = session.worker.try_receive() {
        let started = std::time::Instant::now();
        session.accept(completed);
        tracing::debug!(target: "aggregate_client::simulation_performance", apply_ms = started.elapsed().as_secs_f64() * 1000., "Committed UI snapshot published");
    }
}
