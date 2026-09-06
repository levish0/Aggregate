use super::{MapLabel, MapModeSelect, controls::icon_button, outliner::MapOutliner};
use crate::state::{InterfaceAction, InterfaceState};
use aggregate_ui::icon::Icon;
use aggregate_ui::{
    button::UiButton,
    components as ui,
    fonts::UiFonts,
    layout::{self, UiPointerBlocker},
    select, theme,
};
use bevy::prelude::*;

pub fn build(
    commands: &mut Commands,
    root: Entity,
    fonts: &UiFonts,
    state: &InterfaceState,
    session: &crate::management::ManagementSession,
) {
    if !session.geographic { return; }
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
    crate::country_presentation::flag(commands, header, session.player_country.clone(), 38.);
    let country = session
        .snapshot
        .countries
        .iter()
        .find(|country| country.id == session.player_country);
    let country_name = country
        .map(|country| {
            country
                .name_key
                .as_ref()
                .and_then(|key| state.localization.text(key).ok())
                .unwrap_or_else(|| country.name.clone())
        })
        .unwrap_or_default();
    let title = layout::column(commands, header, 2.);
    ui::text(
        commands,
        title,
        fonts,
        country_name,
        20.,
        theme::ACCENT_BRIGHT,
        true,
    );
    ui::text(
        commands,
        title,
        fonts,
        state.text("map-player-country"),
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
    let population = layout::row(commands, header, 7.);
    aggregate_ui::icon::icon(commands, population, Icon::Population, 20., theme::TEXT);
    let value = ui::text(commands, population, fonts, "—", 18., theme::TEXT, true);
    commands.entity(value).insert(MapLabel::Population);
    ui::node(
        commands,
        header,
        Node {
            flex_grow: 1.,
            ..default()
        },
    );

    let time_column = layout::column(commands, header, 3.);
    let date = ui::text(commands, time_column, fonts, "", 14., theme::TEXT, true);
    commands.entity(date).insert(MapLabel::Day);
    let clock = layout::row(commands, time_column, 4.);
    for (icon, key, action, order) in [
        (
            Icon::Play,
            "management-step",
            crate::management::ManagementAction::StepDay,
            10,
        ),
        (
            Icon::Play,
            "management-play",
            crate::management::ManagementAction::ToggleRunning,
            11,
        ),
    ] {
        let mut style = UiButton::secondary(order);
        style.enabled = session.geographic;
        let button = icon_button(commands, clock, fonts, state, icon, key, style);
        let is_step = matches!(action, crate::management::ManagementAction::StepDay);
        commands.entity(button).insert(action);
        if is_step {
            let bar = ui::node(
                commands,
                button,
                Node {
                    width: px(2),
                    height: px(14),
                    flex_shrink: 0.,
                    ..default()
                },
            );
            commands.entity(bar).insert(BackgroundColor(theme::TEXT));
        }
    }
    crate::management::speed_controls(commands, clock, fonts, session, session.geographic, 20);
    for (icon, key, action, order) in [
        (
            Icon::Globe,
            "settings-language",
            InterfaceAction::SwitchLanguage,
            1,
        ),
        (Icon::Close, "menu-back", InterfaceAction::Back, 2),
    ] {
        let button = icon_button(
            commands,
            header,
            fonts,
            state,
            icon,
            key,
            UiButton::secondary(order),
        );
        commands.entity(button).insert(action);
    }

    let rail = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(84),
            padding: UiRect::all(px(7)),
            width: px(52),
            flex_direction: FlexDirection::Column,
            row_gap: px(9),
            ..default()
        },
    );
    commands.entity(rail).insert(UiPointerBlocker);

    for (icon, label, action, order) in [
        (
            Icon::Government,
            "map-nav-management",
            InterfaceAction::OpenManagement,
            3,
        ),
        (
            Icon::Settings,
            "map-nav-settings",
            InterfaceAction::OpenSettings,
            4,
        ),
    ] {
        let button = icon_button(
            commands,
            rail,
            fonts,
            state,
            icon,
            label,
            UiButton::secondary(order),
        );
        commands.entity(button).insert(action);
    }
    let queue = icon_button(
        commands,
        rail,
        fonts,
        state,
        Icon::Queue,
        "inspection-construction",
        UiButton::secondary(5),
    );
    commands
        .entity(queue)
        .insert(super::inspection::InspectionAction::Construction);
    super::inspection::build(commands, root);

    let outliner = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            top: px(100),
            bottom: px(370),
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
    let news = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            bottom: px(140),
            height: px(214),
            width: px(232),
            padding: UiRect::all(px(12)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
    );
    commands.entity(news).insert(UiPointerBlocker);
    ui::text(
        commands,
        news,
        fonts,
        state.text("map-news-title"),
        16.,
        theme::TEXT,
        true,
    );
    let feed = layout::scroll_area(
        commands,
        news,
        Node {
            width: percent(100),
            min_height: px(0),
            flex_grow: 1.,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    commands.entity(feed).insert(super::news::NewsFeed);
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
