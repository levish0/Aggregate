use crate::state::InterfaceState;
use aggregate_ui::{
    button::UiButton, components as ui, fonts::UiFonts, icon::Icon, theme, tooltip::TooltipContent,
};
use bevy::prelude::*;

pub(super) fn icon_button(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    state: &InterfaceState,
    icon: Icon,
    label_key: &str,
    style: UiButton,
) -> Entity {
    let button = ui::button(commands, parent, fonts, "", style);
    let label = state.text(label_key);
    commands.entity(button).insert((
        Name::new(label.clone()),
        Node {
            width: px(36),
            height: px(36),
            min_height: px(36),
            flex_shrink: 0.,
            border: UiRect::all(px(1)),
            padding: UiRect::all(px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        TooltipContent {
            title: label,
            body: String::new(),
            hint: String::new(),
            locking_label: state.text("tooltip-locking"),
            locked_label: state.text("tooltip-locked"),
            links: vec![],
        },
    ));
    aggregate_ui::icon::icon(commands, button, icon, 20., theme::TEXT);
    button
}
