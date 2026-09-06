use std::collections::BTreeMap;

use aggregate_world::{FacilityDefinition, GoodId};

use crate::EconomyError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionOutcome {
    pub assigned_workers: u64,
    pub active_workers: u64,
    pub inputs: BTreeMap<GoodId, u64>,
    pub outputs: BTreeMap<GoodId, u64>,
}

/// Plan whole worker-day production batches from a beginning-of-phase stockpile.
///
/// Assignment does not guarantee activity: level capacity and the scarcest input
/// limit active workers. Outputs never satisfy inputs during this same plan,
/// including when a recipe consumes and produces the same good. No state changes
/// occur here; callers must reserve shared inputs between competing facilities.
pub fn plan_production(
    definition: &FacilityDefinition,
    level: u64,
    assigned_workers: u64,
    available_goods: &BTreeMap<GoodId, u64>,
) -> Result<ProductionOutcome, EconomyError> {
    validate_production_definition(definition)?;
    if level == 0 {
        return Err(EconomyError::InvalidFacilityLevel);
    }

    let worker_capacity = definition.workers_per_level.checked_mul(level).ok_or(
        EconomyError::ArithmeticOverflow {
            calculation: "facility worker capacity",
        },
    )?;
    let mut active_workers = assigned_workers.min(worker_capacity);

    for (good_id, units_per_worker) in &definition.inputs_per_worker_day {
        let available_units = available_goods.get(good_id).copied().unwrap_or_default();
        active_workers = active_workers.min(available_units / units_per_worker);
    }

    let inputs = production_quantities(
        &definition.inputs_per_worker_day,
        active_workers,
        "production inputs",
    )?;
    let outputs = production_quantities(
        &definition.outputs_per_worker_day,
        active_workers,
        "production outputs",
    )?;

    Ok(ProductionOutcome {
        assigned_workers,
        active_workers,
        inputs,
        outputs,
    })
}

fn validate_production_definition(definition: &FacilityDefinition) -> Result<(), EconomyError> {
    if definition.workers_per_level == 0 {
        return Err(EconomyError::InvalidProductionDefinition {
            reason: "workers per level must be positive",
        });
    }
    if definition.outputs_per_worker_day.is_empty() && definition.construction_points_per_worker_day == 0 {
        return Err(EconomyError::InvalidProductionDefinition {
            reason: "a production recipe must define at least one output",
        });
    }
    if definition
        .inputs_per_worker_day
        .values()
        .chain(definition.outputs_per_worker_day.values())
        .any(|quantity| *quantity == 0)
    {
        return Err(EconomyError::InvalidProductionDefinition {
            reason: "input and output quantities per worker-day must be positive",
        });
    }
    Ok(())
}

fn production_quantities(
    per_worker_quantities: &BTreeMap<GoodId, u64>,
    active_workers: u64,
    calculation: &'static str,
) -> Result<BTreeMap<GoodId, u64>, EconomyError> {
    per_worker_quantities
        .iter()
        .map(|(good_id, quantity)| {
            quantity
                .checked_mul(active_workers)
                .map(|total| (good_id.clone(), total))
                .ok_or(EconomyError::ArithmeticOverflow { calculation })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use aggregate_world::{ConstructionDefinition, FacilityDefinitionId};

    use super::*;

    fn goods(entries: &[(&str, u64)]) -> BTreeMap<GoodId, u64> {
        entries
            .iter()
            .map(|(good_id, quantity)| (GoodId::from(*good_id), *quantity))
            .collect()
    }

    fn steelworks() -> FacilityDefinition {
        FacilityDefinition {
            construction_points_per_worker_day: 0,
            id: FacilityDefinitionId::from("steelworks"),
            name: "Steelworks".to_owned(),
            workers_per_level: 20,
            inputs_per_worker_day: goods(&[("coal", 2), ("ore", 3)]),
            outputs_per_worker_day: goods(&[("steel", 4)]),
            construction: ConstructionDefinition {
                construction_points: 50,
                max_workers: 10,
                goods: goods(&[("steel", 5)]),
            },
        }
    }

    #[test]
    fn scarcest_input_limits_whole_worker_day_batches() {
        let outcome =
            plan_production(&steelworks(), 1, 10, &goods(&[("coal", 9), ("ore", 100)])).unwrap();
        assert_eq!(outcome.assigned_workers, 10);
        assert_eq!(outcome.active_workers, 4);
        assert_eq!(outcome.inputs, goods(&[("coal", 8), ("ore", 12)]));
        assert_eq!(outcome.outputs, goods(&[("steel", 16)]));
    }

    #[test]
    fn capacity_and_assigned_workforce_both_bound_production() {
        let stockpile = goods(&[("coal", 1_000), ("ore", 1_000)]);
        assert_eq!(
            plan_production(&steelworks(), 2, 100, &stockpile)
                .unwrap()
                .active_workers,
            40
        );
        assert_eq!(
            plan_production(&steelworks(), 2, 3, &stockpile)
                .unwrap()
                .active_workers,
            3
        );
    }

    #[test]
    fn missing_inputs_and_zero_assignments_produce_nothing() {
        let missing_ore = plan_production(&steelworks(), 1, 10, &goods(&[("coal", 100)])).unwrap();
        assert_eq!(missing_ore.active_workers, 0);
        assert!(missing_ore.inputs.values().all(|quantity| *quantity == 0));
        assert!(missing_ore.outputs.values().all(|quantity| *quantity == 0));

        let unstaffed =
            plan_production(&steelworks(), 1, 0, &goods(&[("coal", 100), ("ore", 100)])).unwrap();
        assert_eq!(unstaffed.active_workers, 0);
    }

    #[test]
    fn same_good_output_cannot_fund_its_own_input_and_stockpile_is_unchanged() {
        let mut definition = steelworks();
        definition.inputs_per_worker_day = goods(&[("grain", 2)]);
        definition.outputs_per_worker_day = goods(&[("grain", 3)]);
        let stockpile = goods(&[("grain", 3)]);
        let before = stockpile.clone();

        let outcome = plan_production(&definition, 1, 10, &stockpile).unwrap();

        assert_eq!(outcome.active_workers, 1);
        assert_eq!(outcome.inputs, goods(&[("grain", 2)]));
        assert_eq!(outcome.outputs, goods(&[("grain", 3)]));
        assert_eq!(stockpile, before);
    }

    #[test]
    fn input_free_recipes_still_require_workers() {
        let mut definition = steelworks();
        definition.inputs_per_worker_day.clear();
        let outcome = plan_production(&definition, 1, 5, &BTreeMap::new()).unwrap();
        assert_eq!(outcome.active_workers, 5);
        assert!(outcome.inputs.is_empty());
        assert_eq!(outcome.outputs, goods(&[("steel", 20)]));
    }

    #[test]
    fn rejects_invalid_recipes_and_unbuilt_facilities() {
        let mut definition = steelworks();
        assert_eq!(
            plan_production(&definition, 0, 10, &BTreeMap::new()),
            Err(EconomyError::InvalidFacilityLevel)
        );

        definition.workers_per_level = 0;
        assert!(matches!(
            plan_production(&definition, 1, 0, &BTreeMap::new()),
            Err(EconomyError::InvalidProductionDefinition { .. })
        ));

        definition = steelworks();
        definition
            .inputs_per_worker_day
            .insert(GoodId::from("coal"), 0);
        assert!(matches!(
            plan_production(&definition, 1, 0, &BTreeMap::new()),
            Err(EconomyError::InvalidProductionDefinition { .. })
        ));

        definition = steelworks();
        definition
            .outputs_per_worker_day
            .insert(GoodId::from("steel"), 0);
        assert!(matches!(
            plan_production(&definition, 1, 0, &BTreeMap::new()),
            Err(EconomyError::InvalidProductionDefinition { .. })
        ));

        definition.outputs_per_worker_day.clear();
        assert!(matches!(
            plan_production(&definition, 1, 0, &BTreeMap::new()),
            Err(EconomyError::InvalidProductionDefinition { .. })
        ));
    }

    #[test]
    fn rejects_capacity_and_output_overflow() {
        let mut definition = steelworks();
        definition.workers_per_level = u64::MAX;
        assert!(matches!(
            plan_production(&definition, 2, 0, &BTreeMap::new()),
            Err(EconomyError::ArithmeticOverflow { .. })
        ));

        definition = steelworks();
        definition.inputs_per_worker_day.clear();
        definition.outputs_per_worker_day = goods(&[("steel", u64::MAX)]);
        assert!(matches!(
            plan_production(&definition, 1, 2, &BTreeMap::new()),
            Err(EconomyError::ArithmeticOverflow { .. })
        ));
    }
}
