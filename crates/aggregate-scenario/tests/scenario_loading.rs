use aggregate_scenario::{load_scenario, parse_scenario, validate_scenario};
use aggregate_world::{FacilityId, GoodId, PopulationGroupId, ProvinceId, Scenario};
use std::path::Path;
use uuid::Uuid;

const FOUNDATION: &str = include_str!("../../../scenarios/foundation.json");

fn fixture() -> Scenario {
    parse_scenario(FOUNDATION).unwrap()
}

#[test]
fn foundation_loads_with_normalized_persistent_ids() {
    let scenario = load_scenario(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/foundation.json"),
    )
    .unwrap();
    assert_eq!(scenario, fixture());
    assert_eq!(scenario.initial_state.day, 0);
    assert_eq!(scenario.initial_state.countries.len(), 1);
    assert_eq!(scenario.initial_state.provinces.len(), 2);
    assert_eq!(
        scenario
            .initial_state
            .population_groups
            .iter()
            .map(|group| group.population)
            .sum::<u64>(),
        100
    );
    assert_eq!(
        scenario
            .initial_state
            .population_groups
            .iter()
            .map(|group| group.workforce)
            .sum::<u64>(),
        50
    );
}

#[test]
fn ordering_in_the_file_does_not_change_the_loaded_snapshot() {
    let expected = fixture();
    let mut shuffled = expected.clone();
    shuffled.initial_state.countries.reverse();
    shuffled.initial_state.provinces.reverse();
    shuffled.initial_state.population_groups.reverse();
    shuffled.initial_state.facilities.reverse();
    let loaded = parse_scenario(&serde_json::to_string(&shuffled).unwrap()).unwrap();
    assert_eq!(loaded.initial_state, expected.initial_state);
}

#[test]
fn unknown_json_fields_and_wrong_types_report_their_location() {
    let unknown = FOUNDATION.replacen("\"workers_per_level\"", "\"workers_per_levle\"", 1);
    let error = parse_scenario(&unknown).unwrap_err();
    assert_eq!(
        error.field_path,
        "definitions.facilities[0].workers_per_levle"
    );
    assert!(error.message.contains("workers_per_levle"));

    let mut wrong_type: serde_json::Value = serde_json::from_str(FOUNDATION).unwrap();
    wrong_type["initial_state"]["population_groups"][0]["workforce"] = "twenty".into();
    let error = parse_scenario(&wrong_type.to_string()).unwrap_err();
    assert_eq!(
        error.field_path,
        "initial_state.population_groups[0].workforce"
    );
}

#[test]
fn trailing_json_and_unsupported_schema_are_rejected() {
    let error = parse_scenario(&format!("{FOUNDATION} {{}}")).unwrap_err();
    assert_eq!(error.field_path, "$");
    let mut scenario = fixture();
    scenario.schema_version = 999;
    let error = validate_scenario(&scenario).unwrap_err();
    assert_eq!(error.field_path, "schema_version");
    assert!(error.message.contains("expected 2"));
}

#[test]
fn duplicate_ids_and_blank_names_are_rejected_at_the_bad_entry() {
    let mut scenario = fixture();
    scenario
        .definitions
        .goods
        .push(scenario.definitions.goods[0].clone());
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "definitions.goods[3].id"
    );

    let mut scenario = fixture();
    scenario.initial_state.provinces[1].name = " \n".to_owned();
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.provinces[1].name"
    );

    let mut scenario = fixture();
    scenario.id = " foundation".to_owned();
    assert_eq!(validate_scenario(&scenario).unwrap_err().field_path, "id");
}

#[test]
fn unknown_state_and_recipe_references_are_rejected() {
    let mut scenario = fixture();
    scenario.initial_state.population_groups[0].province = ProvinceId::from(Uuid::from_u128(999));
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.population_groups[0].province"
    );

    let mut scenario = fixture();
    scenario.definitions.facilities[0]
        .inputs_per_worker_day
        .insert(GoodId::from("missing"), 1);
    let error = validate_scenario(&scenario).unwrap_err();
    assert_eq!(
        error.field_path,
        "definitions.facilities[0].inputs_per_worker_day[\"missing\"]"
    );

    let mut scenario = fixture();
    scenario.initial_state.provinces[0]
        .stockpile
        .insert(GoodId::from("missing"), 0);
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.provinces[0].stockpile[\"missing\"]"
    );
}

#[test]
fn workforce_and_zero_quantities_are_validated() {
    let mut scenario = fixture();
    scenario.initial_state.population_groups[0].workforce = 41;
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.population_groups[0].workforce"
    );

    let mut scenario = fixture();
    scenario.definitions.facilities[0]
        .outputs_per_worker_day
        .insert(GoodId::from("grain"), 0);
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "definitions.facilities[0].outputs_per_worker_day[\"grain\"]"
    );

    let mut scenario = fixture();
    scenario.definitions.facilities[0].construction.construction_points = 0;
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "definitions.facilities[0].construction.construction_points"
    );
}

#[test]
fn arithmetic_overflow_is_a_field_error() {
    let mut scenario = fixture();
    scenario.initial_state.population_groups[0].population = u64::MAX;
    scenario.rules.consumption_per_person_day = 2;
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.population_groups[0].population"
    );

    let mut scenario = fixture();
    scenario.initial_state.population_groups[0].population = u64::MAX;
    let error = validate_scenario(&scenario).unwrap_err();
    assert_eq!(
        error.field_path,
        "initial_state.population_groups[1].population"
    );
    assert!(error.message.contains("aggregate"));

    let mut scenario = fixture();
    scenario.initial_state.facilities[0].level = u64::MAX;
    let error = validate_scenario(&scenario).unwrap_err();
    assert_eq!(error.field_path, "initial_state.facilities[0].level");
    assert!(error.message.contains("worker capacity"));
}

#[test]
fn file_errors_preserve_the_requested_path() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/nonexistent-scenario.fixture.json");
    let error = load_scenario(&path).unwrap_err();
    assert_eq!(error.source_path.as_deref(), Some(path.as_path()));
    assert!(error.to_string().starts_with(&path.display().to_string()));
}

#[test]
fn instance_identifiers_must_be_valid_non_nil_unique_uuids() {
    let invalid = FOUNDATION.replacen("00000000-0000-0000-0000-000000000065", "north_residents", 1);
    assert_eq!(
        parse_scenario(&invalid).unwrap_err().field_path,
        "initial_state.population_groups[0].id"
    );

    let mut scenario = fixture();
    scenario.initial_state.population_groups[0].id = PopulationGroupId::from(Uuid::nil());
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.population_groups[0].id"
    );

    let mut scenario = fixture();
    scenario.initial_state.facilities[0].id = FacilityId::from(Uuid::nil());
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.facilities[0].id"
    );

    let mut scenario = fixture();
    scenario.initial_state.facilities[1].id = scenario.initial_state.facilities[0].id.clone();
    assert_eq!(
        validate_scenario(&scenario).unwrap_err().field_path,
        "initial_state.facilities[1].id"
    );
}
