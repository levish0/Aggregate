mod actions;
pub mod presentation;
mod session;
#[cfg(test)]
#[path = "../../tests/management/flows.rs"]
mod tests;

pub use actions::{ManagementAction, advance_running_session, apply_management_actions};
pub use session::{ManagementSession, ManagementViewState, SessionFeedback};
