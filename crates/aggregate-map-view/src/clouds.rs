//! A separate cloud layer; the terrain shader projects the same density as shadow.
use crate::LoadedWorldMap;
use bevy::{camera::visibility::RenderLayers, prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MapCloudMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub density: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub normal: Handle<Image>,
}

impl Material for MapCloudMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/map_clouds.wgsl".into() }
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Blend }
}

pub fn spawn_clouds(
    mut commands: Commands,
    map: Option<Res<LoadedWorldMap>>,
    mut spawned: Local<bool>,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MapCloudMaterial>>,
) {
    let Some(map) = map else { return; };
    if *spawned { return; }
    *spawned = true;
    let size = map.0.terrain.size;
    let material = materials.add(MapCloudMaterial {
        density: crate::loading::repeating_texture(&server, "gfx/map/fog_of_war/cloud.dds", false),
        normal: crate::loading::repeating_texture(&server, "gfx/map/fog_of_war/cloud_normal.dds", false),
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size.x*5.,size.y))),
        MeshMaterial3d(material),
        Transform::from_xyz(size.x*0.5,14.,size.y*0.5),
        RenderLayers::layer(1),
    ));
}
