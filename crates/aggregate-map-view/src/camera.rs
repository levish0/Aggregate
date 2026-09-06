use crate::{LoadedWorldMap, MapViewState};
use bevy::{
    camera::visibility::RenderLayers,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
};

#[derive(Component)]
pub struct MapCamera;

#[derive(Resource)]
pub struct MapCameraController {
    pub target: Vec3,
    pub distance: f32,
    pub desired_distance: f32,
    pub pitch: f32,
    pub yaw: f32,
    rotation_return_yaw: Option<f32>,
    rotating: bool,
    panning: bool,
    pub(crate) overview_return: Option<OverviewReturn>,
    pub(crate) overview_preview: Option<Vec3>,
}

pub(crate) struct OverviewReturn {
    pub target: Vec3,
    pub distance: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for MapCameraController {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 360.,
            desired_distance: 360.,
            pitch: 0.9,
            yaw: 0.,
            rotation_return_yaw: None,
            rotating: false,
            panning: false,
            overview_return: None,
            overview_preview: None,
        }
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            is_active: false,
            order: 0,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.025, 0.04, 0.055)),
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            far: 6000.,
            near: 0.5,
            ..default()
        }),
        Transform::from_xyz(0., 300., 300.).looking_at(Vec3::ZERO, Vec3::Y),
        MapCamera,
        RenderLayers::layer(1),
        Msaa::Sample4,
    ));
}

pub fn control(
    mut state: ResMut<MapViewState>,
    map: Option<Res<LoadedWorldMap>>,
    mut controller: ResMut<MapCameraController>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    window: Single<&Window>,
) {
    if !state.enabled {
        controller.rotating = false;
        controller.panning = false;
        if let Some(yaw) = controller.rotation_return_yaw.take() {
            controller.yaw = yaw;
        }
        if let Some(saved) = controller.overview_return.take() {
            controller.target = saved.target;
            controller.desired_distance = saved.distance;
            controller.pitch = saved.pitch;
            controller.yaw = saved.yaw;
        }
        state.overview_active = false;
        controller.overview_preview = None;
        return;
    }
    let Some(map) = map else {
        return;
    };
    if keys.just_pressed(KeyCode::Tab)
        && state.overview_allowed
        && controller.overview_return.is_none()
    {
        controller.rotating = false;
        controller.panning = false;
        if let Some(yaw) = controller.rotation_return_yaw.take() {
            controller.yaw = yaw;
        }
        controller.overview_return = Some(OverviewReturn {
            target: controller.target,
            distance: controller.desired_distance,
            pitch: controller.pitch,
            yaw: controller.yaw,
        });
        controller.overview_preview = Some(controller.target);
        controller.target = Vec3::new(map.0.terrain.size.x * 0.5, 0., map.0.terrain.size.y * 0.5);
        let aspect = window.width() / window.height().max(1.);
        let tangent = (PerspectiveProjection::default().fov * 0.5).tan();
        controller.desired_distance =
            (map.0.terrain.size.x / aspect).max(map.0.terrain.size.y) * 0.62 / tangent;
        controller.pitch = 1.55;
        controller.yaw = 0.;
    }
    if (!keys.pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::Escape))
        && let Some(saved) = controller.overview_return.take()
    {
        controller.target = saved.target;
        controller.desired_distance = saved.distance;
        controller.pitch = saved.pitch;
        controller.yaw = saved.yaw;
        controller.overview_preview = None;
    }
    state.overview_active = controller.overview_return.is_some();
    let distance = controller.distance;
    if !buttons.pressed(MouseButton::Right) {
        controller.rotating = false;
    }
    if !buttons.pressed(MouseButton::Middle) {
        controller.panning = false;
    }
    if !state.pointer_blocked && !state.overview_active {
        if buttons.just_pressed(MouseButton::Right) {
            let yaw = controller.yaw;
            controller.rotation_return_yaw.get_or_insert(yaw);
            controller.rotating = true;
        }
        if buttons.just_pressed(MouseButton::Middle) {
            controller.panning = true;
        }
    }
    if controller.rotating {
        controller.yaw -= motion.delta.x * 0.004;
        controller.pitch = (controller.pitch + motion.delta.y * 0.003).clamp(0.5, 1.45);
    } else if let Some(yaw) = controller.rotation_return_yaw {
        let factor = if state.reduced_motion {
            1.
        } else {
            1. - (-time.delta_secs() * 12.).exp()
        };
        controller.yaw += (yaw - controller.yaw) * factor;
        if (controller.yaw - yaw).abs() < 0.0001 {
            controller.yaw = yaw;
            controller.rotation_return_yaw = None;
        }
    }
    if controller.panning && !controller.rotating {
        // Project screen motion onto the horizontal map at the camera target.
        let units_per_pixel = 2. * distance * (PerspectiveProjection::default().fov * 0.5).tan()
            / window.height().max(1.);
        let movement = Vec3::new(
            -motion.delta.x,
            0.,
            -motion.delta.y / controller.pitch.sin(),
        );
        let orientation = Quat::from_rotation_y(controller.yaw);
        controller.target += orientation * movement * units_per_pixel;
    }
    if !state.pointer_blocked && !state.overview_active {
        let wheel = scroll.delta.y
            * if scroll.unit == MouseScrollUnit::Pixel {
                0.01
            } else {
                1.
            };
        controller.desired_distance =
            (controller.desired_distance * (-wheel * 0.12).exp()).clamp(45., 2400.);
        let mut movement = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            movement.z -= 1.;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            movement.z += 1.;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            movement.x -= 1.;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            movement.x += 1.;
        }
        let orientation = Quat::from_rotation_y(controller.yaw);
        controller.target +=
            orientation * movement.normalize_or_zero() * distance * time.delta_secs() * 0.65;
    }
    controller.target.x = controller.target.x.rem_euclid(map.0.terrain.size.x);
    controller.target.z = controller.target.z.clamp(-map.0.terrain.size.y * 0.18, map.0.terrain.size.y * 1.18);
    // Bound the far ground-plane edge to less than one world width, including tilt.
    let aspect = window.width() / window.height().max(1.);
    let tangent = (PerspectiveProjection::default().fov * 0.5).tan();
    let tilt_factor = (1. - tangent / controller.pitch.tan()).max(0.15);
    let maximum_distance = (map.0.terrain.size.x * 0.49 * tilt_factor / (tangent * aspect)).max(45.);
    controller.desired_distance = controller.desired_distance.min(maximum_distance);
    let factor = if state.reduced_motion {
        1.
    } else {
        1. - (-time.delta_secs() * 12.).exp()
    };
    controller.distance += (controller.desired_distance - controller.distance) * factor;
}

pub fn update_camera(
    state: Res<MapViewState>,
    controller: Res<MapCameraController>,
    mut camera: Single<(&mut Camera, &mut Transform), With<MapCamera>>,
    map: Option<Res<LoadedWorldMap>>,
    time: Res<Time>,
) {
    camera.0.is_active = state.enabled;
    let offset = Quat::from_rotation_y(controller.yaw)
        * Vec3::new(0., controller.pitch.sin(), controller.pitch.cos())
        * controller.distance;
    let desired = Transform::from_translation(controller.target + offset)
        .looking_at(controller.target, Vec3::Y);
    if let Some(map) = map {
        let width = map.0.terrain.size.x;
        let delta = camera.1.translation.x - desired.translation.x;
        camera.1.translation.x -= (delta / width).round() * width;
    }
    let blend = if state.reduced_motion {
        1.
    } else {
        1. - (-time.delta_secs() * 18.).exp()
    };
    camera.1.translation = camera.1.translation.lerp(desired.translation, blend);
    camera.1.rotation = camera.1.rotation.slerp(desired.rotation, blend);
}

pub fn pick(
    mut state: ResMut<MapViewState>,
    map: Option<Res<LoadedWorldMap>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<MapCamera>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut controller: ResMut<MapCameraController>,
) {
    if !state.enabled || state.pointer_blocked {
        state.hovered_index = 0;
        controller.overview_preview = None;
        return;
    }
    let Some(map) = map else {
        return;
    };
    let hit = window
        .cursor_position()
        .and_then(|cursor| camera.0.viewport_to_world(camera.1, cursor).ok())
        .and_then(|ray| {
            map.0
                .terrain
                .intersect_wrapped_ray(ray.origin, *ray.direction)
        });
    state.hovered_index = hit
        .and_then(|position| {
            map.0.provinces.index_at_uv(Vec2::new(
                position.x.rem_euclid(map.0.terrain.size.x) / map.0.terrain.size.x,
                position.z / map.0.terrain.size.y,
            ))
        })
        .unwrap_or(0);
    if state.overview_active {
        controller.overview_preview =
            hit.map(|point| Vec3::new(point.x.rem_euclid(map.0.terrain.size.x), 0., point.z));
    }
    if buttons.just_pressed(MouseButton::Left) && state.hovered_index != 0 {
        if let Some(saved) = &mut controller.overview_return {
            let point = hit.unwrap();
            saved.target = Vec3::new(point.x.rem_euclid(map.0.terrain.size.x), 0., point.z);
        } else {
            state.selected_index = state.hovered_index;
            state.inspect_country = false;
        }
    }
}
