use aggregate_geography::{Heightfield, WorldMap};
use aggregate_world::CountryId;
use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};
use std::collections::BTreeMap;

pub struct PreparedLabel { pub name: String, pub anchor_x: f32, pub image: Image, pub mesh: Mesh }

pub fn prepare(map: &WorldMap, names: &BTreeMap<CountryId, String>, bytes: &[u8]) -> Vec<PreparedLabel> {
    let font = ab_glyph::FontRef::try_from_slice(bytes).expect("validated UI font");
    let mut countries = BTreeMap::<_, Vec<Vec2>>::new();
    for (index, province) in map.catalog.provinces.iter().enumerate() {
        if !province.water && let Some(country) = &province.owner {
            countries.entry(country).or_default().push(map.provinces.centroids[index + 1]);
        }
    }
    let mut prepared = Vec::new();
    for (country, points) in countries {
        let angle = points.iter().map(|point| Vec2::from_angle(point.x * std::f32::consts::TAU)).sum::<Vec2>().to_angle();
        let center = Vec2::new(angle.rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU, points.iter().map(|point| point.y).sum::<f32>() / points.len() as f32);
        let offset = |point: Vec2| Vec2::new((point.x - center.x + 0.5).rem_euclid(1.) - 0.5, point.y - center.y);
        let anchor = *points.iter().min_by(|left, right| offset(**left).length_squared().total_cmp(&offset(**right).length_squared())).unwrap();
        let spread = (points.iter().map(|point| offset(*point).x.powi(2)).sum::<f32>() / points.len() as f32).sqrt();
        let Some(name) = names.get(country) else { continue; };
        let image = super::typography::rasterize(&font, name);
        let width = (spread * map.terrain.size.x * 2.).max(2.);
        let height = width * image.height() as f32 / image.width() as f32;
        let anchor = anchor * map.terrain.size;
        prepared.push(PreparedLabel { name: name.clone(), anchor_x: anchor.x,
            mesh: surface_mesh(&map.terrain, anchor, Vec2::new(width, height)), image });
    }
    prepared
}

fn surface_mesh(terrain: &Heightfield, anchor: Vec2, size: Vec2) -> Mesh {
    let columns = (size.x / (terrain.size.x / terrain.columns as f32)).ceil().max(2.) as u32;
    let rows = (size.y / (terrain.size.y / terrain.rows as f32)).ceil().max(2.) as u32;
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    for row in 0..=rows {
        for column in 0..=columns {
            let uv = Vec2::new(column as f32 / columns as f32, row as f32 / rows as f32);
            let point = anchor + (uv - Vec2::splat(0.5)) * size;
            let grid = Vec2::new(point.x.rem_euclid(terrain.size.x) / terrain.size.x * terrain.columns as f32,
                (point.y / terrain.size.y).clamp(0., 1.) * terrain.rows as f32);
            let x = (grid.x.floor() as u32).min(terrain.columns - 1);
            let z = (grid.y.floor() as u32).min(terrain.rows - 1);
            let fraction = grid - Vec2::new(x as f32, z as f32);
            let a = terrain.position(x, z).y;
            let b = terrain.position(x + 1, z).y;
            let c = terrain.position(x, z + 1).y;
            let d = terrain.position(x + 1, z + 1).y;
            let height = if fraction.x + fraction.y <= 1. {
                a + (b - a) * fraction.x + (c - a) * fraction.y
            } else { d + (c - d) * (1. - fraction.x) + (b - d) * (1. - fraction.y) };
            positions.push([point.x, height + 0.12, point.y]);
            uvs.push(uv.to_array());
        }
    }
    for row in 0..rows { for column in 0..columns {
        let a = row * (columns + 1) + column;
        let c = a + columns + 1;
        indices.extend_from_slice(&[a, c, a + 1, a + 1, c, c + 1]);
    }}
    let normals = vec![[0., 1., 0.]; positions.len()];
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}
