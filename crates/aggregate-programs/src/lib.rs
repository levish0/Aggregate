//! Native extension contracts. No renderer, ECS access, or dynamic-library ABI.
mod inspection;
mod manifest;
pub use inspection::{InspectionMetric, InspectionScope, InspectionSection, MetricValue};
mod runtime;
mod simulation;

pub use manifest::{PROGRAM_API_VERSION, ProgramDependency, ProgramManifest};
pub use runtime::{PreparedPrograms, ProgramRuntime, SavedProgramState};
pub use semver::{Version, VersionReq};
pub use simulation::{ProgramContext, ProgramPlan, SimulationProgram};
