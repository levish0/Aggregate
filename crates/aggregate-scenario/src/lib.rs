//! Validated scenario file loading. JSON is a data format, not executable rules.

mod definition_validation;
mod error;
mod loading;
mod state_validation;
mod validation;

pub use error::ScenarioError;
pub use loading::{load_scenario, parse_scenario};
pub use validation::{validate_scenario, validate_world_state};
