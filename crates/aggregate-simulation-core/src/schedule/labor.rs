use crate::world_storage::{
    ConstructionProject, DayWork, DefinitionRegistry, Facility, PopulationGroup, WorkforceLimits,
};
use aggregate_economy::{LaborRequest, allocate_labor};
use aggregate_world::ProvinceId;
use bevy_ecs::prelude::*;
use std::collections::BTreeMap;

#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn allocate_workers(
    population: Query<&PopulationGroup>,
    facilities: Query<&Facility>,
    projects: Query<&ConstructionProject>,
    definitions: Res<DefinitionRegistry>,
    workforce_limits: Res<WorkforceLimits>,
    mut work: ResMut<DayWork>,
) {
    if work.failure.is_some() {
        return;
    }
    if let Err(reason) = plan(
        &population,
        &facilities,
        &projects,
        &definitions,
        &workforce_limits,
        &mut work,
    ) {
        work.fail("labor_allocation", reason);
    }
}

fn plan(
    population: &Query<&PopulationGroup>,
    facilities: &Query<&Facility>,
    projects: &Query<&ConstructionProject>,
    definitions: &DefinitionRegistry,
    workforce_limits: &WorkforceLimits,
    work: &mut DayWork,
) -> Result<(), String> {
    for group in population {
        let province = work
            .provinces
            .get_mut(&group.0.province)
            .ok_or_else(|| format!("missing population province {}", group.0.province))?;
        province.report.population = province
            .report
            .population
            .checked_add(group.0.population)
            .ok_or("province population overflow")?;
        province.report.available_workers = province
            .report
            .available_workers
            .checked_add(
                workforce_limits
                    .0
                    .get(&group.0.id)
                    .copied()
                    .unwrap_or(group.0.workforce),
            )
            .ok_or("province workforce overflow")?;
    }
    let mut requests: BTreeMap<ProvinceId, Vec<LaborRequest>> = BTreeMap::new();
    for facility in facilities {
        let definition = definitions
            .facilities
            .get(&facility.0.definition)
            .ok_or("missing facility definition")?;
        let requested = definition
            .workers_per_level
            .checked_mul(facility.0.level)
            .ok_or("facility worker capacity overflow")?;
        requests
            .entry(facility.0.province.clone())
            .or_default()
            .push(LaborRequest {
                facility_id: facility.0.id.clone(),
                requested_workers: requested,
            });
    }
    for project in projects {
        requests
            .entry(project.0.province.clone())
            .or_default()
            .push(LaborRequest {
                facility_id: project.0.facility_id.clone(),
                requested_workers: project
                    .0
                    .requested_workers
                    .min(project.0.remaining_worker_days),
            });
    }
    for (province_id, province) in &mut work.provinces {
        let allocations = allocate_labor(
            province.report.available_workers,
            requests.get(province_id).map(Vec::as_slice).unwrap_or(&[]),
        )
        .map_err(|error| error.to_string())?;
        province.allocations = allocations
            .into_iter()
            .map(|allocation| (allocation.facility_id, allocation.allocated_workers))
            .collect();
    }
    Ok(())
}
