use crate::GeographyCatalog;
use aggregate_world::{CountryId, StateId};
use std::collections::BTreeMap;

/// Dense rendering indices are derived; StateId is the persisted identity.
pub struct StateGeography {
    pub catalog_index: usize,
    pub province_indices: Vec<u32>,
    pub terrain_counts: BTreeMap<String, usize>,
}

pub struct AdministrativeIndex {
    pub states: BTreeMap<StateId, StateGeography>,
    pub countries: BTreeMap<CountryId, Vec<StateId>>,
    pub province_states: Vec<Option<StateId>>,
}

impl AdministrativeIndex {
    pub fn new(catalog: &GeographyCatalog) -> Self {
        let mut index = Self { states: BTreeMap::new(), countries: BTreeMap::new(), province_states: vec![None; catalog.provinces.len() + 1] };
        let mut groups = BTreeMap::new();
        for (position, state) in catalog.states.iter().enumerate() {
            groups.insert((&state.region, &state.country), &state.id);
            index.countries.entry(state.country.clone()).or_default().push(state.id.clone());
            index.states.insert(state.id.clone(), StateGeography { catalog_index: position, province_indices: Vec::new(), terrain_counts: BTreeMap::new() });
        }
        for (position, province) in catalog.provinces.iter().enumerate() {
            if province.water { continue; }
            let Some(state_id) = province.region.as_ref().zip(province.owner.as_ref()).and_then(|group| groups.get(&group)) else { continue; };
            let state = index.states.get_mut(*state_id).expect("catalog state indexed");
            state.province_indices.push(position as u32 + 1);
            *state.terrain_counts.entry(province.terrain.clone()).or_default() += 1;
            index.province_states[position + 1] = Some((*state_id).clone());
        }
        index
    }

    pub fn state_for_province(&self, index: u32) -> Option<&StateId> {
        self.province_states.get(index as usize).and_then(Option::as_ref)
    }
}
