//! Native extension contracts. No renderer, ECS access, or dynamic-library ABI.
mod manifest;
mod runtime;
mod simulation;

pub use manifest::{MODULE_API_VERSION, ModuleDependency, ModuleManifest};
pub use runtime::{ModuleRuntime, PreparedModules, SavedModuleState};
pub use semver::{Version, VersionReq};
pub use simulation::{ModuleContext, ModulePlan, SimulationModule};
