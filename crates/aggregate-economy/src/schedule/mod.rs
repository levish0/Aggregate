use crate::day_work::{DayWork, ProvincePlan};
pub(crate) mod commit;
mod construction;
mod consumption;
mod labor;
mod production;

use aggregate_programs::{
    SimulationClock,
    {DayReport, ProvinceDayReport},
    world_storage::Province,
};
use bevy_ecs::prelude::*;
use std::collections::BTreeMap;

/// A day is an ordered transaction. Bevy component conflicts do not define economic order.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationPhase {
    BeginDay,
    LaborAllocation,
    Production,
    Construction,
    Consumption,
    Commit,
}

pub(crate) fn create_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.configure_sets(
        (
            SimulationPhase::BeginDay,
            SimulationPhase::LaborAllocation,
            SimulationPhase::Production,
            SimulationPhase::Construction,
            SimulationPhase::Consumption,
            SimulationPhase::Commit,
        )
            .chain(),
    );
    schedule.add_systems((
        begin_day.in_set(SimulationPhase::BeginDay),
        labor::allocate_workers.in_set(SimulationPhase::LaborAllocation),
        production::produce_goods.in_set(SimulationPhase::Production),
        construction::advance_construction.in_set(SimulationPhase::Construction),
        consumption::consume_goods.in_set(SimulationPhase::Consumption),
    ));
    schedule
}

fn begin_day(clock: Res<SimulationClock>, provinces: Query<&Province>, mut work: ResMut<DayWork>) {
    *work = DayWork::default();
    let Some(day) = clock.day().checked_add(1) else {
        work.fail(
            "begin_day",
            "simulation day exhausted its representable range",
        );
        return;
    };
    work.day = day;
    work.report = Some(DayReport {
        day,
        provinces: Vec::new(),
        facilities: Vec::new(),
        constructions: Vec::new(),
        goods_flows: Vec::new(),
        events: Vec::new(),
    });
    for province in &provinces {
        work.provinces.insert(
            province.0.id.clone(),
            ProvincePlan {
                stockpile: province.0.stockpile.clone(),
                allocations: BTreeMap::new(),
                report: ProvinceDayReport {
                    province: province.0.id.clone(),
                    population: 0,
                    available_workers: 0,
                    production_workers: 0,
                    construction_workers: 0,
                    food_required: 0,
                    food_consumed: 0,
                    food_shortfall: 0,
                },
            },
        );
    }
}
