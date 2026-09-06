use crate::ModuleManifest;
use aggregate_world::{PopulationGroupId, WorldSnapshot};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct ModuleContext<'a> {
    pub day: u64,
    /// Committed state at the beginning of the day. Never partially computed state.
    pub world: &'a WorldSnapshot,
    /// Only declared dependencies, also from the beginning of the day.
    pub dependencies: BTreeMap<&'a str, &'a Value>,
}

pub struct ModulePlan {
    /// Module-owned serialized typed state, validated before publication.
    pub next_state: Value,
    /// Temporary effective workforce limits; multiple limits combine by minimum.
    /// Population and underlying workforce are not permanently changed.
    pub workforce_limits: BTreeMap<PopulationGroupId, u64>,
}

/// Implement mechanisms with ordinary Rust structs/functions. Return proposals;
/// never mutate the engine or perform external side effects inside a calculation.
/// All persistent state belongs in the payload, not interior mutable trait fields.
pub trait SimulationModule: Send + Sync {
    fn manifest(&self) -> ModuleManifest;
    fn initialize(&self, world: &WorldSnapshot) -> Result<Value, String>;
    fn validate_state(&self, world: &WorldSnapshot, state: &Value) -> Result<(), String>;
    fn plan_day(&self, context: &ModuleContext<'_>, state: &Value) -> Result<ModulePlan, String>;
}
