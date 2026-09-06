//! One authoritative world, program orchestration, coordinated ticks and portable saves.
mod diagnostics;
mod save;
mod simulation;

pub use aggregate_programs::{
    RecordedCommand, SimulationClock, SimulationCommand, SimulationError,
    CommandOutcome, ConstructionDayReport, DayReport, FacilityDayReport, GoodsFlow, GoodsFlowCause,
    ProvinceDayReport, SimulationEvent,
};
pub use save::{RULESET_VERSION, SAVE_SCHEMA_VERSION};
pub use simulation::Simulation;
