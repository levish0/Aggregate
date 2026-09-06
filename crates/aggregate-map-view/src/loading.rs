use crate::{
    LoadedAdministration, LoadedWorldMap, MapAssetRoot, MapCameraController, MapTerrainMaterial,
    MapViewState, TerrainMaterialHandle,
    material::{ProvinceStyle, terrain_color},
    terrain_mesh,
};
use aggregate_geography::WorldMap;
use bevy::camera::visibility::RenderLayers;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        storage::ShaderBuffer,
    },
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use std::{collections::BTreeMap, sync::Arc};
use tracing::Instrument;

#[derive(Resource)]
pub struct MapLoadTask(Task<Result<PreparedWorldMap, String>>);

#[derive(Resource)]
pub struct PendingMapTextures(Vec<Handle<Image>>);

pub fn finish_textures(
    mut commands: Commands,
    pending: Option<Res<PendingMapTextures>>,
    server: Res<AssetServer>,
    mut state: ResMut<MapViewState>,
) {
    let Some(pending) = pending else {
        return;
    };
    if pending
        .0
        .iter()
        .all(|handle| server.is_loaded_with_dependencies(handle.id()))
    {
        state.loading = false;
        commands.remove_resource::<PendingMapTextures>();
        info!("Map textures ready");
    }
    for handle in &pending.0 {
        if let Some(bevy::asset::LoadState::Failed(error)) = server.get_load_state(handle.id()) {
            state.error = Some(format!("map texture: {error}"));
            state.loading = false;
            error!(%error, "Map texture loading failed");
            commands.remove_resource::<PendingMapTextures>();
            break;
        }
    }
}

struct PreparedWorldMap {
    map: WorldMap,
    province_texture: Vec<u8>,
    meshes: Vec<Mesh>,
    scenery: crate::scenery::PreparedScenery,
}

pub fn start_loading(
    mut commands: Commands,
    mut state: ResMut<MapViewState>,
    root: Res<MapAssetRoot>,
    loaded: Option<Res<LoadedWorldMap>>,
    task: Option<Res<MapLoadTask>>,
) {
    if !state.enabled || loaded.is_some() || task.is_some() || state.error.is_some() {
        return;
    }
    state.loading = true;
    let root = root.0.clone();
    let span = info_span!("map_preparation", asset_root = %root.display());
    commands.insert_resource(MapLoadTask(
        AsyncComputeTaskPool::get().spawn(
            async move {
                let cache = std::env::var_os("LOCALAPPDATA")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(std::env::temp_dir)
                    .join("Aggregate/cache/geography");
                let (map, report) = WorldMap::load_with_cache(&root, Some(&cache))
                    .map_err(|error| format!("{error:#}"))?;
                info!(
                    cache_hit = report.cache_hit,
                    total_ms = report.total.as_secs_f64() * 1000.,
                    catalog_ms = report.catalog.as_secs_f64() * 1000.,
                    raster_ms = report.raster.as_secs_f64() * 1000.,
                    heightfield_ms = report.heightfield.as_secs_f64() * 1000.,
                    cache_ms = report.cache.as_secs_f64() * 1000.,
                    "Map data loaded"
                );
                if let Some(reason) = &report.cache_warning {
                    warn!(%reason, "Map cache fallback");
                }
                let started = std::time::Instant::now();
                let province_texture = if cfg!(target_endian = "little") {
                    bytemuck::cast_slice(&map.provinces.indices).to_vec()
                } else {
                    map.provinces
                        .indices
                        .iter()
                        .flat_map(|index| index.to_le_bytes())
                        .collect()
                };
                let meshes = terrain_mesh::chunks(&map.terrain);
                info!(
                    elapsed_ms = started.elapsed().as_secs_f64() * 1000.,
                    mesh_count = meshes.len(),
                    "Map meshes prepared"
                );
                let scenery = crate::scenery::prepare(&root, &map)?;
                Ok(PreparedWorldMap {
                    map,
                    province_texture,
                    meshes,
                    scenery,
                })
            }
            .instrument(span),
        ),
    ));
}

pub fn finish_loading(
    mut commands: Commands,
    server: Res<AssetServer>,
    task: Option<ResMut<MapLoadTask>>,
    mut state: ResMut<MapViewState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut materials: ResMut<Assets<MapTerrainMaterial>>,
    mut controller: ResMut<MapCameraController>,
    mut forest_materials: ResMut<Assets<crate::scenery::ForestMaterial>>,
    mut road_materials: ResMut<Assets<crate::scenery::RoadMaterial>>,
) {
    let Some(mut task) = task else {
        return;
    };
    let Some(result) = block_on(future::poll_once(&mut task.0)) else {
        return;
    };
    commands.remove_resource::<MapLoadTask>();
    let prepared = match result {
        Ok(map) => map,
        Err(error) => {
            state.loading = false;
            error!(%error, "Map preparation failed");
            state.error = Some(error);
            return;
        }
    };
    let PreparedWorldMap {
        map,
        province_texture,
        meshes: prepared_meshes,
        scenery,
    } = prepared;
    let image = images.add(Image::new(
        Extent3d {
            width: map.provinces.width,
            height: map.provinces.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        province_texture,
        TextureFormat::R32Uint,
        RenderAssetUsages::RENDER_WORLD,
    ));
    let countries: BTreeMap<_, _> = map
        .catalog
        .countries
        .iter()
        .enumerate()
        .map(|(i, c)| (&c.id, (i as u32 + 1, c)))
        .collect();
    let regions: BTreeMap<_, _> = map
        .catalog
        .regions
        .iter()
        .enumerate()
        .map(|(i, r)| (&r.id, i as u32 + 1))
        .collect();
    let administration = aggregate_geography::AdministrativeIndex::new(&map.catalog);
    let state_indices: BTreeMap<_, _> = map
        .catalog
        .states
        .iter()
        .enumerate()
        .map(|(index, state)| (&state.id, index as u32 + 1))
        .collect();
    let mut styles = vec![ProvinceStyle {
        terrain: Vec4::new(0.025, 0.095, 0.15, 1.),
        political: Vec4::ZERO,
        grouping: UVec4::ZERO,
    }];
    for (province_index, province) in map.catalog.provinces.iter().enumerate() {
        let country = province.owner.as_ref().and_then(|id| countries.get(id));
        let terrain = terrain_color(&province.terrain, province.water);
        let political = country
            .map(|(_, country)| {
                let color =
                    Color::srgb(country.color[0], country.color[1], country.color[2]).to_linear();
                Vec4::new(color.red, color.green, color.blue, 1.)
            })
            .unwrap_or(terrain);
        styles.push(ProvinceStyle {
            terrain,
            political,
            grouping: UVec4::new(
                province
                    .region
                    .as_ref()
                    .and_then(|id| regions.get(id))
                    .copied()
                    .unwrap_or(0),
                country.map(|(index, _)| *index).unwrap_or(0),
                u32::from(province.water),
                administration
                    .state_for_province(province_index as u32 + 1)
                    .and_then(|id| state_indices.get(id))
                    .copied()
                    .unwrap_or(0),
            ),
        });
    }
    let terrain_material = MapTerrainMaterial {
        view: Vec4::ZERO,
        selection: UVec4::ZERO,
        province_indices: image,
        province_styles: buffers.add(ShaderBuffer::from(styles)),
        color_map: server.load("gfx/map/textures/colormap.dds"),
        terrain_diffuse: repeating_texture(&server, "gfx/map/derived/terrain_diffuse.dds", true),
        terrain_normal: repeating_texture(&server, "gfx/map/derived/terrain_normal.dds", false),
        terrain_properties: repeating_texture(
            &server,
            "gfx/map/derived/terrain_material.dds",
            false,
        ),
        terrain_relief: server
            .load_builder()
            .with_settings(|settings: &mut bevy::image::ImageLoaderSettings| {
                settings.is_srgb = false;
            })
            .load("gfx/map/derived/terrain_relief.dds"),
        cloud_density: repeating_texture(&server, "gfx/map/fog_of_war/cloud.dds", false),
        water_flow: repeating_texture(&server, "gfx/map/water/flowmap.dds", false),
        water_normal: repeating_texture(&server, "gfx/map/water/ambient_normal.dds", false),
        terrain_weights: server
            .load_builder()
            .with_settings(|settings: &mut bevy::image::ImageLoaderSettings| {
                settings.is_srgb = false;
            })
            .load("gfx/map/derived/terrain_weights.png"),
        terrain_indices: server
            .load_builder()
            .with_settings(|settings: &mut bevy::image::ImageLoaderSettings| {
                settings.is_srgb = false;
            })
            .load("gfx/map/derived/terrain_indices.png"),
        water_color: server.load("gfx/map/water/watercolor_rgb_waterspec_a.dds"),
        river_distance: server
            .load_builder()
            .with_settings(|settings: &mut bevy::image::ImageLoaderSettings| {
                settings.is_srgb = false;
            })
            .load("map_data/river_distance.png"),
    };
    commands.insert_resource(PendingMapTextures(vec![
        terrain_material.color_map.clone(),
        terrain_material.terrain_diffuse.clone(),
        terrain_material.terrain_normal.clone(),
        terrain_material.terrain_properties.clone(),
        terrain_material.terrain_relief.clone(),
        terrain_material.water_normal.clone(),
        terrain_material.cloud_density.clone(),
        terrain_material.water_flow.clone(),
        terrain_material.terrain_weights.clone(),
        terrain_material.terrain_indices.clone(),
        terrain_material.water_color.clone(),
        terrain_material.river_distance.clone(),
    ]));
    let material = materials.add(terrain_material);
    for mesh in prepared_meshes {
        let mesh = meshes.add(mesh);
        for copy in -2..=2 {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(copy as f32 * map.terrain.size.x, 0., 0.),
                RenderLayers::layer(1),
            ));
        }
    }
    let forest = forest_materials.add(crate::scenery::ForestMaterial { tint: Vec4::ONE });
    let road = road_materials.add(crate::scenery::RoadMaterial {
        diffuse: repeating_texture(
            &server,
            "gfx/map/spline_network/road_paved_diffuse.dds",
            true,
        ),
    });
    crate::scenery::spawn(
        &mut commands,
        scenery,
        map.terrain.size.x,
        &mut meshes,
        forest,
        road,
    );
    controller.target = Vec3::new(map.terrain.size.x * 0.51, 0., map.terrain.size.y * 0.24);
    controller.distance = 360.;
    controller.desired_distance = 360.;
    commands.insert_resource(TerrainMaterialHandle(material));
    commands.insert_resource(LoadedAdministration(administration));
    commands.insert_resource(LoadedWorldMap(Arc::new(map)));
}

pub(crate) fn repeating_texture(
    server: &AssetServer,
    path: &'static str,
    is_srgb: bool,
) -> Handle<Image> {
    server
        .load_builder()
        .with_settings(move |settings: &mut bevy::image::ImageLoaderSettings| {
            settings.is_srgb = is_srgb;
            settings.sampler =
                bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                    address_mode_u: bevy::image::ImageAddressMode::Repeat,
                    address_mode_v: bevy::image::ImageAddressMode::Repeat,
                    mag_filter: bevy::image::ImageFilterMode::Linear,
                    min_filter: bevy::image::ImageFilterMode::Linear,
                    mipmap_filter: bevy::image::ImageFilterMode::Linear,
                    anisotropy_clamp: 8,
                    ..default()
                });
        })
        .load(path)
}
