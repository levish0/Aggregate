use crate::{Simulation, SimulationError, RecordedCommand};
use aggregate_programs::{ProgramRuntime, SavedProgramState, SimulationProgram};
use aggregate_scenario::{validate_scenario, validate_world_state};
use aggregate_world::{Scenario, WorldSnapshot};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const SAVE_SCHEMA_VERSION: u32 = 4;
/// Increment when the semantics/order of native rules change. Saves include their definitions.
pub const RULESET_VERSION: &str = "aggregate-program-runtime/1";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SimulationSave {
    schema_version: u32,
    ruleset_version: String,
    scenario: Scenario,
    current_state: WorldSnapshot,
    commands: Vec<RecordedCommand>,
    programs: Vec<SavedProgramState>,
}

impl Simulation {
    #[tracing::instrument(level = "info", skip_all, fields(day = self.clock().day()), err)]
    pub fn save_json(&mut self) -> Result<String, SimulationError> {
        let save = SimulationSave {
            schema_version: SAVE_SCHEMA_VERSION,
            ruleset_version: RULESET_VERSION.into(),
            scenario: self.initial_scenario.clone(),
            current_state: self.snapshot(),
            commands: self.commands.clone(),
            programs: self.programs.snapshot(),
        };
        let encoded = serde_json::to_string_pretty(&save)
            .map_err(|error| SimulationError::InvalidSave(error.to_string()))?;
        tracing::info!(
            day = save.current_state.day,
            bytes = encoded.len(),
            programs = save.programs.len(),
            "Save serialized"
        );
        Ok(encoded)
    }

    pub fn from_save_json(source: &str) -> Result<Self, SimulationError> {
        Self::from_save_json_with_programs(source, Vec::new())
    }

    #[tracing::instrument(level = "info", skip_all, fields(bytes = source.len()), err)]
    pub fn from_save_json_with_programs(
        source: &str,
        available: Vec<Arc<dyn SimulationProgram>>,
    ) -> Result<Self, SimulationError> {
        let mut deserializer = serde_json::Deserializer::from_str(source);
        let mut save: SimulationSave = serde_path_to_error::deserialize(&mut deserializer)
            .map_err(|error| SimulationError::InvalidSave(error.to_string()))?;
        deserializer
            .end()
            .map_err(|error| SimulationError::InvalidSave(error.to_string()))?;
        if save.schema_version != SAVE_SCHEMA_VERSION {
            return Err(SimulationError::InvalidSave(format!(
                "unsupported schema_version {}",
                save.schema_version
            )));
        }
        if save.ruleset_version != RULESET_VERSION {
            return Err(SimulationError::InvalidSave(format!(
                "unsupported ruleset_version {}",
                save.ruleset_version
            )));
        }
        validate_scenario(&save.scenario)?;
        validate_world_state(
            &save.scenario.definitions,
            &save.scenario.rules,
            &save.current_state,
        )?;
        validate_command_log(&save.commands, save.current_state.day)?;
        save.current_state.normalize();
        let programs = ProgramRuntime::restore(available, save.programs, &save.current_state)
            .map_err(SimulationError::InvalidPrograms)?;
        let mut simulation =
            Self::from_validated_state(save.scenario, &save.current_state, save.commands);
        simulation.programs = programs;
        simulation.programs.install(&mut simulation.world).map_err(SimulationError::InvalidPrograms)?;
        tracing::info!(
            day = simulation.clock().day(),
            commands = simulation.commands.len(),
            "Save restored"
        );
        Ok(simulation)
    }

    /// Hash canonical domain state, excluding ECS memory layout, transient reports and caches.
    pub fn state_hash(&mut self) -> Result<String, SimulationError> {
        let canonical = serde_json::to_vec(&(self.snapshot(), self.programs.snapshot()))
            .map_err(|error| SimulationError::InvalidSave(error.to_string()))?;
        Ok(blake3::hash(&canonical).to_hex().to_string())
    }
}

pub(crate) fn validate_command_log(
    commands: &[RecordedCommand],
    through_day: u64,
) -> Result<(), SimulationError> {
    let mut previous_day = 0;
    for (index, command) in commands.iter().enumerate() {
        let expected = u64::try_from(index)
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| SimulationError::InvalidReplay("command sequence overflow".into()))?;
        if command.sequence != expected || command.day < previous_day || command.day > through_day {
            return Err(SimulationError::InvalidReplay(format!(
                "commands[{index}] has an invalid sequence/day"
            )));
        }
        previous_day = command.day;
    }
    Ok(())
}
