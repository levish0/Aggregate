use super::action_button;
use crate::state::{InterfaceAction, InterfaceState};
use aggregate_ui::{button::UiButton, components as ui, fonts::UiFonts, theme};
use bevy::prelude::*;

pub fn build(commands: &mut Commands, root: Entity, fonts: &UiFonts, state: &InterfaceState) {
    let shade = ui::node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
    );
    commands
        .entity(shade)
        .insert(BackgroundGradient::from(LinearGradient {
            angle: std::f32::consts::FRAC_PI_2,
            stops: vec![
                theme::INK.with_alpha(0.94).into(),
                theme::INK.with_alpha(0.5).into(),
                Color::NONE.into(),
            ],
            ..default()
        }));
    let column = ui::node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: percent(7),
            top: percent(11),
            width: px(420),
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
    );
    commands.entity(column).insert((
        aggregate_ui::motion::PanelEntrance::new(Vec2::new(18., 0.)),
        UiTransform::default(),
    ));
    let brand = ui::node(
        commands,
        column,
        Node {
            align_items: AlignItems::Center,
            column_gap: px(18),
            ..default()
        },
    );
    ui::seal(commands, brand, 62.);
    let name = ui::node(
        commands,
        brand,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        },
    );
    ui::text(
        commands,
        name,
        fonts,
        "AGGREGATE",
        39.,
        theme::GOLD_BRIGHT,
        true,
    );
    ui::text(
        commands,
        name,
        fonts,
        state.text("app-subtitle"),
        13.,
        theme::MUTED,
        false,
    );
    let panel = ui::panel(
        commands,
        column,
        Node {
            width: percent(100),
            padding: UiRect::all(px(30)),
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
    );
    ui::text(
        commands,
        panel,
        fonts,
        "I  /  AGGREGATE",
        12.,
        theme::GOLD,
        false,
    );
    ui::text(
        commands,
        panel,
        fonts,
        state.text("menu-title"),
        29.,
        theme::TEXT,
        true,
    );
    ui::text(
        commands,
        panel,
        fonts,
        state.text("menu-description"),
        16.,
        theme::MUTED,
        false,
    );
    ui::rule(commands, panel);
    action_button(
        commands,
        panel,
        fonts,
        &state.text("menu-management"),
        UiButton::primary(0),
        InterfaceAction::OpenManagement,
    );
    action_button(
        commands,
        panel,
        fonts,
        &state.text("menu-preview"),
        UiButton::secondary(1),
        InterfaceAction::OpenPreview,
    );
    action_button(
        commands,
        panel,
        fonts,
        &state.text("menu-settings"),
        UiButton::secondary(2),
        InterfaceAction::OpenSettings,
    );
    action_button(
        commands,
        panel,
        fonts,
        &state.text("menu-exit"),
        UiButton::secondary(3),
        InterfaceAction::Exit,
    );
    ui::rule(commands, panel);
    action_button(
        commands,
        panel,
        fonts,
        "한국어  /  ENGLISH",
        UiButton::secondary(4),
        InterfaceAction::SwitchLanguage,
    );
    ui::text(
        commands,
        column,
        fonts,
        state.text("menu-footnote"),
        13.,
        theme::MUTED,
        false,
    );

    let cartouche = ui::node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: percent(60),
            top: percent(32),
            width: px(350),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(15),
            ..default()
        },
    );
    ui::text(
        commands,
        cartouche,
        fonts,
        "T E R R A   I N C O G N I T A",
        18.,
        theme::TEXT.with_alpha(0.6),
        false,
    );
    ui::rule(commands, cartouche);
    ui::text(
        commands,
        cartouche,
        fonts,
        "A T L A S   /   0 0 1",
        12.,
        theme::GOLD.with_alpha(0.7),
        false,
    );
    let compass = ui::node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            right: percent(12),
            bottom: percent(16),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(9),
            ..default()
        },
    );
    ui::text(commands, compass, fonts, "N", 16., theme::GOLD, true);
    ui::seal(commands, compass, 90.);
    ui::text(
        commands,
        compass,
        fonts,
        state.text("map-caption"),
        11.,
        theme::MUTED,
        false,
    );
}
