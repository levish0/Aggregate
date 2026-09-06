use anyhow::{Result, ensure};
use std::{fs, path::Path};

pub fn validate() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("assets");
    let count = validate_directory(&root)?;
    ensure!(count > 0, "no RON assets found");
    println!("Validated {count} RON assets.");
    Ok(())
}

fn validate_directory(directory: &Path) -> Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            count += validate_directory(&entry.path())?;
        } else if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "ron")
        {
            if entry.file_name() == "geography.ron" {
                aggregate_geography::GeographyCatalog::load(&entry.path())?;
            } else if entry.file_name() == "terrain_settings.ron" {
                aggregate_geography::TerrainSettings::load(&entry.path())?;
            } else {
                aggregate_asset_data::parse_asset_document(&fs::read_to_string(entry.path())?)
                    .map_err(anyhow::Error::msg)?;
            }
            count += 1;
        }
    }
    Ok(count)
}
