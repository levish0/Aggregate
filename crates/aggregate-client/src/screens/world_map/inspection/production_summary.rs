use crate::management::ManagementSession;
use aggregate_world::{FacilityDefinitionId, GoodId, ProvinceId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct ProductionSummary {
    pub capacity: u128,
    pub active_workers: u128,
    pub inputs: BTreeMap<GoodId, u128>,
    pub outputs: BTreeMap<GoodId, u128>,
    pub construction_points: u128,
}

pub(super) fn summarize(
    session: &ManagementSession,
    provinces: &BTreeSet<ProvinceId>,
) -> BTreeMap<FacilityDefinitionId, ProductionSummary> {
    let definitions: BTreeMap<_, _> = session
        .definitions
        .facilities
        .iter()
        .map(|item| (&item.id, item))
        .collect();
    let mut summaries = BTreeMap::<FacilityDefinitionId, ProductionSummary>::new();
    let mut facilities = BTreeMap::new();
    for facility in session
        .snapshot
        .facilities
        .iter()
        .filter(|item| provinces.contains(&item.province))
    {
        facilities.insert(&facility.id, &facility.definition);
        if let Some(definition) = definitions.get(&facility.definition) {
            summaries
                .entry(facility.definition.clone())
                .or_default()
                .capacity += u128::from(facility.level) * u128::from(definition.workers_per_level);
        }
    }
    if let Some(report) = &session.last_report {
        for facility in &report.facilities {
            let Some(definition) = facilities.get(&facility.facility) else {
                continue;
            };
            let summary = summaries.entry((*definition).clone()).or_default();
            summary.active_workers += u128::from(facility.active_workers);
            summary.construction_points += u128::from(facility.construction_points);
            for (good, amount) in &facility.inputs {
                *summary.inputs.entry(good.clone()).or_default() += u128::from(*amount);
            }
            for (good, amount) in &facility.outputs {
                *summary.outputs.entry(good.clone()).or_default() += u128::from(*amount);
            }
        }
    }
    summaries
}
