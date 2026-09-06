use crate::Heightfield;
use glam::{Vec2, Vec3};

/// Walk only the crossed heightfield cells, intersecting the exact rendered triangles.
pub fn intersect(terrain: &Heightfield, origin: Vec3, direction: Vec3) -> Option<Vec3> {
    if !origin.is_finite() || !direction.is_finite() || direction.length_squared() < 1e-12 {
        return None;
    }
    let mut start: f32 = 0.;
    let mut end = f32::INFINITY;
    for (position, velocity, maximum) in [
        (origin.x, direction.x, terrain.size.x),
        (origin.z, direction.z, terrain.size.y),
    ] {
        if velocity.abs() < 1e-8 {
            if !(0. ..=maximum).contains(&position) {
                return None;
            }
        } else {
            let first = -position / velocity;
            let second = (maximum - position) / velocity;
            start = start.max(first.min(second));
            end = end.min(first.max(second));
        }
    }
    if start > end {
        return None;
    }
    let cell_size = terrain.size / Vec2::new(terrain.columns as f32, terrain.rows as f32);
    let point = origin + direction * start;
    let mut x = ((point.x / cell_size.x).floor() as i32).clamp(0, terrain.columns as i32 - 1);
    let mut z = ((point.z / cell_size.y).floor() as i32).clamp(0, terrain.rows as i32 - 1);
    let step_x = if direction.x >= 0. { 1 } else { -1 };
    let step_z = if direction.z >= 0. { 1 } else { -1 };
    let delta_x = if direction.x.abs() < 1e-8 {
        f32::INFINITY
    } else {
        cell_size.x / direction.x.abs()
    };
    let delta_z = if direction.z.abs() < 1e-8 {
        f32::INFINITY
    } else {
        cell_size.y / direction.z.abs()
    };
    let mut next_x = if delta_x.is_infinite() {
        f32::INFINITY
    } else {
        ((x + i32::from(step_x > 0)) as f32 * cell_size.x - origin.x) / direction.x
    };
    let mut next_z = if delta_z.is_infinite() {
        f32::INFINITY
    } else {
        ((z + i32::from(step_z > 0)) as f32 * cell_size.y - origin.z) / direction.z
    };
    for _ in 0..terrain.columns + terrain.rows + 4 {
        let a = terrain.position(x as u32, z as u32);
        let b = terrain.position(x as u32 + 1, z as u32);
        let c = terrain.position(x as u32, z as u32 + 1);
        let d = terrain.position(x as u32 + 1, z as u32 + 1);
        let limit = next_x.min(next_z).min(end);
        let hit = [
            triangle(origin, direction, a, c, b),
            triangle(origin, direction, b, c, d),
        ]
        .into_iter()
        .flatten()
        .filter(|distance| *distance >= start - 0.001 && *distance <= limit + 0.001)
        .min_by(f32::total_cmp);
        if let Some(distance) = hit {
            return Some(origin + direction * distance);
        }
        if limit >= end || limit.is_infinite() {
            break;
        }
        start = limit;
        if next_x <= limit {
            x += step_x;
            next_x += delta_x;
        }
        if next_z <= limit {
            z += step_z;
            next_z += delta_z;
        }
        if x < 0 || z < 0 || x >= terrain.columns as i32 || z >= terrain.rows as i32 {
            break;
        }
    }
    None
}

fn triangle(origin: Vec3, direction: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let edge1 = b - a;
    let edge2 = c - a;
    let perpendicular = direction.cross(edge2);
    let determinant = edge1.dot(perpendicular);
    if determinant.abs() < 1e-8 {
        return None;
    }
    let inverse = determinant.recip();
    let offset = origin - a;
    let u = offset.dot(perpendicular) * inverse;
    let cross = offset.cross(edge1);
    let v = direction.dot(cross) * inverse;
    let distance = edge2.dot(cross) * inverse;
    (u >= -1e-5 && v >= -1e-5 && u + v <= 1.00001 && distance >= 0.).then_some(distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ray_matches_sloped_mesh_instead_of_flat_plane() {
        let terrain = Heightfield {
            columns: 2,
            rows: 1,
            size: Vec2::new(2., 1.),
            heights: vec![0., 1., 2., 0., 1., 2.],
        };
        let hit = intersect(&terrain, Vec3::new(1.5, 5., 0.5), -Vec3::Y).unwrap();
        assert!((hit - Vec3::new(1.5, 1.5, 0.5)).length() < 0.001);
        assert!(intersect(&terrain, Vec3::new(-1., 5., 0.5), -Vec3::Y).is_none());
        let hit = intersect(&terrain, Vec3::new(-1., 3., 0.5), Vec3::new(1., -1., 0.)).unwrap();
        assert!((hit - Vec3::new(1., 1., 0.5)).length() < 0.001);
        assert!(intersect(&terrain, Vec3::new(1., 5., 0.5), Vec3::Y).is_none());
    }
}
