use aggregate_programs::SimulationProgram;
use aggregate_simulation_core::Simulation;
use aggregate_world::*;
use std::sync::Arc;

fn programs() -> Vec<Arc<dyn SimulationProgram>> {
    vec![Arc::new(aggregate_economy::EconomyProgram)]
}

fn fixture(level: u64, same_country: bool, has_project: bool) -> Scenario {
    let mut scenario = aggregate_scenario::parse_scenario(include_str!("../../../scenarios/foundation.json")).unwrap();
    let mut sector = aggregate_economy::content::definitions().facilities.into_iter().find(|item| item.id.0 == "construction_sector").unwrap();
    sector.construction.construction_points = 100;
    scenario.definitions.facilities.push(sector);
    scenario.initial_state.facilities.clear();
    let source = scenario.initial_state.provinces[0].id.clone();
    let destination = scenario.initial_state.provinces[1].id.clone();
    for province in &mut scenario.initial_state.provinces {
        province.stockpile.insert("timber".into(), 1000);
        province.stockpile.insert("tools".into(), 1000);
    }
    for population in &mut scenario.initial_state.population_groups {
        population.workforce = if population.province == source { 20 } else { 0 };
    }
    if !same_country {
        let mut country = scenario.initial_state.countries[0].clone();
        country.id = "00000000-0000-0000-0000-000000000099".parse().unwrap();
        scenario.initial_state.provinces[1].country = country.id.clone();
        scenario.initial_state.countries.push(country);
    }
    scenario.initial_state.facilities.push(FacilityState {
        id: "00000000-0000-0000-0000-000000000001".parse().unwrap(),
        province: source,
        definition: "construction_sector".into(),
        level,
        production_priority: 30,
    });
    if has_project {
        scenario.initial_state.construction_projects.push(ConstructionProjectState {
            facility_id: "00000000-0000-0000-0000-000000000002".parse().unwrap(),
            province: destination,
            definition: "construction_sector".into(),
            requested_workers: 10,
            remaining_construction_points: 100,
            production_priority: 30,
        });
    }
    scenario
}

fn start(mut scenario: Scenario) -> Simulation {
    let projects = std::mem::take(&mut scenario.initial_state.construction_projects);
    let countries: std::collections::BTreeMap<_,_> = scenario.initial_state.provinces.iter().map(|province| (province.id.clone(), province.country.clone())).collect();
    let mut simulation = Simulation::from_scenario_with_programs(scenario, programs()).unwrap();
    for project in projects {
        simulation.execute(aggregate_simulation_core::SimulationCommand::StartConstruction { country: countries[&project.province].clone(), province: project.province, facility: project.facility_id, definition: project.definition, workers: project.requested_workers, production_priority: project.production_priority }).unwrap();
    }
    simulation
}

#[test]
fn construction_industry_consumes_inputs_and_supplies_other_provinces_without_double_labor() {
    for (level, workers, points) in [(1, 10, 40), (2, 20, 80)] {
        let mut simulation = start(fixture(level, true, true));
        let report = simulation.step().unwrap();
        let facility = &report.facilities[0];
        assert_eq!(facility.active_workers, workers);
        assert_eq!(facility.inputs[&GoodId("timber".into())], workers * 2);
        assert_eq!(facility.inputs[&GoodId("tools".into())], workers);
        assert_eq!(facility.construction_points, points);
        assert_eq!(report.constructions[0].active_workers, 0);
        assert_eq!(report.constructions[0].sector_construction_points, points);
        assert_eq!(report.constructions[0].remaining_construction_points, 100 - points);
        assert_eq!(report.provinces.iter().map(|province| province.production_workers + province.construction_workers).sum::<u64>(), workers);
        let mut restored = Simulation::from_save_json_with_programs(&simulation.save_json().unwrap(), programs()).unwrap();
        assert_eq!(simulation.step().unwrap(), restored.step().unwrap());
        assert_eq!(simulation.snapshot(), restored.snapshot());
    }
}

#[test]
fn idle_or_foreign_construction_does_not_consume_domestic_materials() {
    for (same_country, has_project) in [(true, false), (false, true)] {
        let mut simulation = start(fixture(1, same_country, has_project));
        let report = simulation.step().unwrap();
        assert_eq!(report.facilities[0].active_workers, 0);
        assert_eq!(report.facilities[0].construction_points, 0);
        assert!(report.facilities[0].inputs.values().all(|amount| *amount == 0));
        assert!(report.constructions.iter().all(|project| project.sector_construction_points == 0));
    }
}

#[test]
fn missing_materials_reduce_construction_supply_and_failure_remains_atomic() {
    let mut scenario = fixture(1, true, true);
    scenario.initial_state.provinces[0].stockpile.insert("tools".into(), 3);
    let mut simulation = start(scenario);
    let report = simulation.step().unwrap();
    assert_eq!(report.facilities[0].active_workers, 3);
    assert_eq!(report.constructions[0].sector_construction_points, 12);

    let mut scenario = fixture(1, true, true);
    let sector = scenario.definitions.facilities.iter_mut().find(|item| item.id.0 == "construction_sector").unwrap();
    sector.outputs_per_worker_day.insert("grain".into(), 1);
    scenario.initial_state.provinces[0].stockpile.insert("grain".into(), u64::MAX);
    let mut simulation = start(scenario);
    let before = simulation.snapshot();
    assert!(simulation.step().is_err());
    assert_eq!(simulation.snapshot(), before);
}
