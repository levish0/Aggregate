use crate::day_work::ConstructionProgress;
use crate::day_work::DayWork;
use aggregate_programs::{
    {ConstructionDayReport, SimulationEvent},
    world_storage::ConstructionProject,
};
use crate::{LaborRequest, allocate_labor};
use aggregate_world::ProvinceId;
use bevy_ecs::prelude::*;
use std::collections::BTreeMap;

#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn advance_construction(
    projects: Query<&ConstructionProject>,
    mut work: ResMut<DayWork>,
) {
    if work.failure.is_some() {
        return;
    }
    if let Err(reason) = plan(&projects, &mut work) {
        work.fail("construction", reason);
    }
}

fn plan(projects: &Query<&ConstructionProject>, work: &mut DayWork) -> Result<(), String> {
    let mut ordered: Vec<_> = projects.iter().collect();
    ordered.sort_by(|left, right| left.0.facility_id.cmp(&right.0.facility_id));
    let mut deficits: BTreeMap<ProvinceId, Vec<LaborRequest>> = BTreeMap::new();
    let mut reserved: BTreeMap<ProvinceId, u64> = BTreeMap::new();
    for project in &ordered {
        let province = &work.provinces[&project.0.province];
        let allocated = province
            .allocations
            .get(&project.0.facility_id)
            .copied()
            .unwrap_or(0);
        let demand = project
            .0
            .requested_workers
            .min(project.0.remaining_worker_days);
        let total = reserved.entry(project.0.province.clone()).or_default();
        *total = total
            .checked_add(allocated)
            .ok_or("reserved construction workforce overflow")?;
        deficits
            .entry(project.0.province.clone())
            .or_default()
            .push(LaborRequest {
                facility_id: project.0.facility_id.clone(),
                requested_workers: demand
                    .checked_sub(allocated)
                    .ok_or("construction allocation exceeds demand")?,
            });
    }
    // Production has already planned its actual work. Reuse only workers who did no
    // production, preserving original construction allocations and per-project caps.
    for (province_id, requests) in deficits {
        let province = work
            .provinces
            .get_mut(&province_id)
            .expect("validated province");
        let available = province
            .report
            .available_workers
            .checked_sub(province.report.production_workers)
            .and_then(|workers| workers.checked_sub(reserved[&province_id]))
            .ok_or("daily workforce overallocated")?;
        for allocation in allocate_labor(available, &requests).map_err(|error| error.to_string())? {
            if allocation.allocated_workers > 0 {
                tracing::debug!(day = work.day, province = %province_id, facility = %allocation.facility_id, workers = allocation.allocated_workers, committed = false, "Unused workforce reassignment planned");
            }
            let current = province
                .allocations
                .entry(allocation.facility_id)
                .or_default();
            *current = current
                .checked_add(allocation.allocated_workers)
                .ok_or("construction workforce overflow")?;
        }
    }
    for project in ordered {
        let province = work
            .provinces
            .get_mut(&project.0.province)
            .expect("validated project province");
        let allocated = province
            .allocations
            .get(&project.0.facility_id)
            .copied()
            .unwrap_or(0);
        let actual = allocated.min(project.0.remaining_worker_days);
        let total = province
            .report
            .construction_workers
            .checked_add(actual)
            .ok_or("construction workforce overflow")?;
        province.report.construction_workers = total;
        let remaining = project.0.remaining_worker_days - actual;
        if actual
            < project
                .0
                .requested_workers
                .min(project.0.remaining_worker_days)
        {
            tracing::debug!(day = work.day, province = %project.0.province, facility = %project.0.facility_id, active_workers = actual, remaining_worker_days = remaining, committed = false, "Construction plan limited by local workforce");
        }
        work.constructions.insert(
            project.0.facility_id.clone(),
            ConstructionProgress {
                remaining_worker_days: remaining,
            },
        );
        work.report
            .as_mut()
            .expect("initialized report")
            .constructions
            .push(ConstructionDayReport {
                facility: project.0.facility_id.clone(),
                requested_workers: project
                    .0
                    .requested_workers
                    .min(project.0.remaining_worker_days),
                active_workers: actual,
                remaining_worker_days: remaining,
            });
        if remaining == 0 {
            work.report
                .as_mut()
                .expect("initialized report")
                .events
                .push(SimulationEvent::ConstructionCompleted {
                    province: project.0.province.clone(),
                    facility: project.0.facility_id.clone(),
                });
        }
    }
    Ok(())
}
