//! Native Rust simulation of shared labor, production, consumption and construction.
//! One authoritative ECS World, staged daily commits, validated commands and portable saves.
mod clock;
mod command;
mod diagnostics;
mod error;
mod report;
mod save;
mod schedule;
mod simulation;
mod world_storage;

pub use clock::SimulationClock;
pub use command::{RecordedCommand, SimulationCommand};
pub use error::SimulationError;
pub use report::{
    CommandOutcome, ConstructionDayReport, DayReport, FacilityDayReport, GoodsFlow, GoodsFlowCause,
    ProvinceDayReport, SimulationEvent,
};
pub use save::{RULESET_VERSION, SAVE_SCHEMA_VERSION};
pub use schedule::SimulationPhase;
pub use simulation::Simulation;
