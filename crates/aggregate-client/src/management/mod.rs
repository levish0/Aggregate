mod actions;
pub mod construction;
pub mod presentation;
mod session;
mod playback;
mod snapshot_index;
mod worker;
#[cfg(test)]
#[path = "../../tests/management/flows.rs"]
mod tests;

pub use actions::{ManagementAction, apply_management_actions};
pub use playback::{SimulationSpeed, advance_running_session, speed_controls, update_speed_controls};
pub use session::{ManagementSession, ManagementViewState, SessionFeedback};
pub use session::poll_simulation;
