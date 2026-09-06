mod actions;
mod layout;
pub use actions::{apply_actions, finish_initialization};
use aggregate_world::CountryId;
use aggregate_world_generation::WorldInitializationSettings;
use bevy::{prelude::*, tasks::Task};
pub use layout::rebuild;

#[derive(Resource, Default)]
pub struct WorldSetup {
    pub open: bool,
    pub country: Option<CountryId>,
    pub settings: WorldInitializationSettings,
    pub public_health: bool,
    pub error: Option<String>,
    pub revision: u64,
}

#[derive(Resource)]
pub struct WorldInitializationTask(pub Task<Result<crate::management::ManagementSession, String>>);

#[derive(Component)]
pub struct SetupRoot;
#[derive(Component)]
pub enum SetupSelect {
    Country,
    Population,
    Workforce,
    Reserves,
}
#[derive(Component)]
pub enum SetupAction {
    Close,
    Start,
    TogglePublicHealth,
}
