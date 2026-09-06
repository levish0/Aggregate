fn main() -> anyhow::Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let cache = root.join("../target/map-cache");
    for (label, directory) in [
        ("source", None),
        ("cache build", Some(cache.as_path())),
        ("cache read", Some(cache.as_path())),
    ] {
        let (map, report) = aggregate_geography::WorldMap::load_with_cache(&root, directory)?;
        println!(
            "{label}: {report:?}; {} provinces",
            map.catalog.provinces.len()
        );
    }
    Ok(())
}
