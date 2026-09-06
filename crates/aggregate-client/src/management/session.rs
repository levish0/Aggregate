use aggregate_scenario::parse_scenario;
use aggregate_simulation_core::{
    DayReport, Simulation, SimulationCommand, SimulationError, SimulationEvent,
};
use aggregate_world::{
    ContentDefinitions, CountryId, FacilityDefinitionId, ProvinceId, WorldSnapshot,
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

/// Owns the simulation. Snapshot/report fields are refreshed only after core operations;
/// presentation never writes them back into the authoritative World.
#[derive(Resource)]
pub struct ManagementSession {
    simulation: Simulation,
    pub snapshot: WorldSnapshot,
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
}

#[derive(Resource)]
pub struct ManagementViewState {
    pub selected_province: ProvinceId,
}

impl ManagementSession {
    pub fn inspect_programs(
        &mut self,
        scope: &aggregate_programs::InspectionScope,
    ) -> Result<Vec<aggregate_programs::InspectionSection>, SimulationError> {
        self.simulation.inspect_programs(scope)
    }

    pub fn foundation() -> Self {
        let scenario = parse_scenario(include_str!("../../../../scenarios/foundation.json"))
            .expect("validated bundled scenario");
        let player_country = scenario.initial_state.countries[0].id.clone();
        let definitions = scenario.definitions.clone();
        let mut simulation = Simulation::from_scenario_with_programs(scenario, vec![std::sync::Arc::new(aggregate_economy::EconomyProgram)]).expect("validated scenario");
        let snapshot = simulation.snapshot();
        Self {
            simulation,
            snapshot,
            definitions,
            player_country,
            last_report: None,
            news: Vec::new(),
            feedback: SessionFeedback::Ready,
            running: false,
            speed: super::SimulationSpeed::default(),
            session_id: uuid::Uuid::now_v7(),
            geographic: false,
            revision: 0,
        }
    }

    pub fn geographic(scenario: aggregate_world::Scenario, player_country: CountryId, programs: Vec<std::sync::Arc<dyn aggregate_programs::SimulationProgram>>) -> Result<Self, SimulationError> {
        if !scenario.initial_state.provinces.iter().any(|province| province.country == player_country) {
            return Err(SimulationError::CommandRejected("selected country has no playable land".into()));
        }
        let definitions = scenario.definitions.clone();
        let mut simulation = Simulation::from_scenario_with_programs(scenario, programs)?;
        let snapshot = simulation.snapshot();
        Ok(Self { simulation, snapshot, definitions, player_country, last_report: None, news: Vec::new(), feedback: SessionFeedback::Ready, running: false, speed: super::SimulationSpeed::default(), session_id: uuid::Uuid::now_v7(), geographic: true, revision: 0 })
    }


    pub fn initial_view(&self) -> ManagementViewState {
        ManagementViewState {
            selected_province: self
                .snapshot
                .provinces
                .iter()
                .find(|province| province.country == self.player_country)
                .expect("bundled country has a province")
                .id
                .clone(),
        }
    }

    pub fn start_construction(&mut self, province: &ProvinceId, definition: &FacilityDefinitionId) {
        self.revision += 1;
        let Some(recipe) = self
            .definitions
            .facilities
            .iter()
            .find(|item| item.id == *definition)
        else {
            self.feedback = SessionFeedback::Error(SimulationError::CommandRejected(format!(
                "unknown facility definition {definition}"
            )));
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
        match self.simulation.execute(command) {
            Ok(outcome) => {
                self.snapshot = self.simulation.snapshot();
                self.news.push(NewsEntry {
                    day: self.snapshot.day,
                    event: outcome.event,
                });
                self.feedback = SessionFeedback::ConstructionStarted(definition.clone());
            }
            Err(error) => self.feedback = SessionFeedback::Error(error),
        }
    }

    pub fn step(&mut self) {
        self.revision += 1;
        match self.simulation.step() {
            Ok(report) => {
                self.snapshot = self.simulation.snapshot();
                let owned: std::collections::BTreeSet<_> = self.snapshot.provinces.iter().filter(|province|province.country == self.player_country).map(|province|&province.id).collect();
                self.news
                    .extend(report.events.iter().filter(|event| {
                        let (SimulationEvent::ConstructionStarted { province, .. } | SimulationEvent::ConstructionCompleted { province, .. } | SimulationEvent::FoodShortfall { province, .. }) = event;
                        owned.contains(province)
                    }).cloned().map(|event| NewsEntry {
                        day: report.day,
                        event,
                    }));
                if self.news.len() > 200 { self.news.drain(..self.news.len()-200); }
                self.last_report = Some(report);
                if matches!(self.feedback, SessionFeedback::Error(_)) {
                    self.feedback = SessionFeedback::Ready;
                }
            }
            Err(error) => {
                self.running = false;
                self.feedback = SessionFeedback::Error(error);
            }
        }
    }
}
