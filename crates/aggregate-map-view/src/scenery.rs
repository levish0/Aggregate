//! Offline-authored scenery, batched into spatial meshes on the map loading worker.
mod geometry;
mod preparation;

use crate::{MapCameraController, MapViewState};
use bevy::{
    camera::visibility::RenderLayers, prelude::*, render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
pub use preparation::prepare;

pub struct PreparedScenery {
    pub forests: Vec<(Vec2, Mesh)>,
    pub roads: Vec<(Vec2, Mesh)>,
}

#[derive(Component)]
pub struct SceneryChunk(pub Vec2);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ForestMaterial {
    #[uniform(0)]
    pub tint: Vec4,
}

impl Material for ForestMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_forest.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct RoadMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub diffuse: Handle<Image>,
}

impl Material for RoadMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_roads.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

pub fn spawn(
    commands: &mut Commands,
    prepared: PreparedScenery,
    width: f32,
    meshes: &mut Assets<Mesh>,
    forest: Handle<ForestMaterial>,
    road: Handle<RoadMaterial>,
) {
    for (center, mesh) in prepared.forests {
        let mesh = meshes.add(mesh);
        for copy in -2..=2 {
            let offset = copy as f32 * width;
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(forest.clone()),
                Transform::from_xyz(offset, 0., 0.),
                SceneryChunk(center + Vec2::X * offset),
                RenderLayers::layer(1),
                Visibility::Hidden,
            ));
        }
    }
    for (center, mesh) in prepared.roads {
        let mesh = meshes.add(mesh);
        for copy in -2..=2 {
            let offset = copy as f32 * width;
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(road.clone()),
                Transform::from_xyz(offset, 0., 0.),
                SceneryChunk(center + Vec2::X * offset),
                RenderLayers::layer(1),
                Visibility::Hidden,
            ));
        }
    }
}

pub fn update_visibility(
    state: Res<MapViewState>,
    controller: Res<MapCameraController>,
    mut chunks: Query<(&SceneryChunk, &mut Visibility)>,
) {
    if !controller.is_changed() && !state.is_changed() {
        return;
    }
    for (chunk, mut visibility) in &mut chunks {
        let visible = state.enabled
            && controller.distance < 280.
            && chunk.0.distance(controller.target.xz()) < controller.distance * 1.6 + 90.;
        let next = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}
