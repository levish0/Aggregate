mod actions;
mod buildings;
mod components;
mod construction_queue;
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
    Construction,
    Population,
    Territory,
    Programs,
}

#[derive(Resource, Default)]
pub struct InspectionView {
    pub tab: InspectionTab,
    pub page: usize,
}

#[derive(Component)]
pub struct InspectionRoot;
#[derive(Component)]
pub struct InspectionContent;
#[derive(Component, Clone)]
pub enum InspectionAction {
    Tab(InspectionTab),
    Page(usize),
    Country,
    Construction,
    State,
    Province(u32),
    Close,
}
