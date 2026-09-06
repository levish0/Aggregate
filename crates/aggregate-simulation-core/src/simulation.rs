use crate::{
    SimulationClock,
    command::{RecordedCommand, SimulationCommand},
    error::SimulationError,
    report::{CommandOutcome, DayReport},
    schedule::create_schedule,
    world_storage::{self, DayWork},
};
use aggregate_modules::{ModuleRuntime, SavedModuleState, SimulationModule};
use aggregate_scenario::validate_scenario;
use aggregate_world::{Scenario, WorldSnapshot};
use bevy_ecs::prelude::*;
use std::sync::Arc;

/// Owns one authoritative ECS World. All public commands run between complete days.
/// No domain calculation depends on a renderer, wall-clock time, or ECS entity IDs.
pub struct Simulation {
    pub(crate) world: World,
    schedule: Schedule,
    pub(crate) initial_scenario: Scenario,
    pub(crate) commands: Vec<RecordedCommand>,
    pub(crate) modules: ModuleRuntime,
}

impl Simulation {
    pub fn from_scenario_with_modules(
        scenario: Scenario,
        modules: Vec<Arc<dyn SimulationModule>>,
    ) -> Result<Self, SimulationError> {
        let mut simulation = Self::from_scenario(scenario)?;
        simulation.modules =
            ModuleRuntime::initialize(modules, &simulation.initial_scenario.initial_state)
                .map_err(SimulationError::InvalidModules)?;
        Ok(simulation)
    }

    pub fn module_states(&self) -> Vec<SavedModuleState> {
        self.modules.snapshot()
    }

    pub fn from_scenario(mut scenario: Scenario) -> Result<Self, SimulationError> {
        validate_scenario(&scenario)?;
        scenario.initial_state.normalize();
        tracing::info!(preset = %scenario.id, countries = scenario.initial_state.countries.len(), provinces = scenario.initial_state.provinces.len(), "World initialized");
        Ok(Self::from_validated_state(
            scenario.clone(),
            &scenario.initial_state,
            Vec::new(),
        ))
    }

    pub(crate) fn from_validated_state(
        scenario: Scenario,
        state: &WorldSnapshot,
        commands: Vec<RecordedCommand>,
    ) -> Self {
        Self {
            world: world_storage::create_world(&scenario, state),
            schedule: create_schedule(),
            initial_scenario: scenario,
            commands,
            modules: ModuleRuntime::default(),
        }
    }

    pub fn clock(&self) -> SimulationClock {
        *self.world.resource::<SimulationClock>()
    }

    pub fn snapshot(&mut self) -> WorldSnapshot {
        world_storage::snapshot(&mut self.world)
    }

    pub fn command_log(&self) -> &[RecordedCommand] {
        &self.commands
    }

    #[tracing::instrument(level = "debug", skip_all, fields(day = self.clock().day()))]
    pub fn execute(
        &mut self,
        command: SimulationCommand,
    ) -> Result<CommandOutcome, SimulationError> {
        let sequence = u64::try_from(self.commands.len())
            .ok()
            .and_then(|length| length.checked_add(1))
            .ok_or_else(|| SimulationError::CommandRejected("command sequence exhausted".into()))?;
        let outcome = crate::command::execute_command(
            &mut self.world,
            &command,
            sequence,
            &self.initial_scenario,
        )
        .inspect_err(|error| {
            tracing::warn!(?command, reason = %error, "Simulation command rejected");
        })?;
        tracing::info!(sequence, ?command, "Simulation command accepted");
        self.commands.push(RecordedCommand {
            day: self.clock().day(),
            sequence,
            command,
        });
        Ok(outcome)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(day = self.clock().day().saturating_add(1)))]
    pub fn step(&mut self) -> Result<DayReport, SimulationError> {
        let next_day = self.clock().day().saturating_add(1);
        let prepared_modules = if self.modules.is_empty() {
            None
        } else {
            let state = self.snapshot();
            let prepared = self.modules.prepare_day(&state, next_day).map_err(|reason| {
                tracing::error!(day = next_day, %reason, "Module calculation failed; state not committed");
                SimulationError::DayFailed { day: next_day, phase: "modules", reason }
            })?;
            self.world
                .resource_mut::<world_storage::WorkforceLimits>()
                .0
                .clone_from(&prepared.workforce_limits);
            Some(prepared)
        };
        self.schedule.run(&mut self.world);
        let mut work = self.world.resource_mut::<DayWork>();
        if let Some((phase, reason)) = work.failure.take() {
            tracing::error!(day = next_day, phase, %reason, "Daily calculation failed; state not committed");
            return Err(SimulationError::DayFailed {
                day: next_day,
                phase,
                reason,
            });
        }
        let report = work
            .report
            .take()
            .expect("successful commit creates a day report");
        tracing::debug!(
            day = report.day,
            events = report.events.len(),
            "Daily state committed"
        );
        if let Some(prepared) = prepared_modules {
            self.modules.commit(prepared);
        }
        Ok(report)
    }

    /// Replay uses the same validated command path. Log order is semantic, never ECS order.
    pub fn replay(
        scenario: Scenario,
        commands: &[RecordedCommand],
        through_day: u64,
    ) -> Result<Self, SimulationError> {
        Self::replay_with_modules(scenario, commands, through_day, Vec::new())
    }

    pub fn replay_with_modules(
        scenario: Scenario,
        commands: &[RecordedCommand],
        through_day: u64,
        modules: Vec<Arc<dyn SimulationModule>>,
    ) -> Result<Self, SimulationError> {
        crate::save::validate_command_log(commands, through_day)?;
        let mut simulation = Self::from_scenario_with_modules(scenario, modules)?;
        for record in commands {
            while simulation.clock().day() < record.day {
                simulation.step()?;
            }
            let outcome = simulation.execute(record.command.clone())?;
            if outcome.sequence != record.sequence {
                return Err(SimulationError::InvalidReplay(
                    "command sequence mismatch".into(),
                ));
            }
        }
        while simulation.clock().day() < through_day {
            simulation.step()?;
        }
        Ok(simulation)
    }
}
