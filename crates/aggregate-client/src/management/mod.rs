mod actions;
pub mod presentation;
mod session;
#[cfg(test)]
mod tests;

pub use actions::{ManagementAction, advance_running_session, apply_management_actions};
pub use session::{ManagementSession, ManagementViewState, SessionFeedback};
