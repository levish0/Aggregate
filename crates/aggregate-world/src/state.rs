use crate::{CountryId, FacilityDefinitionId, FacilityId, GoodId, PopulationGroupId, ProvinceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CountryState {
    pub id: CountryId,
    pub name: String,
    #[serde(default)]
    pub name_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvinceState {
    pub id: ProvinceId,
    pub country: CountryId,
    pub name: String,
    #[serde(default)]
    pub name_key: Option<String>,
    pub stockpile: BTreeMap<GoodId, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationGroupState {
    pub id: PopulationGroupId,
    pub province: ProvinceId,
    pub population: u64,
    pub workforce: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityState {
    pub id: FacilityId,
    pub province: ProvinceId,
    pub definition: FacilityDefinitionId,
    pub level: u64,
    /// Lower values reserve production inputs first; persistent IDs break ties.
    /// Labor is allocated proportionally and does not use this priority.
    pub production_priority: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionProjectState {
    pub facility_id: FacilityId,
    pub province: ProvinceId,
    pub definition: FacilityDefinitionId,
    pub requested_workers: u64,
    pub remaining_worker_days: u64,
    /// Production priority inherited by the completed facility, not construction labor priority.
    pub production_priority: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldSnapshot {
    pub day: u64,
    pub countries: Vec<CountryState>,
    pub provinces: Vec<ProvinceState>,
    pub population_groups: Vec<PopulationGroupState>,
    pub facilities: Vec<FacilityState>,
    pub construction_projects: Vec<ConstructionProjectState>,
}

impl WorldSnapshot {
    /// Stable ordering for persistence, replay comparisons, and tie breaking.
    pub fn normalize(&mut self) {
        self.countries.sort_by(|left, right| left.id.cmp(&right.id));
        self.provinces.sort_by(|left, right| left.id.cmp(&right.id));
        self.population_groups
            .sort_by(|left, right| left.id.cmp(&right.id));
        self.facilities
            .sort_by(|left, right| left.id.cmp(&right.id));
        self.construction_projects
            .sort_by(|left, right| left.facility_id.cmp(&right.facility_id));
    }
}
