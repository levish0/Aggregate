use super::ManagementSession;
use aggregate_world::{FacilityDefinition, ProvinceId};
use std::collections::{BTreeMap, BTreeSet};

/// Choose a local site with resources, preferring idle labor and shorter queues.
/// State construction does not teleport stockpiles or recruit across province borders.
pub fn construction_site(
    session: &ManagementSession,
    provinces: &BTreeSet<ProvinceId>,
    definition: &FacilityDefinition,
) -> Option<ProvinceId> {
    let mut workforce = BTreeMap::<&ProvinceId, u64>::new();
    let mut demand = BTreeMap::<&ProvinceId, u64>::new();
    let mut queues = BTreeMap::<&ProvinceId, usize>::new();
    for group in session
        .snapshot
        .population_groups
        .iter()
        .filter(|group| provinces.contains(&group.province))
    {
        *workforce.entry(&group.province).or_default() += group.workforce;
    }
    for facility in session
        .snapshot
        .facilities
        .iter()
        .filter(|facility| provinces.contains(&facility.province))
    {
        if let Some(definition) = session
            .definitions
            .facilities
            .iter()
            .find(|definition| definition.id == facility.definition)
        {
            *demand.entry(&facility.province).or_default() +=
                definition.workers_per_level * facility.level;
        }
    }
    for project in session
        .snapshot
        .construction_projects
        .iter()
        .filter(|project| provinces.contains(&project.province))
    {
        *demand.entry(&project.province).or_default() +=
            project.requested_workers.min(project.remaining_worker_days);
        *queues.entry(&project.province).or_default() += 1;
    }
    session
        .snapshot
        .provinces
        .iter()
        .filter(|province| {
            provinces.contains(&province.id) && province.country == session.player_country
        })
        .filter(|province| {
            definition
                .construction
                .goods
                .iter()
                .all(|(good, cost)| province.stockpile.get(good).copied().unwrap_or(0) >= *cost)
        })
        .max_by_key(|province| {
            (
                workforce
                    .get(&province.id)
                    .copied()
                    .unwrap_or(0)
                    .saturating_sub(demand.get(&province.id).copied().unwrap_or(0)),
                std::cmp::Reverse(queues.get(&province.id).copied().unwrap_or(0)),
                std::cmp::Reverse(province.id.clone()),
            )
        })
        .map(|province| province.id.clone())
}
