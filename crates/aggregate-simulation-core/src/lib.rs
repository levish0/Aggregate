//! Headless ECS ownership and explicit stepping. No economic model or renderer.
mod clock;
mod simulation;

pub use clock::{SimulationClock, TickOverflow};
pub use simulation::Simulation;
