mod bindings;
mod layout;
mod lists;

pub use bindings::update_management_labels;
pub use layout::build;
pub use lists::update_management_lists;

use aggregate_world::{FacilityDefinitionId, FacilityId, GoodId};
use bevy::prelude::*;

#[derive(Component)]
pub enum ManagementLabel {
    Country,
    Day,
    Population,
    Workforce,
    Province,
    Allocation,
    ReportDay,
    FoodShortfall,
    Feedback,
    Stock(GoodId),
    StockChange(GoodId),
    FacilityStaffing(FacilityDefinitionId),
    ProjectProgress(FacilityId),
}

#[derive(Component)]
pub struct ProjectProgressFill(pub FacilityId);

#[derive(Component)]
pub enum ManagementList {
    Projects,
    News,
}
