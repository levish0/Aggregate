use crate::{
    ScenarioError,
    validation::{require_positive, validate_name, validate_unique_identifiers},
};
use aggregate_world::{ContentDefinitions, GoodId, WorldRules};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn validate_definitions(
    definitions: &ContentDefinitions,
    rules: &WorldRules,
) -> Result<(), ScenarioError> {
    validate_unique_identifiers(
        definitions.goods.iter().map(|good| good.id.0.as_str()),
        "definitions.goods",
        "id",
    )?;
    validate_unique_identifiers(
        definitions
            .facilities
            .iter()
            .map(|facility| facility.id.0.as_str()),
        "definitions.facilities",
        "id",
    )?;
    let goods: BTreeSet<_> = definitions.goods.iter().map(|good| &good.id).collect();
    for (index, good) in definitions.goods.iter().enumerate() {
        validate_name(&good.name, &format!("definitions.goods[{index}].name"))?;
    }
    if !goods.contains(&rules.staple_good) {
        return Err(ScenarioError::new(
            "rules.staple_good",
            format!("unknown good `{}`", rules.staple_good),
        ));
    }
    require_positive(
        rules.consumption_per_person_day,
        "rules.consumption_per_person_day",
    )?;
    for (index, facility) in definitions.facilities.iter().enumerate() {
        let path = format!("definitions.facilities[{index}]");
        validate_name(&facility.name, &format!("{path}.name"))?;
        require_positive(
            facility.workers_per_level,
            &format!("{path}.workers_per_level"),
        )?;
        validate_good_quantities(
            &facility.inputs_per_worker_day,
            &goods,
            &format!("{path}.inputs_per_worker_day"),
        )?;
        if facility.outputs_per_worker_day.is_empty() {
            return Err(ScenarioError::new(
                format!("{path}.outputs_per_worker_day"),
                "a production facility must define at least one output",
            ));
        }
        validate_good_quantities(
            &facility.outputs_per_worker_day,
            &goods,
            &format!("{path}.outputs_per_worker_day"),
        )?;
        require_positive(
            facility.construction.construction_points,
            &format!("{path}.construction.construction_points"),
        )?;
        require_positive(
            facility.construction.max_workers,
            &format!("{path}.construction.max_workers"),
        )?;
        validate_good_quantities(
            &facility.construction.goods,
            &goods,
            &format!("{path}.construction.goods"),
        )?;
    }
    Ok(())
}

fn validate_good_quantities(
    quantities: &BTreeMap<GoodId, u64>,
    known_goods: &BTreeSet<&GoodId>,
    path: &str,
) -> Result<(), ScenarioError> {
    for (good, quantity) in quantities {
        let quantity_path = format!("{path}[{:?}]", good.0);
        if !known_goods.contains(good) {
            return Err(ScenarioError::new(
                quantity_path,
                format!("unknown good `{good}`"),
            ));
        }
        require_positive(*quantity, &quantity_path)?;
    }
    Ok(())
}
