use crate::day_work::DayWork;
use crate::plan_production;
use aggregate_programs::{
    world_storage::{DefinitionRegistry, Facility},
    {FacilityDayReport, GoodsFlow, GoodsFlowCause},
};
use bevy_ecs::prelude::*;

#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn produce_goods(
    facilities: Query<&Facility>,
    definitions: Res<DefinitionRegistry>,
    mut work: ResMut<DayWork>,
) {
    if work.failure.is_some() {
        return;
    }
    if let Err(reason) = plan(&facilities, &definitions, &mut work) {
        work.fail("production", reason);
    }
}

fn plan(
    facilities: &Query<&Facility>,
    definitions: &DefinitionRegistry,
    work: &mut DayWork,
) -> Result<(), String> {
    let mut ordered: Vec<_> = facilities.iter().collect();
    // Explicit scenario production_priority, then persistent ID: inputs are drawn in this stable order.
    // Outputs are available to later facilities and to household consumption on this day.
    ordered.sort_by(|left, right| {
        (left.0.production_priority, &left.0.id).cmp(&(right.0.production_priority, &right.0.id))
    });
    for facility in ordered {
        let definition = definitions
            .facilities
            .get(&facility.0.definition)
            .ok_or("missing facility definition")?;
        let province = work
            .provinces
            .get_mut(&facility.0.province)
            .ok_or("missing facility province")?;
        let assigned = province
            .allocations
            .get(&facility.0.id)
            .copied()
            .unwrap_or(0);
        let outcome = plan_production(definition, facility.0.level, assigned, &province.stockpile)
            .map_err(|error| error.to_string())?;
        for (good, amount) in &outcome.inputs {
            let available = province.stockpile.get(good).copied().unwrap_or(0);
            province.stockpile.insert(
                good.clone(),
                available
                    .checked_sub(*amount)
                    .ok_or("planned inputs exceed stockpile")?,
            );
        }
        for (good, amount) in &outcome.outputs {
            let available = province.stockpile.get(good).copied().unwrap_or(0);
            province.stockpile.insert(
                good.clone(),
                available.checked_add(*amount).ok_or_else(|| {
                    format!(
                        "facility {}, good {good}: stockpile overflow",
                        facility.0.id
                    )
                })?,
            );
        }
        province.report.production_workers = province
            .report
            .production_workers
            .checked_add(outcome.active_workers)
            .ok_or("production workers overflow")?;
        let report = work
            .report
            .as_mut()
            .expect("begin_day initialized the report");
        for (good, amount) in &outcome.inputs {
            if *amount > 0 {
                report.goods_flows.push(GoodsFlow {
                    province: facility.0.province.clone(),
                    good: good.clone(),
                    amount: *amount,
                    cause: GoodsFlowCause::ProductionInput {
                        facility: facility.0.id.clone(),
                    },
                });
            }
        }
        for (good, amount) in &outcome.outputs {
            if *amount > 0 {
                report.goods_flows.push(GoodsFlow {
                    province: facility.0.province.clone(),
                    good: good.clone(),
                    amount: *amount,
                    cause: GoodsFlowCause::ProductionOutput {
                        facility: facility.0.id.clone(),
                    },
                });
            }
        }
        report.facilities.push(FacilityDayReport {
            facility: facility.0.id.clone(),
            assigned_workers: outcome.assigned_workers,
            active_workers: outcome.active_workers,
            inputs: outcome.inputs,
            outputs: outcome.outputs,
        });
    }
    Ok(())
}
