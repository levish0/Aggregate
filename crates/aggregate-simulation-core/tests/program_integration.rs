use aggregate_programs::{
    ProgramContext, ProgramDependency, ProgramManifest, ProgramPlan, SimulationProgram, Version,
};
use aggregate_simulation_core::Simulation;
use aggregate_world::{Scenario, WorldSnapshot};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone)]
struct WorkforceModule {
    manifest: ProgramManifest,
    limit: u64,
    fail: bool,
}

impl SimulationProgram for WorkforceModule {
    fn manifest(&self) -> ProgramManifest {
        self.manifest.clone()
    }
    fn initialize(&self, _: &WorldSnapshot) -> Result<Value, String> {
        Ok(json!({"days":0}))
    }
    fn validate_state(&self, _: &WorldSnapshot, state: &Value) -> Result<(), String> {
        state
            .get("days")
            .and_then(Value::as_u64)
            .map(|_| ())
            .ok_or("invalid test program state".into())
    }
    fn plan_day(&self, context: &ProgramContext<'_>, state: &Value) -> Result<ProgramPlan, String> {
        if self.fail {
            return Err("test calculation failure".into());
        }
        for dependency in context.dependencies.values() {
            assert_eq!(
                dependency["days"], state["days"],
                "dependencies must expose committed state, not this day's proposed state"
            );
        }
        Ok(ProgramPlan {
            next_state: json!({"days":state["days"].as_u64().unwrap() + 1}),
            workforce_limits: BTreeMap::from([(
                context.world.population_groups[0].id.clone(),
                self.limit,
            )]),
        })
    }
}

fn program(id: &str, limit: u64) -> WorkforceModule {
    WorkforceModule {
        manifest: ProgramManifest {
            id: id.into(),
            version: Version::new(1, 0, 0),
            api_version: 1,
            state_schema_version: 1,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
        },
        limit,
        fail: false,
    }
}

fn scenario() -> Scenario {
    aggregate_scenario::parse_scenario(include_str!("../../../scenarios/foundation.json")).unwrap()
}

#[test]
fn module_effects_are_opt_in_and_save_resume_replay_preserve_owned_state() {
    let provider: Arc<dyn SimulationProgram> = Arc::new(program("test.workforce", 5));
    let mut baseline = with_programs(scenario(), Vec::new()).unwrap();
    let mut simulation =
        with_programs(scenario(), vec![provider.clone()]).unwrap();
    assert_eq!(baseline.step().unwrap().provinces[0].available_workers, 20);
    let before = simulation.snapshot().population_groups;
    assert_eq!(simulation.step().unwrap().provinces[0].available_workers, 5);
    assert_eq!(
        simulation.snapshot().population_groups,
        before,
        "temporary availability must not destroy population/workforce"
    );
    assert_eq!(simulation.program_states().iter().find(|state| state.manifest.id == "test.workforce").unwrap().payload["days"], 1);
    let save = simulation.save_json().unwrap();
    assert!(
        Simulation::from_save_json(&save).is_err(),
        "missing program must never silently disappear from a save"
    );
    let mut resumed = restore_programs(
        &save,
        vec![provider.clone(), Arc::new(program("test.disabled", 0))],
    )
    .unwrap();
    assert_eq!(
        resumed.program_states().len(),
        2,
        "installed extras remain disabled"
    );
    assert_eq!(resumed.step().unwrap(), simulation.step().unwrap());
    let mut replayed =
        Simulation::replay_with_programs(scenario(), &[], 2, with_economy(vec![provider])).unwrap();
    assert_eq!(
        replayed.state_hash().unwrap(),
        simulation.state_hash().unwrap()
    );
    assert_eq!(
        resumed.state_hash().unwrap(),
        simulation.state_hash().unwrap()
    );
    let changed = Arc::new(WorkforceModule {
        manifest: ProgramManifest {
            version: Version::new(2, 0, 0),
            ..program("test.workforce", 5).manifest
        },
        ..program("test.workforce", 5)
    });
    assert!(restore_programs(&save, vec![changed]).is_err());
}

#[test]
fn modules_reject_cycles_conflicts_missing_or_incompatible_dependencies() {
    let mut first = program("test.first", 10);
    first.manifest.dependencies.push(ProgramDependency {
        id: "test.second".into(),
        version: "^1".parse().unwrap(),
    });
    assert!(
        with_programs(scenario(), vec![Arc::new(first.clone())]).is_err()
    );
    let mut second = program("test.second", 12);
    second.manifest.dependencies.push(ProgramDependency {
        id: "test.first".into(),
        version: "^1".parse().unwrap(),
    });
    assert!(
        with_programs(
            scenario(),
            vec![Arc::new(first.clone()), Arc::new(second.clone())]
        )
        .is_err()
    );
    second.manifest.dependencies.clear();
    second.manifest.version = Version::new(2, 0, 0);
    assert!(
        with_programs(
            scenario(),
            vec![Arc::new(first.clone()), Arc::new(second)]
        )
        .is_err()
    );
    first.manifest.dependencies.clear();
    first.manifest.conflicts.push("test.second".into());
    assert!(
        with_programs(
            scenario(),
            vec![Arc::new(first), Arc::new(program("test.second", 12))]
        )
        .is_err()
    );
    assert!(
        with_programs(
            scenario(),
            vec![
                Arc::new(program("test.same", 12)),
                Arc::new(program("test.same", 12))
            ]
        )
        .is_err()
    );
}

#[test]
fn order_is_stable_limits_combine_and_failed_modules_commit_nothing() {
    let first = program("test.first", 10);
    let mut second = program("test.second", 8);
    second.manifest.dependencies.push(ProgramDependency {
        id: "test.first".into(),
        version: "^1".parse().unwrap(),
    });
    let mut forward = with_programs(
        scenario(),
        vec![Arc::new(first.clone()), Arc::new(second.clone())],
    )
    .unwrap();
    let mut reverse = with_programs(
        scenario(),
        vec![Arc::new(second.clone()), Arc::new(first.clone())],
    )
    .unwrap();
    assert_eq!(forward.step().unwrap(), reverse.step().unwrap());
    assert_eq!(forward.step().unwrap().provinces[0].available_workers, 8);
    second.fail = true;
    let mut failing = with_programs(
        scenario(),
        vec![Arc::new(first), Arc::new(second)],
    )
    .unwrap();
    let before = failing.state_hash().unwrap();
    assert!(failing.step().is_err());
    assert_eq!(before, failing.state_hash().unwrap());
    assert_eq!(failing.clock().day(), 0);
    let mut invalid = with_programs(
        scenario(),
        vec![Arc::new(program("test.invalid", 21))],
    )
    .unwrap();
    let before = invalid.state_hash().unwrap();
    assert!(invalid.step().is_err());
    assert_eq!(before, invalid.state_hash().unwrap());
}

#[test]
fn core_failure_also_discards_successfully_prepared_module_state() {
    let mut fixture = scenario();
    fixture.initial_state.provinces[0]
        .stockpile
        .insert("grain".into(), u64::MAX);
    let mut simulation = with_programs(
        fixture,
        vec![Arc::new(program("test.workforce", 20))],
    )
    .unwrap();
    let before = simulation.state_hash().unwrap();
    assert!(simulation.step().is_err());
    assert_eq!(before, simulation.state_hash().unwrap());
    assert_eq!(simulation.program_states().iter().find(|state| state.manifest.id == "test.workforce").unwrap().payload["days"], 0);
}

fn with_economy(mut programs: Vec<Arc<dyn SimulationProgram>>) -> Vec<Arc<dyn SimulationProgram>> {
    programs.push(Arc::new(aggregate_economy::EconomyProgram));
    programs
}
fn with_programs(scenario: Scenario, programs: Vec<Arc<dyn SimulationProgram>>) -> Result<Simulation, aggregate_simulation_core::SimulationError> {
    Simulation::from_scenario_with_programs(scenario, with_economy(programs))
}
fn restore_programs(source: &str, programs: Vec<Arc<dyn SimulationProgram>>) -> Result<Simulation, aggregate_simulation_core::SimulationError> {
    Simulation::from_save_json_with_programs(source, with_economy(programs))
}
