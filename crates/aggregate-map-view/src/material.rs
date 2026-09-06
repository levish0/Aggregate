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
    #[texture(3)]
    #[sampler(4)]
    pub color_map: Handle<Image>,
    #[texture(5, dimension = "2d_array")]
    #[sampler(6)]
    pub terrain_diffuse: Handle<Image>,
    #[texture(7, dimension = "2d_array")]
    #[sampler(8)]
    pub terrain_normal: Handle<Image>,
    #[texture(9)]
    #[sampler(10)]
    pub water_color: Handle<Image>,
    #[texture(11)]
    #[sampler(12)]
    pub river_distance: Handle<Image>,
    #[texture(13)]
    #[sampler(14)]
    pub terrain_weights: Handle<Image>,
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
            u32::from(state.inspect_country),
        );
        if material.selection != selection {
            material.selection = selection;
        }
    }
}
