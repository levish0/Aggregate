mod backdrop;
mod diagnostics;
mod interaction;
mod management;
#[cfg(all(test, target_os = "windows"))]
mod native_capture;
#[cfg(all(test, target_os = "windows"))]
mod native_map_capture;
mod screens;
mod state;

use aggregate_ui::{AggregateUiPlugin, UiSystems};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    window::WindowResolution,
};

fn main() {
    create_app().run();
}

fn create_app() -> App {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let mut app = App::new();
    app.insert_resource(ClearColor(aggregate_ui::theme::INK))
        .add_plugins(
            DefaultPlugins
                .set(diagnostics::log_plugin())
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(bevy::winit::WinitPlugin {
                    run_on_any_thread: cfg!(test),
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Aggregate — Nation Management".into(),
                        resolution: WindowResolution::new(1440, 900),
                        resize_constraints: WindowResizeConstraints {
                            min_width: 960.,
                            min_height: 640.,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                }),
        );
    let session = management::ManagementSession::foundation();
    let view = session.initial_view();
    app.add_plugins(AggregateUiPlugin)
        .add_plugins(aggregate_map_view::MapViewPlugin { asset_root })
        .init_resource::<state::InterfaceState>()
        .insert_resource(session)
        .insert_resource(view)
        .add_systems(
            Startup,
            (setup_camera, backdrop::setup, diagnostics::log_startup),
        )
        .add_systems(
            Update,
            (
                (
                    interaction::apply_actions,
                    management::apply_management_actions,
                    management::advance_running_session,
                    screens::rebuild,
                    screens::world_map::configure_view,
                    screens::world_map::outliner::apply_jumps,
                    screens::world_map::outliner::rebuild,
                    screens::world_map::update_labels,
                    screens::management::update_management_lists,
                    screens::management::update_management_labels,
                    interaction::update_live_labels,
                )
                    .chain()
                    .after(UiSystems::Interaction)
                    .before(aggregate_map_view::MapViewSystems),
                interaction::responsive_scale.after(interaction::apply_actions),
                screens::world_map::configure_keyboard_policy.before(UiSystems::Interaction),
                backdrop::resize,
                take_screenshot,
            ),
        );
    app
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        IsDefaultUiCamera,
    ));
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
