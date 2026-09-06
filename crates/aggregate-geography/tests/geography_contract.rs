use aggregate_geography::{GeographyCatalog, MapCountry, MapProvince, MapRegion, ProvinceRaster};
use glam::Vec2;
use uuid::Uuid;

fn catalog() -> GeographyCatalog {
    GeographyCatalog {
        schema_version: 2,
        states: vec![aggregate_geography::MapState {
            id: Uuid::from_u128(5).into(),
            region: Uuid::from_u128(2).into(),
            country: Uuid::from_u128(1).into(),
        }],
        countries: vec![MapCountry {
            id: Uuid::from_u128(1).into(),
            key: "country".into(),
            color: [0.2, 0.3, 0.4],
        }],
        regions: vec![MapRegion {
            id: Uuid::from_u128(2).into(),
            key: "region".into(),
            name: "Region".into(),
        }],
        provinces: vec![
            MapProvince {
                id: Uuid::from_u128(3).into(),
                raster_color: 0xff0000,
                region: Some(Uuid::from_u128(2).into()),
                owner: Some(Uuid::from_u128(1).into()),
                terrain: "plains".into(),
                water: false,
            },
            MapProvince {
                id: Uuid::from_u128(4).into(),
                raster_color: 0x00ff00,
                region: None,
                owner: None,
                terrain: "ocean".into(),
                water: true,
            },
        ],
    }
}

#[test]
fn catalog_rejects_broken_identity_references_and_colors() {
    let mut data = catalog();
    data.validate().unwrap();
    data.provinces[0].id = Uuid::nil().into();
    assert!(data.validate().unwrap_err().to_string().contains("UUID"));
    let mut data = catalog();
    data.provinces[1].raster_color = 0xff0000;
    assert!(data.validate().is_err());
    let mut data = catalog();
    data.provinces[0].owner = Some(Uuid::from_u128(99).into());
    assert!(data.validate().unwrap_err().to_string().contains("owner"));
    let mut data = catalog();
    data.provinces[0].region = Some(Uuid::from_u128(99).into());
    assert!(data.validate().unwrap_err().to_string().contains("region"));
}

#[test]
fn raster_lookup_retains_uuid_when_catalog_order_changes_and_rejects_unknown_colors() {
    let directory = std::env::temp_dir().join(format!("aggregate-raster-{}", Uuid::now_v7()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("provinces.png");
    let pixels =
        image::RgbImage::from_raw(2, 2, vec![255, 0, 0, 0, 255, 0, 0, 255, 0, 255, 0, 0]).unwrap();
    pixels.save(&path).unwrap();
    let mut data = catalog();
    let raster = ProvinceRaster::load(&path, &data).unwrap();
    let id = data.provinces[raster.index_at_uv(Vec2::new(0.1, 0.1)).unwrap() as usize - 1]
        .id
        .clone();
    assert_eq!(raster.index_at_uv(Vec2::new(0.5, 0.)), Some(2));
    assert_eq!(raster.index_at_uv(Vec2::ONE), Some(1));
    assert_eq!(raster.index_at_uv(Vec2::new(-0.001, 0.)), None);
    assert_eq!(raster.index_at_uv(Vec2::splat(f32::NAN)), None);
    data.provinces.reverse();
    let reordered = ProvinceRaster::load(&path, &data).unwrap();
    assert_eq!(
        id,
        data.provinces[reordered.index_at_uv(Vec2::new(0.1, 0.1)).unwrap() as usize - 1].id
    );
    let restored: GeographyCatalog = ron::from_str(&ron::to_string(&data).unwrap()).unwrap();
    assert_eq!(restored.provinces[1].id, id);
    data.provinces.pop();
    assert!(ProvinceRaster::load(&path, &data).is_err());
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn derived_cache_recovers_from_corruption_and_invalidates_on_authored_changes() {
    use aggregate_geography::{TerrainSettings, WorldMap};
    let root = std::env::temp_dir().join(format!("aggregate-cache-test-{}", Uuid::now_v7()));
    let assets = root.join("map_data");
    let cache = root.join("cache");
    std::fs::create_dir_all(&assets).unwrap();
    let mut data = catalog();
    std::fs::write(assets.join("geography.ron"), ron::to_string(&data).unwrap()).unwrap();
    let mut settings = TerrainSettings {
        schema_version: 1,
        grid_columns: 32,
        world_width: 20.,
        sea_level_raw: 0,
        height_scale: 10.,
    };
    std::fs::write(
        assets.join("terrain_settings.ron"),
        ron::to_string(&settings).unwrap(),
    )
    .unwrap();
    image::RgbImage::from_raw(2, 2, vec![255, 0, 0, 0, 255, 0, 0, 255, 0, 255, 0, 0])
        .unwrap()
        .save(assets.join("provinces.png"))
        .unwrap();
    image::ImageBuffer::<image::Luma<u16>, _>::from_raw(2, 2, vec![20000u16; 4])
        .unwrap()
        .save(assets.join("heightmap.png"))
        .unwrap();
    let (source, first) = WorldMap::load_with_cache(&root, Some(&cache)).unwrap();
    assert!(!first.cache_hit);
    let (cached, second) = WorldMap::load_with_cache(&root, Some(&cache)).unwrap();
    assert!(second.cache_hit);
    assert_eq!(source.provinces.indices, cached.provinces.indices);
    assert_eq!(source.terrain.heights, cached.terrain.heights);
    let path = std::fs::read_dir(&cache)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(&path, [0xff; 10]).unwrap();
    let (_, repaired) = WorldMap::load_with_cache(&root, Some(&cache)).unwrap();
    assert!(!repaired.cache_hit && repaired.cache_warning.is_some());
    assert!(
        WorldMap::load_with_cache(&root, Some(&cache))
            .unwrap()
            .1
            .cache_hit
    );
    settings.height_scale = 20.;
    std::fs::write(
        assets.join("terrain_settings.ron"),
        ron::to_string(&settings).unwrap(),
    )
    .unwrap();
    let (changed, report) = WorldMap::load_with_cache(&root, Some(&cache)).unwrap();
    assert!(!report.cache_hit);
    assert!(changed.terrain.heights[0] > source.terrain.heights[0]);
    data.provinces.reverse();
    std::fs::write(assets.join("geography.ron"), ron::to_string(&data).unwrap()).unwrap();
    let (changed, report) = WorldMap::load_with_cache(&root, Some(&cache)).unwrap();
    assert!(!report.cache_hit);
    assert_eq!(
        changed.catalog.provinces[changed.provinces.indices[0] as usize - 1].id,
        source.catalog.provinces[0].id
    );
    for directory in [&assets, &cache] {
        for entry in std::fs::read_dir(directory).unwrap() {
            std::fs::remove_file(entry.unwrap().path()).unwrap();
        }
        std::fs::remove_dir(directory).unwrap();
    }
    std::fs::remove_dir(root).unwrap();
}

#[test]
fn repeated_world_copies_resolve_to_the_same_terrain_hit() {
    use glam::Vec3;
    let terrain = aggregate_geography::Heightfield {
        columns: 1,
        rows: 1,
        size: Vec2::new(10., 10.),
        heights: vec![0.; 4],
    };
    for copy in -2..=2 {
        let x = 4. + copy as f32 * terrain.size.x;
        let hit = terrain
            .intersect_wrapped_ray(Vec3::new(x, 8., 6.), Vec3::NEG_Y)
            .unwrap();
        assert!((hit.x.rem_euclid(terrain.size.x) - 4.).abs() < 0.001);
        assert!((hit.z - 6.).abs() < 0.001);
    }
    assert!(
        terrain
            .intersect_wrapped_ray(Vec3::new(4., 8., -10.), Vec3::NEG_Y)
            .is_none()
    );
}
