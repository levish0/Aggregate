use super::action_button;
use crate::state::{InterfaceAction, InterfaceState};
use aggregate_ui::{button::UiButton, components as ui, fonts::UiFonts, theme};
use bevy::prelude::*;

pub fn build(commands: &mut Commands, root: Entity, fonts: &UiFonts, state: &InterfaceState) {
    let container = ui::node(
        commands,
        root,
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(50)),
            ..default()
        },
    );
    commands
        .entity(container)
        .insert(BackgroundColor(theme::INK.with_alpha(0.55)));
    let panel = ui::panel(
        commands,
        container,
        Node {
            width: px(560),
            max_width: percent(100),
            padding: UiRect::all(px(32)),
            flex_direction: FlexDirection::Column,
            row_gap: px(16),
            ..default()
        },
    );
    commands.entity(panel).insert((
        aggregate_ui::motion::PanelEntrance::new(Vec2::new(0., 18.)),
        UiTransform::default(),
    ));
    ui::text(commands, panel, fonts, "AGGREGATE", 13., theme::ACCENT, true);
    ui::text(
        commands,
        panel,
        fonts,
        state.text("nav-settings"),
        32.,
        theme::TEXT,
        true,
    );
    ui::text(
        commands,
        panel,
        fonts,
        state.text("settings-description"),
        15.,
        theme::MUTED,
        false,
    );
    ui::rule(commands, panel);
    ui::text(
        commands,
        panel,
        fonts,
        state.text("settings-language"),
        16.,
        theme::ACCENT,
        false,
    );
    action_button(
        commands,
        panel,
        fonts,
        "한국어  /  ENGLISH",
        UiButton::primary(0),
        InterfaceAction::SwitchLanguage,
    );
    let scale_label = ui::text(
        commands,
        panel,
        fonts,
        format!(
            "{}  ·  {:.0}%",
            state.text("settings-scale"),
            state.scale * 100.
        ),
        16.,
        theme::ACCENT,
        false,
    );
    commands.entity(scale_label).insert(super::ScaleLabel);
    let row = ui::node(
        commands,
        panel,
        Node {
            column_gap: px(12),
            ..default()
        },
    );
    action_button(
        commands,
        row,
        fonts,
        &state.text("settings-smaller"),
        UiButton::secondary(1),
        InterfaceAction::ScaleDown,
    );
    action_button(
        commands,
        row,
        fonts,
        &state.text("settings-larger"),
        UiButton::secondary(2),
        InterfaceAction::ScaleUp,
    );
    action_button(
        commands,
        panel,
        fonts,
        &state.text("settings-reset"),
        UiButton::secondary(3),
        InterfaceAction::ResetScale,
    );
    action_button(
        commands,
        panel,
        fonts,
        &state.text(if state.reduced_motion {
            "settings-motion-reduced"
        } else {
            "settings-motion-full"
        }),
        UiButton::secondary(4),
        InterfaceAction::ToggleMotion,
    );
    ui::rule(commands, panel);
    action_button(
        commands,
        panel,
        fonts,
        &state.text("menu-back"),
        UiButton::secondary(5),
        InterfaceAction::Back,
    );
}
