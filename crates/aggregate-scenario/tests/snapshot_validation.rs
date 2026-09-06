use aggregate_scenario::{parse_scenario, validate_scenario, validate_world_state};
use aggregate_world::{
    ConstructionProjectState, FacilityDefinitionId, FacilityId, ProvinceId, Scenario,
};
use uuid::Uuid;

fn fixture() -> Scenario {
    parse_scenario(include_str!("../../../scenarios/foundation.json")).unwrap()
}

#[test]
fn managed_geographic_ids_reject_nil_and_duplicate_uuids() {
    let mut scenario = fixture();
    scenario.initial_state.countries[0].id = Uuid::nil().into();
    assert!(validate_scenario(&scenario).is_err());
    let mut scenario = fixture();
    scenario.initial_state.provinces[0].id = Uuid::nil().into();
    assert!(validate_scenario(&scenario).is_err());
    let mut scenario = fixture();
    scenario.initial_state.provinces[1].id = scenario.initial_state.provinces[0].id.clone();
    assert!(validate_scenario(&scenario).is_err());
    assert!("northern_province".parse::<ProvinceId>().is_err());
}

fn project() -> ConstructionProjectState {
    ConstructionProjectState {
        facility_id: FacilityId::from(Uuid::from_u128(10)),
        province: "01a07577-e209-792a-aba0-8dba97d92ac6"
            .parse::<ProvinceId>()
            .unwrap(),
        definition: FacilityDefinitionId::from("grain_farm"),
        requested_workers: 10,
        remaining_construction_points: 10,
        production_priority: 10,
    }
}

#[test]
fn save_snapshots_accept_progress_that_scenario_initial_states_reject() {
    let mut scenario = fixture();
    scenario.initial_state.day = 7;
    scenario.initial_state.construction_projects.push(project());
    validate_world_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
    )
    .unwrap();
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.day"
    );
    scenario.initial_state.day = 0;
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.construction_projects"
    );
}

#[test]
fn in_progress_projects_cannot_reuse_facility_ids_or_exceed_their_definition() {
    let mut scenario = fixture();
    let mut construction = project();
    construction.facility_id = scenario.initial_state.facilities[0].id.clone();
    scenario
        .initial_state
        .construction_projects
        .push(construction);
    let error = validate_world_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
    )
    .unwrap_err();
    assert_eq!(
        error.field_path,
        "state.construction_projects[0].facility_id"
    );

    scenario.initial_state.construction_projects[0] = project();
    scenario.initial_state.construction_projects[0].requested_workers = 11;
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

    scenario.initial_state.construction_projects[0] = project();
    scenario.initial_state.construction_projects[0].remaining_construction_points = 21;
    let error = validate_world_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
    )
    .unwrap_err();
    assert_eq!(
        error.field_path,
        "state.construction_projects[0].remaining_construction_points"
    );
}

#[test]
fn save_validation_also_checks_definitions_and_rules() {
    let mut scenario = fixture();
    scenario.definitions.facilities[0].workers_per_level = 0;
    let error = validate_world_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
    )
    .unwrap_err();
    assert_eq!(
        error.field_path,
        "definitions.facilities[0].workers_per_level"
    );
    let mut scenario = fixture();
    scenario.rules.consumption_per_person_day = 0;
    let error = validate_world_state(
        &scenario.definitions,
        &scenario.rules,
        &scenario.initial_state,
    )
    .unwrap_err();
    assert_eq!(error.field_path, "rules.consumption_per_person_day");
}

#[test]
fn normalization_includes_construction_project_ids() {
    let mut state = fixture().initial_state;
    let mut first = project();
    first.facility_id = FacilityId::from(Uuid::from_u128(16));
    let mut second = project();
    second.facility_id = FacilityId::from(Uuid::from_u128(32));
    state.construction_projects = vec![second.clone(), first.clone()];
    state.normalize();
    assert_eq!(state.construction_projects, vec![first, second]);
    let sorted = state.clone();
    state.normalize();
    assert_eq!(state, sorted);
}
