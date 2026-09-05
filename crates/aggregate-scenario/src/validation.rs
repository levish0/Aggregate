use crate::{ScenarioError, definition_validation, state_validation};
use aggregate_world::{
    ContentDefinitions, SCENARIO_SCHEMA_VERSION, Scenario, WorldRules, WorldSnapshot,
};
use std::collections::BTreeSet;
use uuid::Uuid;

pub fn validate_scenario(scenario: &Scenario) -> Result<(), ScenarioError> {
    if scenario.schema_version != SCENARIO_SCHEMA_VERSION {
        return Err(ScenarioError::new(
            "schema_version",
            format!(
                "unsupported version {}; expected {SCENARIO_SCHEMA_VERSION}",
                scenario.schema_version
            ),
        ));
    }
    validate_identifier(&scenario.id, "id")?;
    validate_name(&scenario.name, "name")?;
    definition_validation::validate_definitions(&scenario.definitions, &scenario.rules)?;
    state_validation::validate_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
        "initial_state",
    )?;
    if scenario.initial_state.day != 0 {
        return Err(ScenarioError::new(
            "initial_state.day",
            "must start at day zero",
        ));
    }
    if !scenario.initial_state.construction_projects.is_empty() {
        return Err(ScenarioError::new(
            "initial_state.construction_projects",
            "scenario construction projects must be empty; start them with simulation commands",
        ));
    }
    Ok(())
}

/// Validates snapshots restored from saves, including in-progress construction.
pub fn validate_world_state(
    definitions: &ContentDefinitions,
    rules: &WorldRules,
    state: &WorldSnapshot,
) -> Result<(), ScenarioError> {
    definition_validation::validate_definitions(definitions, rules)?;
    state_validation::validate_state(definitions, rules, state, "state")
}

pub(crate) fn validate_identifier(value: &str, path: &str) -> Result<(), ScenarioError> {
    if value.is_empty() || value.trim() != value {
        return Err(ScenarioError::new(
            path,
            "identifier must be nonempty and have no surrounding whitespace",
        ));
    }
    Ok(())
}

pub(crate) fn validate_name(value: &str, path: &str) -> Result<(), ScenarioError> {
    if value.trim().is_empty() {
        return Err(ScenarioError::new(path, "name must not be blank"));
    }
    Ok(())
}

pub(crate) fn validate_unique_identifiers<'a>(
    identifiers: impl IntoIterator<Item = &'a str>,
    collection_path: &str,
    id_field: &str,
) -> Result<(), ScenarioError> {
    let mut known = BTreeSet::new();
    for (index, identifier) in identifiers.into_iter().enumerate() {
        let path = format!("{collection_path}[{index}].{id_field}");
        validate_identifier(identifier, &path)?;
        if !known.insert(identifier) {
            return Err(ScenarioError::new(
                path,
                format!("duplicate identifier `{identifier}`"),
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_unique_uuid_identifiers(
    identifiers: impl IntoIterator<Item = Uuid>,
    collection_path: &str,
    id_field: &str,
) -> Result<(), ScenarioError> {
    let mut known = BTreeSet::new();
    for (index, identifier) in identifiers.into_iter().enumerate() {
        let path = format!("{collection_path}[{index}].{id_field}");
        if identifier.is_nil() {
            return Err(ScenarioError::new(
                path,
                "instance identifier must not be the nil UUID",
            ));
        }
        if !known.insert(identifier) {
            return Err(ScenarioError::new(
                path,
                format!("duplicate instance identifier `{identifier}`"),
            ));
        }
    }
    Ok(())
}

pub(crate) fn require_positive(value: u64, path: &str) -> Result<(), ScenarioError> {
    if value == 0 {
        return Err(ScenarioError::new(path, "must be greater than zero"));
    }
    Ok(())
}

pub(crate) fn add_checked(total: &mut u64, amount: u64, path: &str) -> Result<(), ScenarioError> {
    *total = total.checked_add(amount).ok_or_else(|| {
        ScenarioError::new(
            path,
            "aggregate exceeds the supported unsigned 64-bit range",
        )
    })?;
    Ok(())
}
