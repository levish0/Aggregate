use crate::day_work::DayWork;
use aggregate_programs::{
    world_storage::{ConstructionProject, Facility, Province},
};
use aggregate_world::FacilityState;
use bevy_ecs::prelude::*;

pub(super) fn commit_day(
    mut commands: Commands,
    mut provinces: Query<&mut Province>,
    mut projects: Query<(Entity, &mut ConstructionProject)>,
    mut work: ResMut<DayWork>,
) {
    if work.failure.is_some() {
        return;
    }
    for mut province in &mut provinces {
        let stockpile = &work.provinces[&province.0.id].stockpile;
        province.0.stockpile.clone_from(stockpile);
    }
    for (entity, mut project) in &mut projects {
        let remaining = work.constructions[&project.0.facility_id].remaining_worker_days;
        if remaining == 0 {
            commands.spawn(Facility(FacilityState {
                id: project.0.facility_id.clone(),
                province: project.0.province.clone(),
                definition: project.0.definition.clone(),
                level: 1,
                production_priority: project.0.production_priority,
            }));
            commands.entity(entity).despawn();
        } else {
            project.0.remaining_worker_days = remaining;
        }
    }
    let province_reports = work
        .provinces
        .values()
        .map(|province| province.report.clone())
        .collect();
    work.report.as_mut().expect("initialized report").provinces = province_reports;
}
