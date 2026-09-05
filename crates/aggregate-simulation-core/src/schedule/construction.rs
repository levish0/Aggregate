use crate::{
    report::SimulationEvent,
    world_storage::{ConstructionProgress, ConstructionProject, DayWork},
};
use bevy_ecs::prelude::*;

pub(super) fn advance_construction(
    projects: Query<&ConstructionProject>,
    mut work: ResMut<DayWork>,
) {
    if work.failure.is_some() {
        return;
    }
    let mut ordered: Vec<_> = projects.iter().collect();
    ordered.sort_by(|left, right| left.0.facility_id.cmp(&right.0.facility_id));
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
        let Some(total) = province.report.construction_workers.checked_add(actual) else {
            work.fail("construction", "construction workforce overflow");
            return;
        };
        province.report.construction_workers = total;
        let remaining = project.0.remaining_worker_days - actual;
        work.constructions.insert(
            project.0.facility_id.clone(),
            ConstructionProgress {
                remaining_worker_days: remaining,
            },
        );
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
}
