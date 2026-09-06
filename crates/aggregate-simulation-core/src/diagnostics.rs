//! Publish domain facts only after their authoritative transaction succeeds.
use crate::SimulationEvent;

pub(crate) fn log_committed_event(day: u64, event: &SimulationEvent) {
    match event {
        SimulationEvent::ConstructionStarted { province, facility } => {
            tracing::info!(day, event_kind = "construction_started", %province, %facility, "Construction started");
        }
        SimulationEvent::ConstructionCompleted { province, facility } => {
            tracing::info!(day, event_kind = "construction_completed", %province, %facility, "Construction completed");
        }
        SimulationEvent::FoodShortfall {
            province,
            good,
            amount,
        } => {
            // A modeled shortage is a domain outcome, not an engine warning.
            tracing::info!(day, event_kind = "food_shortfall", %province, %good, amount, "Household food shortfall");
        }
    }
}
