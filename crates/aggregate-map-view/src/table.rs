use crate::LoadedWorldMap;
use bevy::{
    camera::visibility::RenderLayers, prelude::*, render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct MapTableMaterial {
    #[uniform(0)]
    pub color: Vec4,
}
impl Material for MapTableMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_table.wgsl".into()
    }
}

pub fn spawn_table(
    mut commands: Commands,
    map: Option<Res<LoadedWorldMap>>,
    mut spawned: Local<bool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MapTableMaterial>>,
    mut standard: ResMut<Assets<StandardMaterial>>,
) {
    let Some(map) = map else {
        return;
    };
    if *spawned {
        return;
    }
    *spawned = true;
    let size = map.0.terrain.size;
    let surface = materials.add(MapTableMaterial {
        color: Vec4::new(0.085, 0.039, 0.019, 1.),
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x * 5., 3., size.y + 9000.))),
        MeshMaterial3d(surface),
        Transform::from_xyz(size.x * 0.5, -2., size.y * 0.5),
        RenderLayers::layer(1),
    ));
    let paper = standard.add(StandardMaterial {
        base_color: Color::srgb(0.66, 0.59, 0.43),
        unlit: true,
        ..default()
    });
    let gold = standard.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.32, 0.18),
        unlit: true,
        ..default()
    });
    for z in [-7., size.y + 7.] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x * 5., 0.15, 14.))),
            MeshMaterial3d(paper.clone()),
            Transform::from_xyz(size.x * 0.5, -0.12, z),
            RenderLayers::layer(1),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x * 5., 0.05, 0.8))),
            MeshMaterial3d(gold.clone()),
            Transform::from_xyz(size.x * 0.5, 0., z),
            RenderLayers::layer(1),
        ));
    }
}
