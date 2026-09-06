use aggregate_world::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SimulationCommand {
    Program {
        program: String,
        command: String,
        payload: serde_json::Value,
    },
    StartConstruction {
        country: CountryId,
        province: ProvinceId,
        facility: FacilityId,
        definition: FacilityDefinitionId,
        workers: u64,
        production_priority: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedCommand {
    pub day: u64,
    pub sequence: u64,
    pub command: SimulationCommand,
}

impl SimulationCommand {
    pub fn program_id(&self) -> &str {
        match self {
            Self::StartConstruction { .. } => "aggregate.economy",
            Self::Program { program, .. } => program,
        }
    }
}
