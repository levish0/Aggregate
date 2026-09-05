use crate::{ScenarioError, validate_scenario};
use aggregate_world::Scenario;
use std::{fs, path::Path};

pub fn load_scenario(path: impl AsRef<Path>) -> Result<Scenario, ScenarioError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(|error| ScenarioError {
        source_path: Some(path.to_owned()),
        field_path: "$".to_owned(),
        message: error.to_string(),
    })?;
    parse_scenario(&source).map_err(|mut error| {
        error.source_path = Some(path.to_owned());
        error
    })
}

pub fn parse_scenario(source: &str) -> Result<Scenario, ScenarioError> {
    let mut deserializer = serde_json::Deserializer::from_str(source);
    let mut scenario: Scenario = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|error| ScenarioError::new(error.path().to_string(), error.inner().to_string()))?;
    deserializer
        .end()
        .map_err(|error| ScenarioError::new("$", error.to_string()))?;
    validate_scenario(&scenario)?;
    scenario.initial_state.normalize();
    Ok(scenario)
}
