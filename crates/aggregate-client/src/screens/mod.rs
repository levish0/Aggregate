mod main_menu;
pub mod management;
mod preview;
mod settings;
pub mod world_map;

use crate::state::{InterfaceAction, InterfaceState, Screen};
use aggregate_ui::{button::UiButton, components, fonts::UiFonts, theme};
use bevy::prelude::*;

#[derive(Component)]
pub struct ScreenRoot;

#[derive(Component)]
pub struct StatusLabel;

#[derive(Component)]
pub struct ScaleLabel;

pub fn rebuild(
    mut commands: Commands,
    state: Res<InterfaceState>,
    fonts: Res<UiFonts>,
    session: Res<crate::management::ManagementSession>,
    roots: Query<Entity, With<ScreenRoot>>,
    mut previous: Local<
        Option<(
            Screen,
            crate::state::PreviewTab,
            aggregate_localization::Language,
        )>,
    >,
) {
    let view = (state.screen, state.tab, state.localization.language());
    if *previous == Some(view) && !fonts.is_changed() {
        return;
    }
    *previous = Some(view);
    for root in &roots {
        commands.entity(root).despawn();
    }
    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            ScreenRoot,
        ))
        .id();
    match state.screen {
        Screen::MainMenu => main_menu::build(&mut commands, root, &fonts, &state),
        Screen::Management => management::build(&mut commands, root, &fonts, &state, &session),
        Screen::WorldMap => world_map::build(&mut commands, root, &fonts, &state),
        Screen::Preview => preview::build(&mut commands, root, &fonts, &state),
        Screen::Settings => settings::build(&mut commands, root, &fonts, &state),
    }
    footer(&mut commands, root, &fonts, &state);
}

pub fn action_button(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    label: &str,
    button: UiButton,
    action: InterfaceAction,
) -> Entity {
    let entity = components::button(commands, parent, fonts, label, button);
    commands.entity(entity).insert(action);
    entity
}

fn footer(commands: &mut Commands, root: Entity, fonts: &UiFonts, state: &InterfaceState) {
    let row = components::node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            bottom: px(0),
            width: percent(100),
            min_height: px(42),
            padding: UiRect::axes(px(30), px(10)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
    );
    commands.entity(row).insert((
        BackgroundColor(theme::INK.with_alpha(0.92)),
        aggregate_ui::layout::UiPointerBlocker,
    ));
    components::text(
        commands,
        row,
        fonts,
        state.text("foundation-label"),
        12.,
        theme::GOLD,
        false,
    );
    components::text(
        commands,
        row,
        fonts,
        state.text(if state.screen == Screen::WorldMap {
            "map-keyboard"
        } else {
            "status-keyboard"
        }),
        12.,
        theme::MUTED,
        false,
    );
}
