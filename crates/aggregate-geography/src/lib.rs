//! Persistent geography and CPU map queries, independent of rendering and simulation clocks.
mod cache;
mod catalog;
mod heightfield;
mod picking;
mod raster;
mod terrain_settings;
mod administration;

pub use catalog::{GeographyCatalog, MapCountry, MapProvince, MapRegion, MapState};
pub use administration::{AdministrativeIndex, StateGeography};
pub use heightfield::Heightfield;
pub use raster::ProvinceRaster;
pub use terrain_settings::TerrainSettings;

use anyhow::{Context, Result};
use std::{
    path::Path,
    time::{Duration, Instant},
};

#[derive(Debug, Default)]
pub struct MapLoadReport {
    pub catalog: Duration,
    pub fingerprint: Duration,
    pub raster: Duration,
    pub heightfield: Duration,
    pub cache: Duration,
    pub cache_hit: bool,
    pub total: Duration,
    pub cache_warning: Option<String>,
}

pub struct WorldMap {
    pub catalog: GeographyCatalog,
    pub provinces: ProvinceRaster,
    pub terrain: Heightfield,
}

impl WorldMap {
    pub fn load(asset_root: &Path) -> Result<Self> {
        Self::load_with_cache(asset_root, None).map(|(map, _)| map)
    }

    pub fn load_with_cache(
        asset_root: &Path,
        cache_directory: Option<&Path>,
    ) -> Result<(Self, MapLoadReport)> {
        let started = Instant::now();
        let mut report = MapLoadReport::default();
        let phase = Instant::now();
        let catalog = GeographyCatalog::load(&asset_root.join("map_data/geography.ron"))?;
        report.catalog = phase.elapsed();
        let phase = Instant::now();
        let fingerprint = cache_directory
            .map(|_| cache::fingerprint(asset_root))
            .transpose()?;
        report.fingerprint = phase.elapsed();
        let cache_path = cache_directory
            .zip(fingerprint.as_ref())
            .map(|(directory, hash)| directory.join(format!("{hash}.bin")));
        if let Some(path) = &cache_path {
            let phase = Instant::now();
            if path.exists() {
                match cache::read(path, fingerprint.as_ref().unwrap(), catalog.provinces.len()) {
                    Ok((provinces, terrain)) => {
                        report.cache = phase.elapsed();
                        report.cache_hit = true;
                        report.total = started.elapsed();
                        return Ok((
                            Self {
                                catalog,
                                provinces,
                                terrain,
                            },
                            report,
                        ));
                    }
                    Err(error) => {
                        report.cache_warning = Some(format!("cache ignored: {error:#}"));
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        }
        let phase = Instant::now();
        let provinces = ProvinceRaster::load(&asset_root.join("map_data/provinces.png"), &catalog)?;
        report.raster = phase.elapsed();
        let phase = Instant::now();
        let settings = TerrainSettings::load(&asset_root.join("map_data/terrain_settings.ron"))?;
        let terrain = Heightfield::load(
            &asset_root.join("map_data/heightmap.png"),
            &provinces,
            &catalog,
            &settings,
        )
        .context("loading terrain heightfield")?;
        report.heightfield = phase.elapsed();
        if let Some(path) = cache_path {
            let phase = Instant::now();
            // Never cache a mixture of files edited during a load.
            if cache::fingerprint(asset_root)? == *fingerprint.as_ref().unwrap()
                && let Err(error) =
                    cache::write(&path, fingerprint.as_ref().unwrap(), &provinces, &terrain)
            {
                report.cache_warning = Some(format!("cache not saved: {error:#}"));
            }
            report.cache = phase.elapsed();
        }
        report.total = started.elapsed();
        Ok((
            Self {
                catalog,
                provinces,
                terrain,
            },
            report,
        ))
    }
}
