use crate::GeographyCatalog;
use anyhow::{Result, ensure};
use glam::Vec2;
use std::{collections::HashMap, path::Path};

/// Dense indices are ephemeral GPU/CPU lookup addresses, never persistent entity IDs.
pub struct ProvinceRaster {
    pub width: u32,
    pub height: u32,
    pub indices: Vec<u32>,
    pub centroids: Vec<Vec2>,
}

impl ProvinceRaster {
    pub fn load(path: &Path, catalog: &GeographyCatalog) -> Result<Self> {
        let image = image::open(path)?.into_rgb8();
        let width = image.width();
        let height = image.height();
        ensure!(width > 0 && height > 0, "empty province raster");
        let colors: HashMap<_, _> = catalog
            .provinces
            .iter()
            .enumerate()
            .map(|(index, province)| (province.raster_color, index as u32 + 1))
            .collect();
        let mut sums = vec![(0u64, 0u64, 0u64); catalog.provinces.len() + 1];
        let mut indices = Vec::with_capacity((width * height) as usize);
        for (x, y, pixel) in image.enumerate_pixels() {
            let color = u32::from(pixel[0]) << 16 | u32::from(pixel[1]) << 8 | u32::from(pixel[2]);
            let index = *colors.get(&color).ok_or_else(|| {
                anyhow::anyhow!("unregistered raster color {color:06x} at {x},{y}")
            })?;
            indices.push(index);
            let sum = &mut sums[index as usize];
            sum.0 += u64::from(x);
            sum.1 += u64::from(y);
            sum.2 += 1;
        }
        ensure!(
            sums.iter().skip(1).all(|sum| sum.2 > 0),
            "catalog province missing from raster"
        );
        let centroids = sums
            .into_iter()
            .map(|(x, y, count)| {
                if count == 0 {
                    Vec2::ZERO
                } else {
                    Vec2::new(
                        (x as f32 / count as f32 + 0.5) / width as f32,
                        (y as f32 / count as f32 + 0.5) / height as f32,
                    )
                }
            })
            .collect();
        Ok(Self {
            width,
            height,
            indices,
            centroids,
        })
    }

    pub fn index_at_uv(&self, uv: Vec2) -> Option<u32> {
        if !uv.is_finite() || uv.x < 0. || uv.x > 1. || uv.y < 0. || uv.y > 1. {
            return None;
        }
        let x = ((uv.x * self.width as f32) as u32).min(self.width - 1);
        let y = ((uv.y * self.height as f32) as u32).min(self.height - 1);
        self.indices.get((y * self.width + x) as usize).copied()
    }
}
