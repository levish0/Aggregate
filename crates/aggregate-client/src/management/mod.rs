mod actions;
#[cfg(test)]
#[path = "../../tests/management/background.rs"]
mod background_tests;
pub mod construction;
mod playback;
pub mod presentation;
mod session;
#[cfg(test)]
#[path = "../../tests/management/flows.rs"]
mod tests;
mod worker;

pub use actions::{ManagementAction, apply_management_actions};
pub use playback::{
    SimulationSpeed, advance_running_session, speed_controls, update_speed_controls,
};
pub use session::{ManagementSession, ManagementViewState, SessionFeedback};
pub use session::{NotificationEntry, poll_simulation};
