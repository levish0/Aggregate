//! Opt-in native acceptance for the actual geographic simulation and a large country.
use crate::{
    management::{ManagementAction, ManagementSession},
    screens::world_map::inspection::{InspectionAction, InspectionRoot, InspectionTab},
    state::{InterfaceAction, InterfaceState, Screen},
    world_setup::{SetupAction, WorldSetup},
};
use aggregate_map_view::{LoadedWorldMap, MapCameraController, MapViewState};
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
    building_site: Option<aggregate_world::ProvinceId>,
    building_button: Option<Entity>,
    camera_target: Vec3,
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
            building_site: None,
            building_button: None,
            camera_target: Vec3::ZERO,
        })
        .add_systems(Update, drive.before(aggregate_ui::UiSystems::Interaction));
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
    interface: Res<InterfaceState>,
    inspection_buttons: Query<(Entity, &InspectionAction)>,
    panels: Query<&ComputedNode, With<InspectionRoot>>,
    camera: Res<MapCameraController>,
    input: (
        Single<&mut Window>,
        ResMut<ButtonInput<MouseButton>>,
        ResMut<bevy::input::mouse::AccumulatedMouseMotion>,
        ResMut<ButtonInput<KeyCode>>,
    ),
) {
    let (mut window, mut mouse, mut motion, mut keys) = input;
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
            assert_eq!(interface.screen, Screen::WorldMap);
            assert!(map.enabled && map.inspect_country);
            assert!(panels.single().unwrap().size().x < window.physical_width() as f32 * 0.40);
            let (entity, _) = inspection_buttons
                .iter()
                .find(|(_, action)| {
                    matches!(action, InspectionAction::Tab(InspectionTab::Buildings))
                })
                .unwrap();
            activated.write(ButtonActivated(entity));
            capture.phase = 5;
            capture.frames = 0;
        }
        5 if capture.frames >= 10 => {
            // Select a real owned state, then build from its card without leaving the map.
            let (entity, _) = inspection_buttons
                .iter()
                .find(|(_, action)| matches!(action, InspectionAction::State))
                .unwrap();
            activated.write(ButtonActivated(entity));
            capture.phase = 6;
            capture.frames = 0;
        }
        6 if capture.frames >= 10 => {
            let (entity, _) = inspection_buttons
                .iter()
                .find(|(_, action)| {
                    matches!(action, InspectionAction::Tab(InspectionTab::Buildings))
                })
                .unwrap();
            activated.write(ButtonActivated(entity));
            capture.phase = 7;
            capture.frames = 0;
        }
        7 if capture.frames >= 10 => {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/screenshots/map-state-buildings-russia.png");
            commands.spawn(Screenshot::primary_window()).observe(
                move |event: On<ScreenshotCaptured>| {
                    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                    event
                        .image
                        .clone()
                        .try_into_dynamic()
                        .unwrap()
                        .to_rgb8()
                        .save(&path)
                        .unwrap();
                },
            );
            let (entity, action) = management_buttons
                .iter()
                .find(|(_, action)| matches!(action, ManagementAction::StartConstructionAt { .. }))
                .unwrap();
            if let ManagementAction::StartConstructionAt { province, .. } = action {
                capture.building_site = Some(province.clone());
            }
            capture.building_button = Some(entity);
            for _ in 0..3 { activated.write(ButtonActivated(entity)); }
            capture.phase = 8;
            capture.frames = 0;
        }
        8 if capture.frames >= 10 && !session.is_busy() => {
            assert_eq!(interface.screen, Screen::WorldMap);
            assert_eq!(session.snapshot.construction_projects.iter().filter(|project| Some(&project.province) == capture.building_site.as_ref()).count(), 3);
            assert!(management_buttons.contains(capture.building_button.unwrap()), "construction updates must retain the existing card controls");
            keys.press(KeyCode::KeyB);
            capture.phase = 9;
            capture.frames = 0;
        }
        9 if capture.frames >= 10 => {
            keys.release(KeyCode::KeyB);
            assert!(inspection_buttons.iter().any(|(_, action)| matches!(action, InspectionAction::Tab(InspectionTab::Construction))));
            let cursor = Vec2::new(window.width() * 0.60, window.height() * 0.55);
            window.set_cursor_position(Some(cursor));
            capture.camera_target = camera.target;
            mouse.press(MouseButton::Middle);
            motion.delta = Vec2::new(80., 20.);
            capture.phase = 10;
            capture.frames = 0;
        }
        10 if capture.frames >= 2 => {
            mouse.release(MouseButton::Middle);
            assert!(camera.target.distance(capture.camera_target) > 1.);
            assert_eq!(interface.screen, Screen::WorldMap);
            println!(
                "LARGE_WORLD provinces={} frames_during_day={} map_construction_and_pan=passed",
                session.snapshot.provinces.len(),
                capture.busy_frames
            );
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/screenshots/map-construction-russia.png");
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
            capture.phase = 11;
        }
        11 if capture.captured => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
