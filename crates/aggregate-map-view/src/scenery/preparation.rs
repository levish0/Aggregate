use super::{
    PreparedScenery,
    geometry::{SceneryGeometry, ground},
};
use aggregate_geography::WorldMap;
use bevy::prelude::*;
use std::{collections::BTreeMap, path::Path};

const CHUNK_SIZE: f32 = 64.;
type Chunks = BTreeMap<(i32, i32), SceneryGeometry>;

fn records(root: &Path, name: &str, magic: &[u8; 4]) -> Result<Vec<[f32; 4]>, String> {
    let bytes = std::fs::read(root.join("gfx/map/derived").join(name))
        .map_err(|error| format!("{name}: {error}"))?;
    if bytes.len() < 12 || &bytes[..4] != magic || bytes[4..8] != 1u32.to_le_bytes() {
        return Err(format!("Invalid scenery header: {name}"));
    }
    let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if bytes.len() - 12 != count * 16 {
        return Err(format!("Invalid scenery length: {name}"));
    }
    bytes[12..]
        .chunks_exact(16)
        .map(|record| {
            let values = std::array::from_fn(|i| {
                f32::from_le_bytes(record[i * 4..i * 4 + 4].try_into().unwrap())
            });
            if values.iter().all(|value| value.is_finite()) {
                Ok(values)
            } else {
                Err(format!("Non-finite scenery: {name}"))
            }
        })
        .collect()
}

fn land(map: &WorldMap, position: Vec2) -> bool {
    map.provinces
        .index_at_uv(position / map.terrain.size)
        .and_then(|index| map.catalog.provinces.get(index.checked_sub(1)? as usize))
        .is_some_and(|province| !province.water)
}

fn chunk(chunks: &mut Chunks, position: Vec2) -> &mut SceneryGeometry {
    chunks
        .entry((
            (position.x / CHUNK_SIZE) as i32,
            (position.y / CHUNK_SIZE) as i32,
        ))
        .or_default()
}

fn finish(chunks: Chunks) -> Vec<(Vec2, Mesh)> {
    chunks
        .into_iter()
        .map(|((x, z), geometry)| {
            (
                Vec2::new(x as f32 + 0.5, z as f32 + 0.5) * CHUNK_SIZE,
                geometry.into_mesh(),
            )
        })
        .collect()
}

pub fn prepare(root: &Path, map: &WorldMap) -> Result<PreparedScenery, String> {
    let started = std::time::Instant::now();
    let mut forests = Chunks::new();
    let mut tree_count = 0;
    for [u, v, height, kind] in records(root, "forest_instances.bin", b"AGTF")? {
        let position = Vec2::new(u, v) * map.terrain.size;
        if !(0. ..1.).contains(&u) || !(0. ..1.).contains(&v) || !land(map, position) {
            continue;
        }
        let base = ground(&map.terrain, position);
        let radius = height * if kind < 0.5 { 0.38 } else { 0.65 };
        let geometry = chunk(&mut forests, position);
        let color = if kind < 0.5 {
            [0.035, 0.070, 0.020, 1.]
        } else {
            [0.065, 0.115, 0.028, 1.]
        };
        // Small map-scale canopy meshes, sharing authored placements; no entity per tree.
        for face in 0..6 {
            let first = face as f32 * std::f32::consts::TAU / 6.;
            let next = (face + 1) as f32 * std::f32::consts::TAU / 6.;
            let a = base + Vec3::new(first.cos() * radius, height * 0.16, first.sin() * radius);
            let b = base + Vec3::new(next.cos() * radius, height * 0.16, next.sin() * radius);
            geometry.triangle([a, base + Vec3::Y * height, b], [Vec2::ZERO; 3], color);
        }
        tree_count += 1;
    }
    let mut roads = Chunks::new();
    let mut road_count = 0;
    for [u, v, end_u, end_v] in records(root, "road_segments.bin", b"AGRD")? {
        let start = Vec2::new(u, v) * map.terrain.size;
        let end = Vec2::new(end_u, end_v) * map.terrain.size;
        let length = start.distance(end);
        // Naval routes in the shared spline source are not roads.
        if !(0.0001..20.).contains(&length) {
            continue;
        }
        let steps = (length / 0.4).ceil() as u32;
        if (0..=steps).any(|step| !land(map, start.lerp(end, step as f32 / steps as f32))) {
            continue;
        }
        let offset = (end - start).normalize().perp() * 0.10;
        let geometry = chunk(&mut roads, (start + end) * 0.5);
        for step in 0..steps {
            let a = start.lerp(end, step as f32 / steps as f32);
            let b = start.lerp(end, (step + 1) as f32 / steps as f32);
            let points = [a - offset, a + offset, b - offset, b + offset]
                .map(|point| ground(&map.terrain, point) + Vec3::Y * 0.018);
            let u0 = length * step as f32 / steps as f32;
            let u1 = length * (step + 1) as f32 / steps as f32;
            geometry.triangle(
                [points[0], points[1], points[2]],
                [Vec2::new(u0, 0.), Vec2::new(u0, 1.), Vec2::new(u1, 0.)],
                [1.; 4],
            );
            geometry.triangle(
                [points[2], points[1], points[3]],
                [Vec2::new(u1, 0.), Vec2::new(u0, 1.), Vec2::new(u1, 1.)],
                [1.; 4],
            );
        }
        road_count += 1;
    }
    tracing::info!(
        tree_count,
        road_count,
        forest_chunks = forests.len(),
        road_chunks = roads.len(),
        elapsed_ms = started.elapsed().as_millis(),
        "Map scenery prepared"
    );
    Ok(PreparedScenery {
        forests: finish(forests),
        roads: finish(roads),
    })
}
