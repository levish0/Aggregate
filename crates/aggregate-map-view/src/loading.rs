use crate::{
    LoadedWorldMap, MapAssetRoot, MapCameraController, MapTerrainMaterial, MapViewState,
    TerrainMaterialHandle,
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

#[derive(Resource)]
pub struct MapLoadTask(Task<Result<PreparedWorldMap, String>>);

struct PreparedWorldMap {
    map: WorldMap,
    province_texture: Vec<u8>,
    meshes: Vec<Mesh>,
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
    commands.insert_resource(MapLoadTask(AsyncComputeTaskPool::get().spawn(async move {
        let cache = std::env::var_os("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("Aggregate/cache/geography");
        let (map, report) =
            WorldMap::load_with_cache(&root, Some(&cache)).map_err(|error| format!("{error:#}"))?;
        info!("Map data load: {report:?}");
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
        info!("Map render preparation: {:?}", started.elapsed());
        Ok(PreparedWorldMap {
            map,
            province_texture,
            meshes,
        })
    })));
}

pub fn finish_loading(
    mut commands: Commands,
    task: Option<ResMut<MapLoadTask>>,
    mut state: ResMut<MapViewState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut materials: ResMut<Assets<MapTerrainMaterial>>,
    mut controller: ResMut<MapCameraController>,
) {
    let Some(mut task) = task else {
        return;
    };
    let Some(result) = block_on(future::poll_once(&mut task.0)) else {
        return;
    };
    commands.remove_resource::<MapLoadTask>();
    state.loading = false;
    let prepared = match result {
        Ok(map) => map,
        Err(error) => {
            state.error = Some(error);
            return;
        }
    };
    let PreparedWorldMap {
        map,
        province_texture,
        meshes: prepared_meshes,
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
    let mut styles = vec![ProvinceStyle {
        terrain: Vec4::new(0.025, 0.095, 0.15, 1.),
        political: Vec4::ZERO,
        grouping: UVec4::ZERO,
    }];
    for province in &map.catalog.provinces {
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
                0,
            ),
        });
    }
    let material = materials.add(MapTerrainMaterial {
        selection: UVec4::ZERO,
        province_indices: image,
        province_styles: buffers.add(ShaderBuffer::from(styles)),
    });
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
    controller.target = Vec3::new(map.terrain.size.x * 0.51, 0., map.terrain.size.y * 0.24);
    controller.distance = 360.;
    controller.desired_distance = 360.;
    commands.insert_resource(TerrainMaterialHandle(material));
    commands.insert_resource(LoadedWorldMap(Arc::new(map)));
}
