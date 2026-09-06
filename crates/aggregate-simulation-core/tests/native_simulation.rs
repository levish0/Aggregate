use aggregate_scenario::parse_scenario;
use aggregate_simulation_core::{
    GoodsFlowCause, RecordedCommand, Simulation, SimulationCommand, SimulationEvent,
};
use aggregate_world::{FacilityId, FacilityState, GoodId, ProvinceId, Scenario, WorldSnapshot};
use std::collections::BTreeMap;

fn scenario() -> Scenario {
    parse_scenario(include_str!("../../../scenarios/foundation.json")).unwrap()
}
fn facility_id(value: u64) -> FacilityId {
    format!("00000000-0000-0000-0000-{value:012x}")
        .parse()
        .unwrap()
}
fn build_command() -> SimulationCommand {
    SimulationCommand::StartConstruction {
        country: "01a07577-e203-76f4-8315-dae6c69b13bb".parse().unwrap(),
        province: "01a07577-e209-792a-aba0-8dba97d92ac6".parse().unwrap(),
        facility: facility_id(100),
        definition: "grain_farm".into(),
        workers: 10,
        production_priority: 20,
    }
}
fn stockpiles(snapshot: &WorldSnapshot) -> BTreeMap<(ProvinceId, GoodId), i128> {
    snapshot
        .provinces
        .iter()
        .flat_map(|province| {
            province
                .stockpile
                .iter()
                .map(|(good, amount)| ((province.id.clone(), good.clone()), i128::from(*amount)))
        })
        .collect()
}

#[test]
fn workers_blocked_by_production_inputs_finish_waiting_construction_without_double_work() {
    let mut fixture = scenario();
    let north = fixture.initial_state.provinces[0].id.clone();
    fixture.initial_state.provinces[0]
        .stockpile
        .insert("timber".into(), 0);
    fixture.initial_state.population_groups[0].workforce = 2;
    fixture.initial_state.facilities.retain(|facility| {
        facility.province != north || facility.definition == "tool_workshop".into()
    });
    fixture
        .initial_state
        .facilities
        .iter_mut()
        .find(|facility| facility.province == north)
        .unwrap()
        .level = 2;
    let recipe = fixture
        .definitions
        .facilities
        .iter_mut()
        .find(|definition| definition.id == "grain_farm".into())
        .unwrap();
    recipe.construction.goods.clear();
    recipe.construction.worker_days = 1;
    recipe.construction.max_workers = 1;
    let mut simulation = Simulation::from_scenario(fixture).unwrap();
    for number in [100, 101] {
        let mut command = build_command();
        let SimulationCommand::StartConstruction {
            facility, workers, ..
        } = &mut command;
        *facility = facility_id(number);
        *workers = 1;
        simulation.execute(command).unwrap();
    }
    let report = simulation.step().unwrap();
    let province = report
        .provinces
        .iter()
        .find(|province| province.province == north)
        .unwrap();
    assert_eq!(province.production_workers, 0);
    assert_eq!(
        province.construction_workers, 2,
        "idle workers must not remain reserved at an input-starved factory"
    );
    assert_eq!(
        province.production_workers + province.construction_workers,
        province.available_workers
    );
    assert!(simulation.snapshot().construction_projects.is_empty());
    assert!(
        report
            .facilities
            .iter()
            .filter(|facility| facility.facility == facility_id(3))
            .all(|facility| facility.outputs.values().all(|amount| *amount == 0))
    );
    let mut resumed = Simulation::from_save_json(&simulation.save_json().unwrap()).unwrap();
    assert_eq!(resumed.step().unwrap(), simulation.step().unwrap());
}

#[test]
fn production_and_consumption_have_explicit_balanced_goods_flows() {
    let mut simulation = Simulation::from_scenario(scenario()).unwrap();
    let before = simulation.snapshot();
    let report = simulation.step().unwrap();
    let after = simulation.snapshot();
    assert_eq!(report.day, 1);
    assert_eq!(before.population_groups, after.population_groups);
    let mut expected = stockpiles(&before);
    for flow in &report.goods_flows {
        let delta = match flow.cause {
            GoodsFlowCause::ProductionOutput { .. } => i128::from(flow.amount),
            _ => -i128::from(flow.amount),
        };
        *expected
            .entry((flow.province.clone(), flow.good.clone()))
            .or_default() += delta;
    }
    assert_eq!(expected, stockpiles(&after));
    for province in &report.provinces {
        assert!(
            province.production_workers + province.construction_workers
                <= province.available_workers
        );
        assert_eq!(
            province.food_required,
            province.food_consumed + province.food_shortfall
        );
    }
    let north = report
        .provinces
        .iter()
        .find(|province| province.province.to_string() == "01a07577-e209-792a-aba0-8dba97d92ac6")
        .unwrap();
    assert_eq!(
        (
            north.production_workers,
            north.food_required,
            north.food_shortfall
        ),
        (20, 40, 0)
    );
}

#[test]
fn construction_uses_real_stock_and_competes_with_existing_jobs() {
    let fixture = scenario();
    let mut baseline = Simulation::from_scenario(fixture.clone()).unwrap();
    let baseline_report = baseline.step().unwrap();
    let mut simulation = Simulation::from_scenario(fixture).unwrap();
    let before = simulation.snapshot();
    let result = simulation.execute(build_command()).unwrap();
    assert_eq!(result.sequence, 1);
    let mut expected = stockpiles(&before);
    for flow in result.goods_flows {
        assert!(matches!(
            flow.cause,
            GoodsFlowCause::ConstructionCost { .. }
        ));
        *expected.entry((flow.province, flow.good)).or_default() -= i128::from(flow.amount);
    }
    assert_eq!(expected, stockpiles(&simulation.snapshot()));
    let first = simulation.step().unwrap();
    let north = first
        .provinces
        .iter()
        .find(|province| province.province.to_string() == "01a07577-e209-792a-aba0-8dba97d92ac6")
        .unwrap();
    let baseline_north = baseline_report
        .provinces
        .iter()
        .find(|province| province.province.to_string() == "01a07577-e209-792a-aba0-8dba97d92ac6")
        .unwrap();
    assert!(north.construction_workers > 0);
    assert!(north.production_workers < baseline_north.production_workers);
    assert_eq!(
        north.production_workers + north.construction_workers,
        north.available_workers
    );
    let mut completed = false;
    for _ in 0..20 {
        let report = simulation.step().unwrap();
        if report.events.iter().any(|event| matches!(event, SimulationEvent::ConstructionCompleted { facility, .. } if *facility == facility_id(100))) {
            assert!(!report.facilities.iter().any(|facility| facility.facility == facility_id(100)), "completion must not create retroactive production");
            completed = true;
            break;
        }
    }
    assert!(completed);
    assert!(simulation.snapshot().construction_projects.is_empty());
    assert!(
        simulation
            .step()
            .unwrap()
            .facilities
            .iter()
            .any(|facility| facility.facility == facility_id(100))
    );
}

#[test]
fn rejected_commands_do_not_partially_spend_stock_or_append_history() {
    let mut fixture = scenario();
    let north = fixture
        .initial_state
        .provinces
        .iter_mut()
        .find(|province| province.id.to_string() == "01a07577-e209-792a-aba0-8dba97d92ac6")
        .unwrap();
    north.stockpile.insert("tools".into(), 0); // timber validates first; later failure must undo nothing.
    let mut simulation = Simulation::from_scenario(fixture).unwrap();
    let before = simulation.snapshot();
    assert!(simulation.execute(build_command()).is_err());
    assert_eq!(simulation.snapshot(), before);
    assert!(simulation.command_log().is_empty());
    let mut foreign = build_command();
    let SimulationCommand::StartConstruction { country, .. } = &mut foreign;
    *country = "00000000-0000-0000-0000-0000000003e7".parse().unwrap();
    assert!(simulation.execute(foreign).is_err());
    assert_eq!(simulation.snapshot(), before);
}

#[test]
fn repeated_facility_id_is_rejected_without_double_spending() {
    let mut simulation = Simulation::from_scenario(scenario()).unwrap();
    simulation.execute(build_command()).unwrap();
    let before = simulation.snapshot();
    assert!(simulation.execute(build_command()).is_err());
    assert_eq!(simulation.snapshot(), before);
    assert_eq!(simulation.command_log().len(), 1);
}

#[test]
fn save_resume_and_command_replay_match_uninterrupted_execution() {
    let fixture = scenario();
    let mut uninterrupted = Simulation::from_scenario(fixture.clone()).unwrap();
    uninterrupted.step().unwrap();
    uninterrupted.execute(build_command()).unwrap();
    uninterrupted.step().unwrap();
    let save = uninterrupted.save_json().unwrap();
    let mut resumed = Simulation::from_save_json(&save).unwrap();
    assert!(!resumed.snapshot().construction_projects.is_empty());
    for _ in 0..12 {
        assert_eq!(uninterrupted.step().unwrap(), resumed.step().unwrap());
    }
    assert_eq!(uninterrupted.snapshot(), resumed.snapshot());
    let mut replayed = Simulation::replay(
        fixture,
        uninterrupted.command_log(),
        uninterrupted.clock().day(),
    )
    .unwrap();
    assert_eq!(
        uninterrupted.state_hash().unwrap(),
        replayed.state_hash().unwrap()
    );
}

#[test]
fn definition_and_entity_insertion_order_does_not_change_results() {
    let fixture = scenario();
    let mut reordered = fixture.clone();
    reordered.definitions.facilities.reverse();
    reordered.definitions.goods.reverse();
    reordered.initial_state.provinces.reverse();
    reordered.initial_state.population_groups.reverse();
    reordered.initial_state.facilities.reverse();
    let mut first = Simulation::from_scenario(fixture).unwrap();
    let mut second = Simulation::from_scenario(reordered).unwrap();
    first.execute(build_command()).unwrap();
    second.execute(build_command()).unwrap();
    for _ in 0..10 {
        assert_eq!(first.step().unwrap(), second.step().unwrap());
    }
    assert_eq!(first.state_hash().unwrap(), second.state_hash().unwrap());
}

#[test]
fn lower_production_priority_reserves_scarce_inputs_first() {
    let mut fixture = scenario();
    fixture
        .initial_state
        .facilities
        .retain(|facility| facility.province.to_string() != "01a07577-e209-792a-aba0-8dba97d92ac6");
    fixture.initial_state.facilities.extend([
        FacilityState {
            id: facility_id(90),
            province: "01a07577-e209-792a-aba0-8dba97d92ac6".parse().unwrap(),
            definition: "tool_workshop".into(),
            level: 1,
            production_priority: 0,
        },
        FacilityState {
            id: facility_id(80),
            province: "01a07577-e209-792a-aba0-8dba97d92ac6".parse().unwrap(),
            definition: "tool_workshop".into(),
            level: 1,
            production_priority: 10,
        },
    ]);
    fixture
        .initial_state
        .provinces
        .iter_mut()
        .find(|province| province.id.to_string() == "01a07577-e209-792a-aba0-8dba97d92ac6")
        .unwrap()
        .stockpile
        .insert("timber".into(), 1);
    let report = Simulation::from_scenario(fixture).unwrap().step().unwrap();
    assert_eq!(
        report
            .facilities
            .iter()
            .find(|facility| facility.facility == facility_id(90))
            .unwrap()
            .active_workers,
        1
    );
    assert_eq!(
        report
            .facilities
            .iter()
            .find(|facility| facility.facility == facility_id(80))
            .unwrap()
            .active_workers,
        0
    );
}

#[test]
fn failed_day_leaves_all_authoritative_state_unchanged() {
    let mut fixture = scenario();
    for province in &mut fixture.initial_state.provinces {
        province.stockpile.insert(
            "grain".into(),
            if province.id.to_string() == "01a07577-e209-792a-aba0-8dba97d92ac6" {
                u64::MAX
            } else {
                0
            },
        );
    }
    let mut simulation = Simulation::from_scenario(fixture).unwrap();
    let before = simulation.snapshot();
    let error = simulation.step().unwrap_err();
    assert!(error.to_string().contains("overflow"));
    assert_eq!(simulation.snapshot(), before);
    assert_eq!(simulation.clock().day(), 0);
}

#[test]
fn food_shortage_is_reported_without_inventing_demographic_changes() {
    let mut fixture = scenario();
    fixture.initial_state.facilities.clear();
    for province in &mut fixture.initial_state.provinces {
        province.stockpile.insert("grain".into(), 0);
    }
    let mut simulation = Simulation::from_scenario(fixture).unwrap();
    let before = simulation.snapshot().population_groups;
    let report = simulation.step().unwrap();
    assert!(
        report
            .provinces
            .iter()
            .all(|province| province.food_shortfall == province.population)
    );
    assert_eq!(report.events.len(), 2);
    assert_eq!(simulation.snapshot().population_groups, before);
}

#[test]
fn saves_reject_bad_versions_references_and_replay_order() {
    let mut simulation = Simulation::from_scenario(scenario()).unwrap();
    let saved = simulation.save_json().unwrap();
    let mut invalid: serde_json::Value = serde_json::from_str(&saved).unwrap();
    invalid["ruleset_version"] = "unsupported".into();
    assert!(Simulation::from_save_json(&invalid.to_string()).is_err());
    invalid = serde_json::from_str(&saved).unwrap();
    invalid["current_state"]["provinces"][0]["country"] = "unknown_country".into();
    assert!(Simulation::from_save_json(&invalid.to_string()).is_err());
    assert!(Simulation::from_save_json(&(saved + " trailing")).is_err());
    assert!(
        Simulation::replay(
            scenario(),
            &[RecordedCommand {
                day: 0,
                sequence: 2,
                command: build_command()
            }],
            1
        )
        .is_err()
    );
}

#[test]
fn exhausted_day_counter_fails_without_mutation() {
    let mut original = Simulation::from_scenario(scenario()).unwrap();
    let mut saved: serde_json::Value =
        serde_json::from_str(&original.save_json().unwrap()).unwrap();
    saved["current_state"]["day"] = u64::MAX.into();
    let mut simulation = Simulation::from_save_json(&saved.to_string()).unwrap();
    let before = simulation.snapshot();
    assert!(simulation.step().is_err());
    assert_eq!(simulation.snapshot(), before);
}

fn capacity_boundary_scenario(
    operating_capacity: u64,
    new_capacity: u64,
    construction_workers: u64,
) -> Scenario {
    let mut fixture = scenario();
    fixture.initial_state.facilities = vec![FacilityState {
        id: facility_id(1),
        province: "01a07577-e209-792a-aba0-8dba97d92ac6".parse().unwrap(),
        definition: "logging_camp".into(),
        level: 1,
        production_priority: 10,
    }];
    for definition in &mut fixture.definitions.facilities {
        match definition.id.0.as_str() {
            "logging_camp" => definition.workers_per_level = operating_capacity,
            "grain_farm" => {
                definition.workers_per_level = new_capacity;
                definition.construction.max_workers = construction_workers;
                definition.construction.worker_days = construction_workers;
            }
            _ => {}
        }
    }
    fixture
}

#[test]
fn construction_rejects_future_country_capacity_overflow_before_spending() {
    let fixture = capacity_boundary_scenario(u64::MAX - 1, 2, 1);
    let mut simulation = Simulation::from_scenario(fixture).unwrap();
    let before = simulation.snapshot();
    let mut command = build_command();
    let SimulationCommand::StartConstruction {
        province, workers, ..
    } = &mut command;
    *province = "01a07577-e209-7938-8120-efb504849d04".parse().unwrap();
    *workers = 1;
    assert!(matches!(
        simulation.execute(command),
        Err(aggregate_simulation_core::SimulationError::CommandRejected(
            _
        ))
    ));
    assert_eq!(simulation.snapshot(), before);
    assert!(simulation.command_log().is_empty());
}

#[test]
fn construction_rejects_combined_labor_overflow_before_spending() {
    let fixture = capacity_boundary_scenario(u64::MAX - 1, 1, 2);
    let mut simulation = Simulation::from_scenario(fixture).unwrap();
    let before = simulation.snapshot();
    let mut command = build_command();
    let SimulationCommand::StartConstruction { workers, .. } = &mut command;
    *workers = 2;
    assert!(simulation.execute(command).is_err());
    assert_eq!(simulation.snapshot(), before);
    assert!(simulation.command_log().is_empty());
}

#[test]
fn pending_projects_reserve_capacity_for_later_construction_commands() {
    let fixture = capacity_boundary_scenario(u64::MAX - 2, 1, 1);
    let mut simulation = Simulation::from_scenario(fixture).unwrap();
    for id in 100..102 {
        let mut command = build_command();
        let SimulationCommand::StartConstruction {
            facility, workers, ..
        } = &mut command;
        *facility = facility_id(id);
        *workers = 1;
        simulation.execute(command).unwrap();
    }
    let before = simulation.snapshot();
    let mut command = build_command();
    let SimulationCommand::StartConstruction {
        facility, workers, ..
    } = &mut command;
    *facility = facility_id(102);
    *workers = 1;
    assert!(simulation.execute(command).is_err());
    assert_eq!(simulation.snapshot(), before);
    let mut resumed = Simulation::from_save_json(&simulation.save_json().unwrap()).unwrap();
    assert_eq!(resumed.snapshot(), before);
}
