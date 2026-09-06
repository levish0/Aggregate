//! One authoritative world, program orchestration, coordinated ticks and portable saves.
mod diagnostics;
mod save;
mod simulation;

pub use aggregate_programs::{
    CommandOutcome, ConstructionDayReport, DayReport, FacilityDayReport, GoodsFlow, GoodsFlowCause,
    ProvinceDayReport, RecordedCommand, SimulationClock, SimulationCommand, SimulationError,
    SimulationEvent,
};
pub use save::{RULESET_VERSION, SAVE_SCHEMA_VERSION};
pub use simulation::Simulation;
