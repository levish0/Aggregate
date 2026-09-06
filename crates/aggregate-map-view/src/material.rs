use crate::{MapViewState, TerrainMaterialHandle};
use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderBuffer,
    },
    shader::ShaderRef,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MapTerrainMaterial {
    #[uniform(0)]
    pub selection: UVec4,
    #[texture(1, sample_type = "u_int")]
    pub province_indices: Handle<Image>,
    #[storage(2, read_only)]
    pub province_styles: Handle<ShaderBuffer>,
}

#[derive(Clone, Copy, ShaderType)]
pub struct ProvinceStyle {
    pub terrain: Vec4,
    pub political: Vec4,
    pub grouping: UVec4,
}

impl Material for MapTerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_terrain.wgsl".into()
    }
}

pub fn terrain_color(name: &str, water: bool) -> Vec4 {
    if water {
        return Vec4::new(0.025, 0.095, 0.15, 1.);
    }
    match name {
        "mountain" | "mountains" => Vec4::new(0.28, 0.29, 0.25, 1.),
        "desert" => Vec4::new(0.55, 0.40, 0.22, 1.),
        "jungle" => Vec4::new(0.055, 0.18, 0.095, 1.),
        "forest" => Vec4::new(0.13, 0.23, 0.125, 1.),
        "snow" | "tundra" => Vec4::new(0.50, 0.53, 0.49, 1.),
        "hills" => Vec4::new(0.27, 0.31, 0.17, 1.),
        _ => Vec4::new(0.32, 0.38, 0.20, 1.),
    }
}

pub fn update_selection(
    state: Res<MapViewState>,
    handle: Option<Res<TerrainMaterialHandle>>,
    mut materials: ResMut<Assets<MapTerrainMaterial>>,
) {
    if let Some(handle) = handle
        && let Some(mut material) = materials.get_mut(&handle.0)
    {
        let selection = UVec4::new(
            state.selected_index,
            state.hovered_index,
            u32::from(state.political),
            0,
        );
        if material.selection != selection {
            material.selection = selection;
        }
    }
}
