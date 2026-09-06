use aggregate_geography::WorldMap;
use aggregate_world::CountryId;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use std::collections::BTreeMap;

pub struct PreparedLabel {
    pub name: String,
    pub anchor_x: f32,
    pub image: Image,
    pub mesh: Mesh,
}

pub fn prepare(
    map: &WorldMap,
    names: &BTreeMap<CountryId, String>,
    bytes: &[u8],
) -> Vec<PreparedLabel> {
    let font = ab_glyph::FontRef::try_from_slice(bytes).expect("validated UI font");
    let mut countries = BTreeMap::<_, Vec<Vec2>>::new();
    for (index, province) in map.catalog.provinces.iter().enumerate() {
        if !province.water
            && let Some(country) = &province.owner
        {
            countries
                .entry(country)
                .or_default()
                .push(map.provinces.centroids[index + 1]);
        }
    }
    let mut prepared = Vec::new();
    for (country, points) in countries {
        let points = largest_land_cluster(points);
        let angle = points
            .iter()
            .map(|point| Vec2::from_angle(point.x * std::f32::consts::TAU))
            .sum::<Vec2>()
            .to_angle();
        let center = Vec2::new(
            angle.rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU,
            points.iter().map(|point| point.y).sum::<f32>() / points.len() as f32,
        );
        let offset = |point: Vec2| {
            Vec2::new(
                (point.x - center.x + 0.5).rem_euclid(1.) - 0.5,
                point.y - center.y,
            )
        };
        let Some(name) = names.get(country) else {
            continue;
        };
        let image = super::typography::rasterize(&font, name);
        let aspect = image.width() as f32 / image.height() as f32;
        let mut candidates = points.clone();
        candidates.sort_by(|left, right| {
            offset(*left)
                .length_squared()
                .total_cmp(&offset(*right).length_squared())
        });
        let central_count = candidates.len().min(12);
        let mut anchors = candidates[..central_count].to_vec();
        anchors.extend(points.iter().step_by((points.len() / 24).max(1)).copied());
        let extent = points
            .iter()
            .map(|point| (offset(*point) * map.terrain.size).length())
            .fold(0., f32::max)
            * 2.;
        let mut best = None;
        let mut best_score = 0.;
        for anchor in anchors {
            let anchor = anchor * map.terrain.size;
            for degrees in [0_f32, -20., 20., -40., 40., -65., 65., -80., 80.] {
                let angle = degrees.to_radians();
                let mut lower = 0.;
                let mut upper = extent.max(2.);
                for _ in 0..9 {
                    let width = (lower + upper) * 0.5;
                    let size = Vec2::new(width, width / aspect);
                    if fits_country(map, country, anchor, size, angle) {
                        lower = width;
                    } else {
                        upper = width;
                    }
                }
                let score = lower * (1. - degrees.abs() / 500.);
                if score > best_score {
                    best_score = score;
                    best = Some((anchor, Vec2::new(lower, lower / aspect) * 0.92, angle));
                }
            }
        }
        if let Some((anchor, size, angle)) = best {
            prepared.push(PreparedLabel {
                name: name.clone(),
                anchor_x: anchor.x,
                mesh: lettering_mesh(anchor, size, angle),
                image,
            });
        }
    }
    prepared
}

// Group nearby land before measuring a name: distant overseas holdings must not stretch it.
fn largest_land_cluster(points: Vec<Vec2>) -> Vec<Vec2> {
    let mut cells = BTreeMap::<(i32, i32), Vec<Vec2>>::new();
    for point in points {
        cells
            .entry(((point.x * 64.) as i32, (point.y * 64.) as i32))
            .or_default()
            .push(point);
    }
    let mut largest = Vec::new();
    while let Some((&first, _)) = cells.first_key_value() {
        let mut pending = vec![first];
        let mut cluster = Vec::new();
        while let Some((x, y)) = pending.pop() {
            let Some(points) = cells.remove(&(x, y)) else {
                continue;
            };
            cluster.extend(points);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let neighbor = ((x + dx).rem_euclid(64), y + dy);
                    if cells.contains_key(&neighbor) {
                        pending.push(neighbor);
                    }
                }
            }
        }
        if cluster.len() > largest.len() {
            largest = cluster;
        }
    }
    largest
}

fn rotate(point: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(point.x * cos - point.y * sin, point.x * sin + point.y * cos)
}

fn fits_country(map: &WorldMap, country: &CountryId, anchor: Vec2, size: Vec2, angle: f32) -> bool {
    for row in 0..=4 {
        for column in 0..=12 {
            let local = (Vec2::new(column as f32 / 12., row as f32 / 4.) - Vec2::splat(0.5)) * size;
            let mut uv = (anchor + rotate(local, angle)) / map.terrain.size;
            uv.x = uv.x.rem_euclid(1.);
            let Some(index) = map.provinces.index_at_uv(uv) else {
                return false;
            };
            if index == 0 {
                return false;
            }
            let province = &map.catalog.provinces[index as usize - 1];
            if province.water || province.owner.as_ref() != Some(country) {
                return false;
            }
        }
    }
    true
}

fn lettering_mesh(anchor: Vec2, size: Vec2, angle: f32) -> Mesh {
    let uvs = vec![[0., 0.], [1., 0.], [0., 1.], [1., 1.]];
    let positions: Vec<_> = uvs
        .iter()
        .map(|uv| {
            let point = anchor + rotate((Vec2::from_array(*uv) - Vec2::splat(0.5)) * size, angle);
            [point.x, 0.2, point.y]
        })
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 4])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(vec![0, 2, 1, 1, 2, 3]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overseas_holdings_do_not_stretch_mainland_lettering() {
        let mainland = vec![
            Vec2::new(0.48, 0.30),
            Vec2::new(0.49, 0.30),
            Vec2::new(0.50, 0.31),
        ];
        let mut holdings = mainland.clone();
        holdings.push(Vec2::new(0.82, 0.72));
        assert_eq!(largest_land_cluster(holdings).len(), mainland.len());
        assert_eq!(
            largest_land_cluster(vec![Vec2::new(0.999, 0.4), Vec2::new(0.001, 0.4)]).len(),
            2
        );
    }
}
