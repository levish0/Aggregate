use aggregate_economy::EconomyProgram;
use aggregate_simulation_core::Simulation;
use aggregate_world::{FacilityId, PopulationGroupId, ProvinceId, Scenario};
use std::sync::Arc;

/// Deterministic, asset-free workload. Setup and UUID parsing are not timed.
pub fn world(provinces: usize) -> Scenario {
    let mut scenario =
        aggregate_scenario::parse_scenario(include_str!("../../../../scenarios/foundation.json"))
            .unwrap();
    let province = scenario.initial_state.provinces[0].clone();
    let group = scenario.initial_state.population_groups[0].clone();
    let facility = scenario.initial_state.facilities[0].clone();
    scenario.initial_state.provinces.clear();
    scenario.initial_state.population_groups.clear();
    scenario.initial_state.facilities.clear();
    for index in 0..provinces {
        let province_id: ProvinceId = format!("10000000-0000-7000-8000-{index:012x}")
            .parse()
            .unwrap();
        let group_id: PopulationGroupId = format!("20000000-0000-7000-8000-{index:012x}")
            .parse()
            .unwrap();
        let facility_id: FacilityId = format!("30000000-0000-7000-8000-{index:012x}")
            .parse()
            .unwrap();
        let mut province = province.clone();
        province.id = province_id.clone();
        let mut group = group.clone();
        group.id = group_id;
        group.province = province_id.clone();
        let mut facility = facility.clone();
        facility.id = facility_id;
        facility.province = province_id;
        scenario.initial_state.provinces.push(province);
        scenario.initial_state.population_groups.push(group);
        scenario.initial_state.facilities.push(facility);
    }
    scenario
}

pub fn simulation(scenario: Scenario) -> Simulation {
    Simulation::from_scenario_with_programs(scenario, vec![Arc::new(EconomyProgram)]).unwrap()
}
