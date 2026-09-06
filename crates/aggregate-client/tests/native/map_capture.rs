//! Real window/render acceptance; input is injected into Bevy, not sent to the OS.
use crate::state::{InterfaceAction, InterfaceState, Screen};
use aggregate_map_view::{LoadedWorldMap, MapCamera, MapCameraController, MapViewState};
use aggregate_ui::{
    button::ButtonActivated,
    select::{Select, SelectItem, SelectTrigger},
};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};

#[derive(Resource, Default)]
struct MapCapture {
    frame: u32,
    ready_frames: u32,
    captures: u32,
    expected_index: u32,
    initial_target: Vec3,
    initial_pitch: f32,
    initial_yaw: f32,
}

#[test]
#[ignore = "opens a native window and requires the local map assets and a graphics adapter"]
fn native_map_capture() {
    std::fs::create_dir_all(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screenshots"),
    )
    .unwrap();
    let mut app = crate::create_app();
    app.insert_resource(bevy::winit::WinitSettings::continuous())
        .init_resource::<MapCapture>()
        .add_systems(
            Update,
            drive_capture.before(aggregate_ui::UiSystems::Interaction),
        );
    assert_eq!(app.run(), AppExit::Success);
}

fn drive_capture(
    mut commands: Commands,
    mut capture: ResMut<MapCapture>,
    map: Option<Res<LoadedWorldMap>>,
    state: Res<MapViewState>,
    mut interface: ResMut<InterfaceState>,
    mut window: Single<&mut Window>,
    camera: Single<(&Camera, &GlobalTransform), With<MapCamera>>,
    buttons: Query<(Entity, &InterfaceAction)>,
    triggers: Query<Entity, With<SelectTrigger>>,
    items: Query<(Entity, &SelectItem)>,
    selects: Query<&Select>,
    mut activated: MessageWriter<ButtonActivated>,
    input: (
        ResMut<ButtonInput<MouseButton>>,
        ResMut<ButtonInput<KeyCode>>,
        ResMut<bevy::input::mouse::AccumulatedMouseMotion>,
    ),
    mut exit: MessageWriter<AppExit>,
    mut controller: ResMut<MapCameraController>,
) {
    let (mut mouse, mut keys, mut motion) = input;
    capture.frame += 1;
    assert!(
        state.error.is_none(),
        "map loading failed: {:?}",
        state.error
    );
    if capture.frame == 35 {
        interface.reduced_motion = true;
        let entity = buttons
            .iter()
            .find(|(_, action)| **action == InterfaceAction::OpenWorldMap)
            .unwrap()
            .0;
        activated.write(ButtonActivated(entity));
    }
    if let Some(map) = map && !state.loading {
        capture.ready_frames += 1;
        let frame = capture.ready_frames;
        // Choose an actual on-screen land pixel. Recover the ID through the camera ray,
        // then exercise the production click path and compare its result next frame.
        if frame == 50 {
            let (cursor, index) = (450..1000)
                .step_by(8)
                .find_map(|x| {
                    let cursor = Vec2::new(x as f32, window.height() * 0.50);
                    let ray = camera.0.viewport_to_world(camera.1, cursor).ok()?;
                    let point = map.0.terrain.intersect_ray(ray.origin, *ray.direction)?;
                    let index = map.0.provinces.index_at_uv(Vec2::new(
                        point.x / map.0.terrain.size.x,
                        point.z / map.0.terrain.size.y,
                    ))?;
                    let province = &map.0.catalog.provinces[index as usize - 1];
                    (!province.water && province.region.is_some()).then_some((cursor, index))
                })
                .expect("visible land province");
            window.set_cursor_position(Some(cursor));
            mouse.press(MouseButton::Left);
            capture.expected_index = index;
        }
        if frame == 51 {
            assert_eq!(state.selected_index, capture.expected_index);
            mouse.release(MouseButton::Left);
        }
        if frame == 70 {
            activated.write(ButtonActivated(triggers.single().unwrap()));
        }
        if frame == 72 {
            assert!(selects.single().unwrap().open);
        }
        if frame == 95 {
            let entity = items
                .iter()
                .find(|(_, item)| item.value == "political")
                .unwrap()
                .0;
            activated.write(ButtonActivated(entity));
        }
        if frame == 98 {
            assert!(state.political);
            assert!(!selects.single().unwrap().open);
        }
        if frame == 115 {
            window.set_cursor_position(Some(Vec2::new(80., 180.)));
            mouse.press(MouseButton::Left);
        }
        if frame == 116 {
            assert!(state.pointer_blocked);
            assert_eq!(state.selected_index, capture.expected_index);
            mouse.release(MouseButton::Left);
        }
        if frame == 140 {
            activated.write(ButtonActivated(triggers.single().unwrap()));
        }
        if frame == 143 {
            keys.press(KeyCode::Escape);
        }
        if frame == 145 {
            assert!(!selects.single().unwrap().open);
            assert_eq!(interface.screen, Screen::WorldMap);
            keys.release(KeyCode::Escape);
        }
        if frame == 160 {
            capture.initial_target = controller.target;
            keys.press(KeyCode::Tab);
        }
        if frame == 162 {
            assert!(state.overview_active);
        }
        if frame == 180 {
            window.set_cursor_position(Some(Vec2::new(800., 430.)));
        }
        if frame == 195 {
            mouse.press(MouseButton::Left);
        }
        if frame == 196 {
            mouse.release(MouseButton::Left);
            assert_eq!(state.selected_index, capture.expected_index);
        }
        if frame == 205 {
            keys.release(KeyCode::Tab);
        }
        if frame == 210 {
            assert!(!state.overview_active);
            assert!(controller.target.distance(capture.initial_target) > 10.);
            controller.target.x = map.0.terrain.size.x + 20.;
        }
        if frame == 211 {
            assert!((controller.target.x - 20.).abs() < 0.01);
        }
        if frame == 220 {
            window.set_cursor_position(Some(Vec2::new(800., 430.)));
            capture.initial_target = controller.target;
            capture.initial_pitch = controller.pitch;
            capture.initial_yaw = controller.yaw;
            mouse.press(MouseButton::Middle);
            motion.delta = Vec2::new(70., 20.);
        }
        if frame == 221 {
            mouse.release(MouseButton::Middle);
            assert!(controller.target.distance(capture.initial_target) > 1.);
            assert_eq!(controller.yaw, capture.initial_yaw);
            assert_eq!(controller.pitch, capture.initial_pitch);
        }
        if frame == 224 {
            capture.initial_target = controller.target;
            mouse.press(MouseButton::Right);
            motion.delta = Vec2::new(70., -20.);
        }
        if frame == 225 {
            assert!((controller.yaw - capture.initial_yaw).abs() > 0.1);
            assert!((controller.pitch - capture.initial_pitch).abs() > 0.01);
            assert_eq!(controller.target, capture.initial_target);
            capture.initial_pitch = controller.pitch;
            interface.reduced_motion = false;
            mouse.release(MouseButton::Right);
        }
        if frame == 227 {
            assert!((controller.yaw - capture.initial_yaw).abs() < 0.28);
            assert_eq!(controller.pitch, capture.initial_pitch);
            interface.reduced_motion = true;
        }
        if frame == 229 {
            assert!((controller.yaw - capture.initial_yaw).abs() < 0.0001);
        }
        if frame == 235 {
            keys.press(KeyCode::Escape);
        }
        if frame == 237 {
            keys.release(KeyCode::Escape);
            assert_eq!(state.selected_index, 0);
            assert_eq!(interface.screen, Screen::WorldMap);
        }
        if frame == 250 {
            interface
                .localization
                .set_language(aggregate_localization::Language::English);
            window.resolution.set(960., 640.);
        }
        let name = match frame {
            60 => Some("world-map-terrain-ko.png"),
            85 => Some("world-map-select-ko.png"),
            130 => Some("world-map-political-ko.png"),
            190 => Some("world-map-overview-ko.png"),
            290 => Some("world-map-small-en.png"),
            _ => None,
        };
        if let Some(name) = name {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/screenshots")
                .join(name);
            commands.spawn(Screenshot::primary_window()).observe(
                move |event: On<ScreenshotCaptured>, mut capture: ResMut<MapCapture>| {
                    event
                        .image
                        .clone()
                        .try_into_dynamic()
                        .unwrap()
                        .to_rgb8()
                        .save(&path)
                        .unwrap();
                    capture.captures += 1;
                },
            );
        }
        if frame > 320 && capture.captures == 5 {
            exit.write(AppExit::Success);
        }
    }
    assert!(capture.frame < 20000, "native map capture timed out");
}
