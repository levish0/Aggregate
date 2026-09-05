//! Pure, deterministic economic plans. The caller owns state validation and commits.

mod consumption;
mod error;
mod labor;
mod production;

pub use consumption::{ConsumptionOutcome, plan_consumption};
pub use error::EconomyError;
pub use labor::{LaborAllocation, LaborRequest, allocate_labor};
pub use production::{ProductionOutcome, plan_production};
