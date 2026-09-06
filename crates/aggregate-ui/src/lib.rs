//! Native Bevy UI components and their visual theme. No simulation dependency.
pub mod button;
pub mod components;
pub mod fonts;
pub mod layout;
pub mod motion;
pub mod scroll;
pub mod select;
pub mod skin;
pub mod theme;
pub mod tooltip;

use bevy::{
    input_focus::{InputFocus, InputFocusVisible},
    prelude::*,
};
use button::{ButtonActivated, KeyboardFocus};
use tooltip::TooltipState;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum UiSystems {
    Interaction,
}

pub struct AggregateUiPlugin;

impl Plugin for AggregateUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<skin::SurfaceMaterial>::default())
            .init_resource::<InputFocus>()
            .init_resource::<InputFocusVisible>()
            .init_resource::<motion::MotionPreferences>()
            .init_resource::<KeyboardFocus>()
            .init_resource::<button::UiKeyboardPolicy>()
            .init_resource::<TooltipState>()
            .init_resource::<tooltip::TooltipSettings>()
            .init_resource::<select::SelectInteractionState>()
            .add_message::<select::SelectChanged>()
            .add_message::<ButtonActivated>()
            .add_systems(Startup, fonts::load_fonts)
            .add_systems(
                Update,
                (
                    button::keyboard_navigation,
                    button::pointer_interaction,
                    select::interact,
                    button::animate_buttons,
                    tooltip::update_tooltips,
                )
                    .chain()
                    .in_set(UiSystems::Interaction),
            )
            .add_systems(Update, (motion::animate_panels, scroll::scroll_regions))
            .add_systems(
                PostUpdate,
                (
                    select::update_labels,
                    skin::apply_surfaces,
                    skin::animate_surfaces,
                )
                    .chain(),
            );
    }
}
