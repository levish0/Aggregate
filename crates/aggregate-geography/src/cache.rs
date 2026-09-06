//! Disposable, content-addressed derived planes. Authored RON and PNG files stay authoritative.
use crate::{Heightfield, ProvinceRaster};
use anyhow::{Result, ensure};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

const VERSION: u32 = 1;
const MAX_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct Header {
    version: u32,
    little_endian: bool,
    source_hash: String,
    payload_hash: String,
    width: u32,
    height: u32,
    centroids: Vec<[f32; 2]>,
    columns: u32,
    rows: u32,
    size: [f32; 2],
}

pub fn fingerprint(root: &Path) -> Result<String> {
    let mut hash = blake3::Hasher::new();
    hash.update(&VERSION.to_le_bytes());
    for name in [
        "geography.ron",
        "terrain_settings.ron",
        "provinces.png",
        "heightmap.png",
    ] {
        let mut file = fs::File::open(root.join("map_data").join(name))?;
        hash.update(name.as_bytes());
        hash.update(&file.metadata()?.len().to_le_bytes());
        let mut buffer = vec![0; 1024 * 1024];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
    }
    Ok(hash.finalize().to_hex().to_string())
}

pub fn read(
    path: &Path,
    source_hash: &str,
    province_count: usize,
) -> Result<(ProvinceRaster, Heightfield)> {
    let mut file = fs::File::open(path)?;
    let mut length = [0; 4];
    file.read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;
    ensure!(length < 16 * 1024 * 1024, "oversized cache header");
    let mut header = vec![0; length];
    file.read_exact(&mut header)?;
    let header: Header = serde_json::from_slice(&header)?;
    ensure!(
        header.version == VERSION
            && header.source_hash == source_hash
            && header.little_endian == cfg!(target_endian = "little"),
        "stale cache"
    );
    ensure!(
        header.width > 0 && header.height > 0 && header.columns > 0 && header.rows > 0,
        "empty cache planes"
    );
    let pixels = u64::from(header.width) * u64::from(header.height);
    let vertices = (u64::from(header.columns) + 1)
        .checked_mul(u64::from(header.rows) + 1)
        .ok_or_else(|| anyhow::anyhow!("cache dimensions overflow"))?;
    let bytes = pixels
        .checked_add(vertices)
        .and_then(|elements| elements.checked_mul(4))
        .ok_or_else(|| anyhow::anyhow!("cache dimensions overflow"))?;
    ensure!(bytes < MAX_BYTES, "oversized cache planes");
    ensure!(
        header.centroids.len() == province_count + 1
            && header
                .centroids
                .iter()
                .flatten()
                .all(|v| v.is_finite() && (0. ..=1.).contains(v)),
        "invalid cache centroids"
    );
    ensure!(
        header.size.iter().all(|v| v.is_finite() && *v > 0.),
        "invalid cached world size"
    );
    let mut payload = Vec::new();
    zstd::stream::read::Decoder::new(file)?
        .take(bytes + 1)
        .read_to_end(&mut payload)?;
    ensure!(
        payload.len() as u64 == bytes
            && blake3::hash(&payload).to_hex().as_str() == header.payload_hash,
        "corrupt cache payload"
    );
    let mut indices = vec![0u32; pixels as usize];
    let split = indices.len() * 4;
    bytemuck::cast_slice_mut(&mut indices).copy_from_slice(&payload[..split]);
    ensure!(
        indices
            .iter()
            .all(|i| *i > 0 && *i as usize <= province_count),
        "invalid cached province index"
    );
    let mut heights = vec![0f32; vertices as usize];
    bytemuck::cast_slice_mut(&mut heights).copy_from_slice(&payload[split..]);
    ensure!(
        heights.iter().all(|h| h.is_finite() && *h >= 0.),
        "invalid cached elevation"
    );
    Ok((
        ProvinceRaster {
            width: header.width,
            height: header.height,
            indices,
            centroids: header.centroids.into_iter().map(Vec2::from_array).collect(),
        },
        Heightfield {
            columns: header.columns,
            rows: header.rows,
            size: Vec2::from_array(header.size),
            heights,
        },
    ))
}

pub fn write(
    path: &Path,
    source_hash: &str,
    raster: &ProvinceRaster,
    terrain: &Heightfield,
) -> Result<()> {
    let mut payload = Vec::with_capacity((raster.indices.len() + terrain.heights.len()) * 4);
    payload.extend_from_slice(bytemuck::cast_slice(&raster.indices));
    payload.extend_from_slice(bytemuck::cast_slice(&terrain.heights));
    let header = serde_json::to_vec(&Header {
        version: VERSION,
        little_endian: cfg!(target_endian = "little"),
        source_hash: source_hash.into(),
        payload_hash: blake3::hash(&payload).to_hex().to_string(),
        width: raster.width,
        height: raster.height,
        centroids: raster.centroids.iter().map(|p| p.to_array()).collect(),
        columns: terrain.columns,
        rows: terrain.rows,
        size: terrain.size.to_array(),
    })?;
    fs::create_dir_all(path.parent().unwrap())?;
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::now_v7()));
    let result = (|| -> Result<()> {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(&(header.len() as u32).to_le_bytes())?;
        file.write_all(&header)?;
        let mut encoder = zstd::stream::write::Encoder::new(file, 1)?;
        encoder.write_all(&payload)?;
        encoder.finish()?.sync_all()?;
        // Same content key can be populated by another process while this one prepares it.
        if fs::rename(&temporary, path).is_err() && !path.exists() {
            anyhow::bail!("cannot install map cache");
        }
        Ok(())
    })();
    let _ = fs::remove_file(&temporary);
    result
}
