mod actions;
mod components;
mod buildings;
mod content;
mod layout;
pub use actions::apply_actions;
use bevy::prelude::*;
pub use content::refresh;
pub use layout::build;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectionTab {
    #[default]
    Overview,
    Buildings,
    Population,
    Territory,
    Programs,
}

#[derive(Resource, Default)]
pub struct InspectionView {
    pub tab: InspectionTab,
}

#[derive(Component)]
pub struct InspectionRoot;
#[derive(Component)]
pub struct InspectionContent;
#[derive(Component, Clone)]
pub enum InspectionAction {
    Tab(InspectionTab),
    Country,
    State,
    Province(u32),
    Close,
}
