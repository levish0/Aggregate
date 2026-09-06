//! Native game-program contracts, shared world protocol and transactional execution.
//! Programs are trusted Rust implementations; no renderer or dynamic-library ABI.
mod clock;
mod command;
mod error;
mod execution;
mod report;
pub mod world_storage;
pub use clock::SimulationClock;
pub use command::{RecordedCommand, SimulationCommand};
pub use error::SimulationError;
pub use execution::ProgramExecution;
pub use report::*;
mod inspection;
mod manifest;
pub use inspection::{InspectionMetric, InspectionScope, InspectionSection, MetricValue};
mod runtime;
mod simulation;

pub use manifest::{PROGRAM_API_VERSION, ProgramDependency, ProgramManifest};
pub use runtime::{PreparedPrograms, ProgramRuntime, SavedProgramState};
pub use semver::{Version, VersionReq};
pub use simulation::{ProgramContext, ProgramPlan, SimulationProgram};
