//! Explicit sandbox initialization on persistent geographic identities.
//! Parameters are authored starting conditions, not demographic observations.
mod initialization;
mod content;
mod settings;
pub use initialization::initialize_world;
pub use settings::WorldInitializationSettings;
