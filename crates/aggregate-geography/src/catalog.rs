use aggregate_world::{CountryId, ProvinceId, RegionId, StateId};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeographyCatalog {
    pub schema_version: u32,
    pub countries: Vec<MapCountry>,
    pub regions: Vec<MapRegion>,
    pub states: Vec<MapState>,
    pub provinces: Vec<MapProvince>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapCountry {
    pub id: CountryId,
    pub key: String,
    pub color: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapRegion {
    pub id: RegionId,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapProvince {
    pub id: ProvinceId,
    pub raster_color: u32,
    pub region: Option<RegionId>,
    pub owner: Option<CountryId>,
    pub terrain: String,
    pub water: bool,
}

/// Initial administrative ownership. Runtime transfer mechanisms must preserve IDs.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapState {
    pub id: StateId,
    pub region: RegionId,
    pub country: CountryId,
}

impl GeographyCatalog {
    pub fn load(path: &Path) -> Result<Self> {
        let catalog: Self = ron::from_str(&fs::read_to_string(path)?)
            .with_context(|| path.display().to_string())?;
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == 2, "unsupported geography schema");
        ensure!(
            !self.provinces.is_empty(),
            "geography contains no provinces"
        );
        let mut identities = BTreeSet::new();
        for id in self
            .countries
            .iter()
            .map(|item| item.id.0)
            .chain(self.regions.iter().map(|item| item.id.0))
            .chain(self.provinces.iter().map(|item| item.id.0))
            .chain(self.states.iter().map(|item| item.id.0))
        {
            ensure!(
                !id.is_nil() && identities.insert(id),
                "nil or duplicate geography UUID: {id}"
            );
        }
        let countries: BTreeSet<_> = self.countries.iter().map(|item| &item.id).collect();
        let regions: BTreeSet<_> = self.regions.iter().map(|item| &item.id).collect();
        let mut colors = BTreeSet::new();
        let mut country_keys = BTreeSet::new();
        let mut region_keys = BTreeSet::new();
        let mut state_groups = BTreeSet::new();
        for state in &self.states {
            ensure!(countries.contains(&state.country) && regions.contains(&state.region), "unknown state country or region");
            ensure!(state_groups.insert((&state.region, &state.country)), "duplicate country portion of a region");
        }
        let mut occupied_groups = BTreeSet::new();
        for country in &self.countries {
            ensure!(country_keys.insert(&country.key), "duplicate country key");
            ensure!(
                country
                    .color
                    .iter()
                    .all(|channel| channel.is_finite() && (0.0..=1.0).contains(channel)),
                "invalid country color"
            );
        }
        for region in &self.regions {
            ensure!(region_keys.insert(&region.key), "duplicate region key");
        }
        for province in &self.provinces {
            ensure!(
                province.raster_color <= 0xffffff && colors.insert(province.raster_color),
                "invalid or duplicate province color"
            );
            ensure!(
                province
                    .owner
                    .as_ref()
                    .is_none_or(|owner| countries.contains(owner)),
                "unknown province owner"
            );
            ensure!(
                province
                    .region
                    .as_ref()
                    .is_none_or(|region| regions.contains(region)),
                "unknown province region"
            );
            if !province.water && let (Some(region), Some(owner)) = (&province.region, &province.owner) {
                ensure!(state_groups.contains(&(region, owner)), "owned land province has no administrative state");
                occupied_groups.insert((region, owner));
            }
        }
        ensure!(occupied_groups == state_groups, "administrative state contains no land provinces");
        Ok(())
    }
}
