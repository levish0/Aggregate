use aggregate_world::PopulationGroupId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthParameters {
    pub transmission_per_thousand: u64,
    pub recovery_per_thousand: u64,
    pub initial_infected_per_thousand: u64,
}
impl Default for HealthParameters {
    fn default() -> Self {
        Self {
            transmission_per_thousand: 240,
            recovery_per_thousand: 100,
            initial_infected_per_thousand: 10,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HealthState {
    pub parameters: HealthParameters,
    pub groups: BTreeMap<PopulationGroupId, HealthGroup>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HealthGroup {
    pub susceptible: u64,
    pub infected: u64,
    pub recovered: u64,
    pub exposure_remainder: u64,
    pub recovery_remainder: u64,
}
