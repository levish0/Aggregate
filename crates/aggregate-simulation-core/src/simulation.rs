use crate::{
    SimulationClock,
    command::{RecordedCommand, SimulationCommand},
    error::SimulationError,
    report::{CommandOutcome, DayReport},
    schedule::create_schedule,
    world_storage::{self, DayWork},
};
use aggregate_scenario::validate_scenario;
use aggregate_world::{Scenario, WorldSnapshot};
use bevy_ecs::prelude::*;

/// Owns one authoritative ECS World. All public commands run between complete days.
/// No domain calculation depends on a renderer, wall-clock time, or ECS entity IDs.
pub struct Simulation {
    pub(crate) world: World,
    schedule: Schedule,
    pub(crate) initial_scenario: Scenario,
    pub(crate) commands: Vec<RecordedCommand>,
}

impl Simulation {
    pub fn from_scenario(mut scenario: Scenario) -> Result<Self, SimulationError> {
        validate_scenario(&scenario)?;
        scenario.initial_state.normalize();
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
        )?;
        self.commands.push(RecordedCommand {
            day: self.clock().day(),
            sequence,
            command,
        });
        Ok(outcome)
    }

    pub fn step(&mut self) -> Result<DayReport, SimulationError> {
        let next_day = self.clock().day().saturating_add(1);
        self.schedule.run(&mut self.world);
        let mut work = self.world.resource_mut::<DayWork>();
        if let Some((phase, reason)) = work.failure.take() {
            return Err(SimulationError::DayFailed {
                day: next_day,
                phase,
                reason,
            });
        }
        Ok(work
            .report
            .take()
            .expect("successful commit creates a day report"))
    }

    /// Replay uses the same validated command path. Log order is semantic, never ECS order.
    pub fn replay(
        scenario: Scenario,
        commands: &[RecordedCommand],
        through_day: u64,
    ) -> Result<Self, SimulationError> {
        crate::save::validate_command_log(commands, through_day)?;
        let mut simulation = Self::from_scenario(scenario)?;
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
