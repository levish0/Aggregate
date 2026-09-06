//! Pure, deterministic economic plans. The caller owns state validation and commits.

mod consumption;
mod error;
mod labor;
mod production;
mod command;
mod day_work;
mod schedule;
mod program;
pub mod content;
pub use program::EconomyProgram;

pub use consumption::{ConsumptionOutcome, plan_consumption};
pub use error::EconomyError;
pub use labor::{LaborAllocation, LaborRequest, allocate_labor};
pub use production::{ProductionOutcome, plan_production};
