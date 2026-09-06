use super::{MapLabel, MapModeSelect, outliner::MapOutliner};
use crate::{
    screens::action_button,
    state::{InterfaceAction, InterfaceState},
};
use aggregate_ui::{
    button::UiButton,
    components as ui,
    fonts::UiFonts,
    layout::{self, UiPointerBlocker},
    select, theme,
};
use bevy::prelude::*;

pub fn build(commands: &mut Commands, root: Entity, fonts: &UiFonts, state: &InterfaceState) {
    let header = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: px(76),
            padding: UiRect::axes(px(22), px(12)),
            align_items: AlignItems::Center,
            column_gap: px(18),
            ..default()
        },
    );
    commands.entity(header).insert(UiPointerBlocker);
    aggregate_ui::icon::icon(
        commands,
        header,
        aggregate_ui::icon::Icon::Globe,
        30.,
        theme::TEXT,
    );
    let title = layout::column(commands, header, 2.);
    ui::text(
        commands,
        title,
        fonts,
        "AGGREGATE",
        20.,
        theme::ACCENT_BRIGHT,
        true,
    );
    ui::text(
        commands,
        title,
        fonts,
        state.text("map-title"),
        12.,
        theme::MUTED,
        false,
    );
    let totals = ui::text(
        commands,
        header,
        fonts,
        state.text("map-loading"),
        13.,
        theme::TEXT,
        false,
    );
    commands.entity(totals).insert(MapLabel::Status);
    ui::node(
        commands,
        header,
        Node {
            flex_grow: 1.,
            ..default()
        },
    );
    for (key, action, order) in [
        ("settings-language", InterfaceAction::SwitchLanguage, 1),
        ("menu-back", InterfaceAction::Back, 2),
    ] {
        let slot = ui::node(
            commands,
            header,
            Node {
                width: px(112),
                ..default()
            },
        );
        action_button(
            commands,
            slot,
            fonts,
            &state.text(key),
            UiButton::secondary(order),
            action,
        );
    }

    let rail = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(120),
            padding: UiRect::all(px(8)),
            width: px(82),
            flex_direction: FlexDirection::Column,
            row_gap: px(9),
            ..default()
        },
    );
    commands.entity(rail).insert(UiPointerBlocker);
    for (label, action, order) in [
        ("map-nav-management", InterfaceAction::OpenManagement, 3),
        ("map-nav-settings", InterfaceAction::OpenSettings, 4),
    ] {
        let button = action_button(
            commands,
            rail,
            fonts,
            &state.text(label),
            UiButton::secondary(order),
            action,
        );
        commands.entity(button).insert(Node {
            width: percent(100),
            min_height: px(60),
            padding: UiRect::all(px(5)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(22)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        });
        aggregate_ui::icon::icon(
            commands,
            button,
            if action == InterfaceAction::OpenManagement {
                aggregate_ui::icon::Icon::Government
            } else {
                aggregate_ui::icon::Icon::Settings
            },
            22.,
            theme::TEXT,
        );
    }

    super::inspection::build(commands, root);

    let outliner = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            top: px(100),
            bottom: px(150),
            width: px(232),
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
    );
    commands
        .entity(outliner)
        .insert((UiPointerBlocker, super::MapOutlinerPanel));
    ui::text(
        commands,
        outliner,
        fonts,
        state.text("map-outliner-title"),
        16.,
        theme::ACCENT_BRIGHT,
        true,
    );
    ui::rule(commands, outliner);
    let list = layout::scroll_area(
        commands,
        outliner,
        Node {
            width: percent(100),
            flex_grow: 1.,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    commands.entity(list).insert(MapOutliner);

    let dock = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            bottom: px(60),
            left: percent(50),
            padding: UiRect::axes(px(18), px(10)),
            align_items: AlignItems::Center,
            column_gap: px(16),
            ..default()
        },
    );
    commands.entity(dock).insert(UiPointerBlocker);
    ui::text(
        commands,
        dock,
        fonts,
        state.text("map-lens-title"),
        13.,
        theme::ACCENT,
        false,
    );
    let mode = select::root(commands, dock, "terrain", 180.);
    commands.entity(mode).insert(MapModeSelect);
    select::trigger(commands, mode, fonts, &state.text("map-mode-terrain"), 0);
    let content = select::content_at(commands, mode, true);
    for (key, value, order) in [
        ("map-mode-terrain", "terrain", 10),
        ("map-mode-political", "political", 11),
    ] {
        select::item(
            commands,
            content,
            mode,
            fonts,
            &state.text(key),
            value,
            order,
        );
    }

    let hover = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            bottom: px(65),
            left: px(100),
            max_width: percent(40),
            padding: UiRect::axes(px(14), px(8)),
            ..default()
        },
    );
    commands.entity(hover).insert(UiPointerBlocker);
    let text = ui::text(
        commands,
        hover,
        fonts,
        state.text("map-hover-hint"),
        13.,
        theme::TEXT,
        false,
    );
    commands.entity(text).insert(MapLabel::Hover);
}
