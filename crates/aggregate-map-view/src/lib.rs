//! Native terrain rendering and interaction.
mod camera;
mod loading;
mod material;
mod overview;
mod table;
mod terrain_mesh;

use aggregate_geography::WorldMap;
use bevy::prelude::*;
use std::{path::PathBuf, sync::Arc};

pub use camera::{MapCamera, MapCameraController};
pub use material::MapTerrainMaterial;

pub struct MapViewPlugin {
    pub asset_root: PathBuf,
}

#[derive(Resource, Default)]
pub struct MapViewState {
    pub enabled: bool,
    pub pointer_blocked: bool,
    pub selected_index: u32,
    pub hovered_index: u32,
    pub political: bool,
    pub error: Option<String>,
    pub loading: bool,
    pub overview_allowed: bool,
    pub overview_active: bool,
    pub reduced_motion: bool,
}

#[derive(Resource)]
pub struct LoadedWorldMap(pub Arc<WorldMap>);

#[derive(Resource)]
pub struct LoadedAdministration(pub aggregate_geography::AdministrativeIndex);

#[derive(Resource)]
struct MapAssetRoot(PathBuf);

#[derive(Resource)]
struct TerrainMaterialHandle(Handle<MapTerrainMaterial>);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapViewSystems;

impl Plugin for MapViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapAssetRoot(self.asset_root.clone()))
            .init_resource::<MapViewState>()
            .init_resource::<MapCameraController>()
            .add_plugins(MaterialPlugin::<MapTerrainMaterial>::default())
            .add_plugins(MaterialPlugin::<table::MapTableMaterial>::default())
            .init_gizmo_group::<overview::OverviewGizmos>()
            .add_systems(Startup, (camera::setup, overview::configure))
            .add_systems(
                Update,
                (
                    loading::start_loading,
                    loading::finish_loading,
                    table::spawn_table,
                    camera::control,
                    camera::update_camera,
                    material::update_selection,
                )
                    .chain()
                    .in_set(MapViewSystems),
            )
            .add_systems(
                PostUpdate,
                (camera::pick, overview::draw_selection)
                    .chain()
                    .after(bevy::transform::TransformSystems::Propagate),
            );
    }
}
