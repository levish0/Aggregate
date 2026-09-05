mod backdrop;
mod interaction;
mod screens;
mod state;

use aggregate_ui::{AggregateUiPlugin, UiSystems};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    window::WindowResolution,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(aggregate_ui::theme::INK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Aggregate — Interface Foundation".into(),
                resolution: WindowResolution::new(1440, 900),
                resize_constraints: WindowResizeConstraints {
                    min_width: 960.,
                    min_height: 640.,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(AggregateUiPlugin)
        .init_resource::<state::InterfaceState>()
        .add_systems(Startup, (setup_camera, backdrop::setup))
        .add_systems(
            Update,
            (
                interaction::apply_actions.after(UiSystems::Interaction),
                interaction::responsive_scale.after(interaction::apply_actions),
                screens::rebuild.after(interaction::apply_actions),
                interaction::update_live_labels.after(screens::rebuild),
                backdrop::resize,
                take_screenshot,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn take_screenshot(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F12) {
        let directory =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screenshots");
        if let Err(error) = std::fs::create_dir_all(&directory) {
            error!("screenshot directory: {error}");
            return;
        }
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(
                directory.join(format!("aggregate-{timestamp}.png")),
            ));
    }
}
