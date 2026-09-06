use crate::{FacilityDefinitionId, GoodId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoodDefinition {
    pub id: GoodId,
    pub name: String,
}

/// One level's construction cost. Goods are reserved when construction starts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionDefinition {
    pub construction_points: u64,
    pub max_workers: u64,
    pub goods: BTreeMap<GoodId, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityDefinition {
    pub id: FacilityDefinitionId,
    pub name: String,
    pub workers_per_level: u64,
    /// Non-storable national construction service, produced only against current demand.
    #[serde(default)]
    pub construction_points_per_worker_day: u64,
    pub inputs_per_worker_day: BTreeMap<GoodId, u64>,
    pub outputs_per_worker_day: BTreeMap<GoodId, u64>,
    pub construction: ConstructionDefinition,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentDefinitions {
    pub goods: Vec<GoodDefinition>,
    pub facilities: Vec<FacilityDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldRules {
    pub staple_good: GoodId,
    pub consumption_per_person_day: u64,
}
