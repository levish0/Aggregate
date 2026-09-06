use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

/// Authored rendering scale, not elevation in physical meters.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainSettings {
    pub schema_version: u32,
    pub grid_columns: u32,
    pub world_width: f32,
    pub sea_level_raw: u16,
    pub height_scale: f32,
}

impl TerrainSettings {
    pub fn load(path: &Path) -> Result<Self> {
        let settings: Self = ron::from_str(&fs::read_to_string(path)?)?;
        settings.validate()?;
        Ok(settings)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == 1,
            "unsupported terrain settings schema"
        );
        ensure!(
            (32..=4096).contains(&self.grid_columns) && self.grid_columns.is_multiple_of(32),
            "terrain columns must be a multiple of 32 in 32..=4096"
        );
        ensure!(
            self.world_width.is_finite() && self.world_width > 0.,
            "invalid terrain width"
        );
        ensure!(
            self.height_scale.is_finite() && self.height_scale > 0.,
            "invalid terrain height scale"
        );
        Ok(())
    }
    pub fn elevation(&self, raw: u16, water: bool) -> f32 {
        if water {
            0.
        } else {
            (f32::from(raw.saturating_sub(self.sea_level_raw)) / 65535. * self.height_scale)
                .max(0.03)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_sea_level_removes_coastal_pedestal_and_invalid_dimensions_fail() {
        let mut settings = TerrainSettings {
            schema_version: 1,
            grid_columns: 1024,
            world_width: 2048.,
            sea_level_raw: 4884,
            height_scale: 40.,
        };
        settings.validate().unwrap();
        assert_eq!(settings.elevation(4884, false), 0.03);
        assert_eq!(settings.elevation(6000, true), 0.);
        assert!(settings.elevation(16000, false) > settings.elevation(6000, false));
        settings.grid_columns = 0;
        assert!(settings.validate().is_err());
    }
}
