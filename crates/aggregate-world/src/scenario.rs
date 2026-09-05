use crate::{ContentDefinitions, WorldRules, WorldSnapshot};
use serde::{Deserialize, Serialize};

pub const SCENARIO_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub rules: WorldRules,
    pub definitions: ContentDefinitions,
    pub initial_state: WorldSnapshot,
}
