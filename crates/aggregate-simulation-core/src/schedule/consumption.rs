use crate::{
    report::{GoodsFlow, GoodsFlowCause, SimulationEvent},
    world_storage::{DayWork, SimulationRules},
};
use aggregate_economy::plan_consumption;
use bevy_ecs::prelude::*;

pub(super) fn consume_goods(rules: Res<SimulationRules>, mut work: ResMut<DayWork>) {
    if work.failure.is_some() {
        return;
    }
    if let Err(reason) = plan(&rules, &mut work) {
        work.fail("consumption", reason);
    }
}

fn plan(rules: &SimulationRules, work: &mut DayWork) -> Result<(), String> {
    for (province_id, province) in &mut work.provinces {
        let available = province
            .stockpile
            .get(&rules.0.staple_good)
            .copied()
            .unwrap_or(0);
        let outcome = plan_consumption(
            province.report.population,
            rules.0.consumption_per_person_day,
            available,
        )
        .map_err(|error| error.to_string())?;
        province
            .stockpile
            .insert(rules.0.staple_good.clone(), available - outcome.consumed);
        province.report.food_required = outcome.required;
        province.report.food_consumed = outcome.consumed;
        province.report.food_shortfall = outcome.shortfall;
        let report = work.report.as_mut().expect("initialized report");
        if outcome.consumed > 0 {
            report.goods_flows.push(GoodsFlow {
                province: province_id.clone(),
                good: rules.0.staple_good.clone(),
                amount: outcome.consumed,
                cause: GoodsFlowCause::HouseholdConsumption,
            });
        }
        if outcome.shortfall > 0 {
            report.events.push(SimulationEvent::FoodShortfall {
                province: province_id.clone(),
                good: rules.0.staple_good.clone(),
                amount: outcome.shortfall,
            });
        }
    }
    Ok(())
}
