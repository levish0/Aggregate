use crate::{LoadedWorldMap, MapCameraController, MapViewState};
use bevy::{camera::visibility::RenderLayers, prelude::*};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct OverviewGizmos;

pub fn configure(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<OverviewGizmos>();
    config.render_layers = RenderLayers::layer(1);
    config.line.width = 2.;
    config.depth_bias = -1.;
}

/// Preview the post-zoom viewport at the pointer, without changing the committed destination.
pub fn draw_selection(
    state: Res<MapViewState>,
    controller: Res<MapCameraController>,
    map: Option<Res<LoadedWorldMap>>,
    window: Single<&Window>,
    mut gizmos: Gizmos<OverviewGizmos>,
) {
    if !state.enabled {
        return;
    }
    let (Some(saved), Some(map)) = (&controller.overview_return, map) else {
        return;
    };
    let Some(target) = controller.overview_preview else {
        return;
    };
    let offset = Quat::from_rotation_y(saved.yaw)
        * Vec3::new(0., saved.pitch.sin(), saved.pitch.cos())
        * saved.distance;
    let transform = Transform::from_translation(target + offset).looking_at(target, Vec3::Y);
    let tangent = (PerspectiveProjection::default().fov * 0.5).tan();
    let aspect = window.width() / window.height().max(1.);
    let points: Vec<_> = [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.), (-1., -1.)]
        .into_iter()
        .map(|(x, y)| {
            let ray = transform.rotation * Vec3::new(x * tangent * aspect, y * tangent, -1.);
            transform.translation + ray * (-transform.translation.y / ray.y.min(-0.001))
        })
        .collect();
    for copy in -2..=2 {
        gizmos.linestrip(
            points
                .iter()
                .map(|point| *point + Vec3::X * (copy as f32 * map.0.terrain.size.x)),
            Color::srgba(0.95, 0.95, 0.9, 0.8),
        );
    }
}
