//! A separate cloud layer; the terrain shader projects the same density as shadow.
use crate::LoadedWorldMap;
use bevy::{
    camera::visibility::RenderLayers, prelude::*, render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MapCloudMaterial {
    #[uniform(4)]
    pub visibility: Vec4,
    #[texture(0)]
    #[sampler(1)]
    pub density: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub normal: Handle<Image>,
}

impl Material for MapCloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_clouds.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

pub fn spawn_clouds(
    mut commands: Commands,
    map: Option<Res<LoadedWorldMap>>,
    mut spawned: Local<bool>,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MapCloudMaterial>>,
) {
    let Some(map) = map else {
        return;
    };
    if *spawned {
        return;
    }
    *spawned = true;
    let size = map.0.terrain.size;
    let material = materials.add(MapCloudMaterial {
        visibility: Vec4::new(0., size.x, 0., 0.),
        density: crate::loading::repeating_texture(&server, "gfx/map/fog_of_war/cloud.dds", false),
        normal: crate::loading::repeating_texture(
            &server,
            "gfx/map/fog_of_war/cloud_normal.dds",
            false,
        ),
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size.x * 5., size.y))),
        MeshMaterial3d(material),
        Transform::from_xyz(size.x * 0.5, 3.5, size.y * 0.5),
        RenderLayers::layer(1),
    ));
}

pub fn opacity(distance: f32) -> f32 {
    let near = ((distance - 35.) / 45.).clamp(0., 1.);
    let far = ((distance - 110.) / 70.).clamp(0., 1.);
    near * near * (3. - 2. * near) * (1. - far * far * (3. - 2. * far))
}

pub fn update_visibility(
    controller: Res<crate::MapCameraController>,
    mut materials: ResMut<Assets<MapCloudMaterial>>,
) {
    if !controller.is_changed() {
        return;
    }
    let opacity = opacity(controller.distance);
    for (_, material) in materials.iter_mut() {
        let visibility = Vec4::new(opacity, material.visibility.y, 0., 0.);
        if material.visibility != visibility {
            material.visibility = visibility;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn clouds_fade_out_for_close_inspection_and_world_overview() {
        assert_eq!(super::opacity(25.), 0.);
        assert_eq!(super::opacity(95.), 1.);
        assert_eq!(super::opacity(450.), 0.);
        assert_eq!(super::opacity(180.), 0.);
        assert!(super::opacity(55.) > 0. && super::opacity(55.) < 1.);
    }
}
