use aggregate_programs::{DayReport, ProvinceDayReport};
use aggregate_world::*;
use bevy_ecs::prelude::*;
use std::collections::BTreeMap;

pub(crate) struct ProvincePlan {
    pub stockpile: BTreeMap<GoodId, u64>,
    pub allocations: BTreeMap<FacilityId, u64>,
    pub report: ProvinceDayReport,
}

pub(crate) struct ConstructionProgress {
    pub remaining_construction_points: u64,
}

/// Only proposed stockpile/progress changes are staged; authoritative entities are unchanged
/// until every phase succeeds. There is no full-world clone or rollback per simulation day.
#[derive(Resource, Default)]
pub(crate) struct DayWork {
    pub day: u64,
    pub provinces: BTreeMap<ProvinceId, ProvincePlan>,
    pub constructions: BTreeMap<FacilityId, ConstructionProgress>,
    pub report: Option<DayReport>,
    pub failure: Option<(&'static str, String)>,
}

impl DayWork {
    pub fn fail(&mut self, phase: &'static str, reason: impl ToString) {
        if self.failure.is_none() {
            self.failure = Some((phase, reason.to_string()));
        }
    }
}
