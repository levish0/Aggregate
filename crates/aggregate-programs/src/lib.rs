//! Native extension contracts. No renderer, ECS access, or dynamic-library ABI.
mod manifest;
mod runtime;
mod simulation;

pub use manifest::{PROGRAM_API_VERSION, ProgramDependency, ProgramManifest};
pub use runtime::{ProgramRuntime, PreparedPrograms, SavedProgramState};
pub use semver::{Version, VersionReq};
pub use simulation::{ProgramContext, ProgramPlan, SimulationProgram};
