//! Opt-in native acceptance for the actual geographic simulation and a large country.
use crate::{
    management::{ManagementAction, ManagementSession},
    state::InterfaceAction,
    world_setup::{SetupAction, WorldSetup},
};
use aggregate_map_view::{LoadedWorldMap, MapViewState};
use aggregate_ui::button::ButtonActivated;
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};

#[derive(Resource)]
struct LargeWorldCapture {
    phase: u8,
    frames: u32,
    busy_frames: u32,
    captured: bool,
    started: std::time::Instant,
}

#[test]
#[ignore = "opens a native window and loads the full local geography"]
fn native_large_world_capture() {
    let mut app = crate::create_app();
    app.insert_resource(bevy::winit::WinitSettings::continuous())
        .insert_resource(LargeWorldCapture {
            phase: 0,
            frames: 0,
            busy_frames: 0,
            captured: false,
            started: std::time::Instant::now(),
        })
        .add_systems(
            Update,
            drive.after(crate::screens::management::update_management_labels),
        );
    assert_eq!(app.run(), AppExit::Success);
}

fn drive(
    mut commands: Commands,
    mut capture: ResMut<LargeWorldCapture>,
    session: Res<ManagementSession>,
    loaded: Option<Res<LoadedWorldMap>>,
    map: Res<MapViewState>,
    mut setup: ResMut<WorldSetup>,
    interface_buttons: Query<(Entity, &InterfaceAction)>,
    setup_buttons: Query<(Entity, &SetupAction)>,
    management_buttons: Query<(Entity, &ManagementAction)>,
    mut activated: MessageWriter<ButtonActivated>,
    mut exit: MessageWriter<AppExit>,
) {
    capture.frames += 1;
    assert!(
        capture.started.elapsed().as_secs() < 120,
        "large world capture timed out"
    );
    match capture.phase {
        0 if capture.frames >= 35 => {
            let (entity, _) = interface_buttons
                .iter()
                .find(|(_, action)| **action == InterfaceAction::ConfigureWorld)
                .unwrap();
            activated.write(ButtonActivated(entity));
            capture.phase = 1;
        }
        1 if setup.open && !map.loading => {
            if let Some(loaded) = loaded
                && let Some(country) = loaded
                    .0
                    .catalog
                    .countries
                    .iter()
                    .find(|country| country.key == "RUS")
                && let Some((entity, _)) = setup_buttons
                    .iter()
                    .find(|(_, action)| matches!(action, SetupAction::Start))
            {
                setup.country = Some(country.id.clone());
                setup.revision += 1;
                activated.write(ButtonActivated(entity));
                capture.phase = 2;
            }
        }
        2 if session.geographic && !setup.open => {
            assert!(session.snapshot.provinces.len() > 30_000);
            let (entity, _) = management_buttons
                .iter()
                .find(|(_, action)| **action == ManagementAction::StepDay)
                .unwrap();
            activated.write(ButtonActivated(entity));
            capture.phase = 3;
        }
        3 => {
            if session.is_busy() {
                capture.busy_frames += 1;
            }
            if session.snapshot.day == 1 && !session.is_busy() {
                assert!(
                    capture.busy_frames > 0,
                    "render updates must continue during the full-world calculation"
                );
                let (entity, _) = interface_buttons
                    .iter()
                    .find(|(_, action)| **action == InterfaceAction::OpenManagement)
                    .unwrap();
                activated.write(ButtonActivated(entity));
                capture.phase = 4;
                capture.frames = 0;
            }
        }
        4 if capture.frames >= 25 => {
            let count = management_buttons
                .iter()
                .filter(|(_, action)| matches!(action, ManagementAction::SelectProvince(_)))
                .count();
            assert!(
                count > 0 && count <= 32,
                "large countries must not create thousands of navigation rows"
            );
            println!(
                "LARGE_WORLD provinces={} frames_during_day={} visible_province_rows={count}",
                session.snapshot.provinces.len(),
                capture.busy_frames
            );
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/screenshots/management-russia-performance.png");
            commands.spawn(Screenshot::primary_window()).observe(
                move |event: On<ScreenshotCaptured>, mut capture: ResMut<LargeWorldCapture>| {
                    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                    event
                        .image
                        .clone()
                        .try_into_dynamic()
                        .unwrap()
                        .to_rgb8()
                        .save(&path)
                        .unwrap();
                    capture.captured = true;
                },
            );
            capture.phase = 5;
        }
        5 if capture.captured => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
