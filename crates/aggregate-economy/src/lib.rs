//! Pure, deterministic economic plans. The caller owns state validation and commits.

mod command;
mod consumption;
pub mod content;
mod day_work;
mod error;
mod labor;
mod production;
mod program;
mod schedule;
pub use program::EconomyProgram;

pub use consumption::{ConsumptionOutcome, plan_consumption};
pub use error::EconomyError;
pub use labor::{LaborAllocation, LaborRequest, allocate_labor};
pub use production::{ProductionOutcome, plan_production};
