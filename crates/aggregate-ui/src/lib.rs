//! Native Bevy UI components and their visual theme. No simulation dependency.
pub mod button;
pub mod components;
pub mod fonts;
pub mod motion;
pub mod scroll;
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
        app.init_resource::<InputFocus>()
            .init_resource::<InputFocusVisible>()
            .init_resource::<motion::MotionPreferences>()
            .init_resource::<KeyboardFocus>()
            .init_resource::<TooltipState>()
            .init_resource::<tooltip::TooltipSettings>()
            .add_message::<ButtonActivated>()
            .add_systems(Startup, fonts::load_fonts)
            .add_systems(
                Update,
                (
                    button::keyboard_navigation,
                    button::pointer_interaction,
                    button::animate_buttons,
                    tooltip::update_tooltips,
                )
                    .chain()
                    .in_set(UiSystems::Interaction),
            )
            .add_systems(Update, (motion::animate_panels, scroll::scroll_regions));
    }
}
