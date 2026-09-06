use crate::{GeographyCatalog, ProvinceRaster, TerrainSettings};
use anyhow::Result;
use glam::{Vec2, Vec3};
use std::path::Path;

pub struct Heightfield {
    pub columns: u32,
    pub rows: u32,
    pub size: Vec2,
    pub heights: Vec<f32>,
}

impl Heightfield {
    pub fn load(
        path: &Path,
        provinces: &ProvinceRaster,
        catalog: &GeographyCatalog,
        settings: &TerrainSettings,
    ) -> Result<Self> {
        let image = image::open(path)?.into_luma16();
        settings.validate()?;
        let columns = settings.grid_columns;
        let rows = ((columns as f32 * provinces.height as f32 / provinces.width as f32).ceil()
            as u32)
            .div_ceil(32)
            * 32;
        let size = Vec2::new(
            settings.world_width,
            settings.world_width * provinces.height as f32 / provinces.width as f32,
        );
        let mut heights = Vec::with_capacity(((columns + 1) * (rows + 1)) as usize);
        for row in 0..=rows {
            for column in 0..=columns {
                let uv = Vec2::new(column as f32 / columns as f32, row as f32 / rows as f32);
                let x = (uv.x * (image.width() - 1) as f32).round() as u32;
                let y = (uv.y * (image.height() - 1) as f32).round() as u32;
                let index = provinces.index_at_uv(uv).unwrap() as usize;
                let water = catalog.provinces[index - 1].water;
                let height = settings.elevation(image.get_pixel(x, y)[0], water);
                heights.push(height);
            }
        }
        // Both render copies meet at identical seam vertices.
        for row in 0..=rows {
            let first = (row * (columns + 1)) as usize;
            let last = first + columns as usize;
            let height = (heights[first] + heights[last]) * 0.5;
            heights[first] = height;
            heights[last] = height;
        }
        Ok(Self {
            columns,
            rows,
            size,
            heights,
        })
    }

    pub fn position(&self, x: u32, z: u32) -> Vec3 {
        Vec3::new(
            x as f32 / self.columns as f32 * self.size.x,
            self.heights[(z * (self.columns + 1) + x) as usize],
            z as f32 / self.rows as f32 * self.size.y,
        )
    }

    pub fn normal(&self, x: u32, z: u32) -> Vec3 {
        let horizontal =
            self.position((x + 1).min(self.columns), z) - self.position(x.saturating_sub(1), z);
        let vertical =
            self.position(x, (z + 1).min(self.rows)) - self.position(x, z.saturating_sub(1));
        vertical.cross(horizontal).normalize_or_zero()
    }

    pub fn intersect_ray(&self, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        crate::picking::intersect(self, origin, direction)
    }

    /// Match the five shared render copies, returning the hit in displayed world coordinates.
    pub fn intersect_wrapped_ray(&self, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        (-2..=2)
            .filter_map(|copy| {
                let offset = Vec3::X * (copy as f32 * self.size.x);
                self.intersect_ray(origin - offset, direction)
                    .map(|point| point + offset)
            })
            .min_by(|a, b| {
                a.distance_squared(origin)
                    .total_cmp(&b.distance_squared(origin))
            })
    }
}
