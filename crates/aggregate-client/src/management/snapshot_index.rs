use aggregate_world::{CountryId, ProvinceId, WorldSnapshot};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct CountryStatistics {
    pub population: u128,
    pub workforce: u128,
    pub province_indices: Vec<usize>,
}

/// Built alongside a committed snapshot on the simulation worker, never per UI row.
#[derive(Default)]
pub struct SnapshotIndex {
    pub provinces: BTreeMap<ProvinceId, usize>,
    pub countries: BTreeMap<CountryId, CountryStatistics>,
}

impl SnapshotIndex {
    pub fn build(snapshot: &WorldSnapshot) -> Self {
        let mut index = Self::default();
        for (position, province) in snapshot.provinces.iter().enumerate() {
            index.provinces.insert(province.id.clone(), position);
            index.countries.entry(province.country.clone()).or_default().province_indices.push(position);
        }
        for group in &snapshot.population_groups {
            if let Some(position) = index.provinces.get(&group.province) {
                let country = &snapshot.provinces[*position].country;
                let totals = index.countries.get_mut(country).expect("indexed country");
                totals.population += u128::from(group.population);
                totals.workforce += u128::from(group.workforce);
            }
        }
        index
    }
}
