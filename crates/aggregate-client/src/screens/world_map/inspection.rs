mod actions;
mod buildings;
mod production_summary;
mod components;
mod construction_queue;
mod content;
mod layout;
pub use actions::apply_actions;
pub use buildings::update as update_buildings;
pub use construction_queue::update as update_queue;
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
    NationalTab(InspectionTab),
    State,
    Province(u32),
    Close,
}
