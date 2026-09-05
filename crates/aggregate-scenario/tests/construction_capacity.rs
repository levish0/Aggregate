use aggregate_scenario::{parse_scenario, validate_world_state};
use aggregate_world::{
    ConstructionProjectState, FacilityDefinitionId, FacilityId, FacilityState, ProvinceId, Scenario,
};
use uuid::Uuid;

fn fixture_with_one_operating_farm(capacity: u64) -> Scenario {
    let mut scenario = parse_scenario(include_str!("../../../scenarios/foundation.json")).unwrap();
    scenario.initial_state.facilities.truncate(1);
    scenario.definitions.facilities[0].workers_per_level = capacity;
    scenario.definitions.facilities[1].construction.max_workers = 20;
    scenario.definitions.facilities[1].construction.worker_days = 20;
    scenario.definitions.facilities[2].construction.max_workers = 20;
    scenario.definitions.facilities[2].construction.worker_days = 20;
    scenario
}

fn project(
    identifier: u128,
    definition: &str,
    province: &str,
    requested_workers: u64,
    remaining_worker_days: u64,
) -> ConstructionProjectState {
    ConstructionProjectState {
        facility_id: FacilityId::from(Uuid::from_u128(identifier)),
        province: ProvinceId::from(province),
        definition: FacilityDefinitionId::from(definition),
        requested_workers,
        remaining_worker_days,
        production_priority: 10,
    }
}

#[test]
fn future_facility_capacity_is_checked_with_existing_province_and_country_capacity() {
    for province in ["north_valley", "south_ridge"] {
        let mut scenario = fixture_with_one_operating_farm(u64::MAX - 5);
        scenario.definitions.facilities[1].workers_per_level = 6;
        scenario.initial_state.construction_projects.push(project(
            10,
            "logging_camp",
            province,
            1,
            1,
        ));
        let error = validate_world_state(
            &scenario.definitions,
            &scenario.rules,
            &scenario.initial_state,
        )
        .unwrap_err();
        assert_eq!(
            error.field_path,
            "state.construction_projects[0].definition"
        );
    }
}

#[test]
fn future_capacity_accumulates_all_pending_projects() {
    let mut scenario = fixture_with_one_operating_farm(1);
    scenario.initial_state.facilities.clear();
    scenario.definitions.facilities[1].workers_per_level = u64::MAX / 2 + 1;
    scenario.initial_state.construction_projects = vec![
        project(10, "logging_camp", "north_valley", 1, 1),
        project(11, "logging_camp", "north_valley", 1, 1),
    ];
    let error = validate_world_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
    )
    .unwrap_err();
    assert_eq!(
        error.field_path,
        "state.construction_projects[1].definition"
    );
}

#[test]
fn current_construction_demand_is_checked_with_operating_capacity() {
    let mut scenario = fixture_with_one_operating_farm(u64::MAX - 5);
    scenario.definitions.facilities[1].workers_per_level = 1;
    scenario.initial_state.construction_projects.push(project(
        10,
        "logging_camp",
        "north_valley",
        6,
        6,
    ));
    let error = validate_world_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
    )
    .unwrap_err();
    assert_eq!(
        error.field_path,
        "state.construction_projects[0].requested_workers"
    );
}

#[test]
fn current_construction_demand_is_limited_by_remaining_work() {
    let mut scenario = fixture_with_one_operating_farm(u64::MAX - 5);
    scenario.definitions.facilities[1].workers_per_level = 1;
    scenario.initial_state.construction_projects.push(project(
        10,
        "logging_camp",
        "north_valley",
        20,
        5,
    ));
    validate_world_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
    )
    .unwrap();
}

#[test]
fn mixed_completion_states_must_fit_even_when_both_endpoints_fit() {
    let mut scenario = fixture_with_one_operating_farm(u64::MAX - 10);
    scenario.initial_state.facilities.clear();
    scenario.definitions.facilities[1].workers_per_level = 1;
    scenario.initial_state.construction_projects = vec![
        project(10, "grain_farm", "north_valley", 1, 1),
        project(11, "logging_camp", "north_valley", 20, 20),
    ];
    let error = validate_world_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
    )
    .unwrap_err();
    assert_eq!(error.field_path, "state.construction_projects[1]");
    assert!(error.message.contains("completion"));
}

#[test]
fn every_completion_subset_of_an_accepted_project_set_remains_valid() {
    let mut scenario = fixture_with_one_operating_farm(u64::MAX - 30);
    scenario.definitions.facilities[1].workers_per_level = 5;
    scenario.definitions.facilities[2].workers_per_level = 10;
    scenario.initial_state.construction_projects = vec![
        project(10, "logging_camp", "north_valley", 10, 10),
        project(11, "tool_workshop", "north_valley", 10, 10),
    ];
    for completed_mask in 0..4 {
        let mut state = scenario.initial_state.clone();
        state.construction_projects.clear();
        for (index, construction) in scenario
            .initial_state
            .construction_projects
            .iter()
            .enumerate()
        {
            if completed_mask & (1 << index) == 0 {
                state.construction_projects.push(construction.clone());
            } else {
                state.facilities.push(FacilityState {
                    id: construction.facility_id.clone(),
                    province: construction.province.clone(),
                    definition: construction.definition.clone(),
                    level: 1,
                    production_priority: construction.production_priority,
                });
            }
        }
        validate_world_state(&scenario.definitions, &scenario.rules, &state).unwrap();
    }
}
