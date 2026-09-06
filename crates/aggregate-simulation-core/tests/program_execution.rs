use aggregate_economy::EconomyProgram;
use aggregate_programs::{
    DayReport, ProgramContext, ProgramDependency, ProgramExecution, ProgramManifest, ProgramPlan,
    SimulationProgram, Version,
};
use aggregate_simulation_core::{Simulation, SimulationCommand};
use aggregate_world::{Scenario, WorldSnapshot};
use bevy_ecs::world::World;
use serde_json::{Value, json};
use std::sync::Arc;

fn fixture() -> Scenario {
    aggregate_scenario::parse_scenario(include_str!("../../../scenarios/foundation.json")).unwrap()
}

#[test]
fn no_program_loadout_advances_only_time_and_rejects_economy_commands() {
    let scenario = fixture();
    let mut simulation = Simulation::from_scenario(scenario.clone()).unwrap();
    let report = simulation.step().unwrap();
    assert!(report.provinces.is_empty());
    assert!(report.events.is_empty());
    let mut expected = scenario.initial_state;
    expected.normalize();
    expected.day = 1;
    assert_eq!(simulation.snapshot(), expected);
    let command: SimulationCommand = serde_json::from_str::<Vec<aggregate_programs::RecordedCommand>>(
        include_str!("../../../scenarios/foundation.commands.json"),
    ).unwrap().remove(0).command;
    assert!(simulation.execute(command).unwrap_err().to_string().contains("aggregate.economy"));
    assert!(simulation.command_log().is_empty());
    let save = simulation.save_json().unwrap();
    let mut restored = Simulation::from_save_json_with_programs(&save, vec![Arc::new(EconomyProgram)]).unwrap();
    assert!(restored.program_states().is_empty(), "installing economy must not enable it in a saved world");
    assert!(restored.step().unwrap().provinces.is_empty());
}

struct FailedPreparationProgram;
impl SimulationProgram for FailedPreparationProgram {
    fn manifest(&self) -> ProgramManifest {
        ProgramManifest {
            id: "test.after_economy".into(), version: Version::new(1, 0, 0), api_version: 1,
            state_schema_version: 1,
            dependencies: vec![ProgramDependency { id: "aggregate.economy".into(), version: "^0.1".parse().unwrap() }],
            conflicts: vec![],
        }
    }
    fn initialize(&self, _: &WorldSnapshot) -> Result<Value, String> { Ok(json!({})) }
    fn validate_state(&self, _: &WorldSnapshot, _: &Value) -> Result<(), String> { Ok(()) }
    fn plan_day(&self, _: &ProgramContext<'_>, state: &Value) -> Result<ProgramPlan, String> {
        Ok(ProgramPlan { next_state: state.clone(), workforce_limits: Default::default() })
    }
    fn install(&self, _: &mut World) -> Result<Option<Box<dyn ProgramExecution>>, String> {
        Ok(Some(Box::new(FailedPreparation)))
    }
}
struct FailedPreparation;
impl ProgramExecution for FailedPreparation {
    fn prepare(&mut self, _: &mut World) -> Result<(), (&'static str, String)> {
        Err(("test", "late preparation failure".into()))
    }
    fn commit(&mut self, _: &mut World) -> Option<DayReport> { panic!("failed day must not commit") }
}

#[test]
fn later_program_failure_discards_prepared_economy_and_clock_changes() {
    let mut simulation = Simulation::from_scenario_with_programs(
        fixture(), vec![Arc::new(FailedPreparationProgram), Arc::new(EconomyProgram)],
    ).unwrap();
    let before = simulation.state_hash().unwrap();
    for _ in 0..2 {
        let error = simulation.step().unwrap_err().to_string();
        assert!(error.contains("test.after_economy"));
        assert_eq!(simulation.clock().day(), 0);
        assert_eq!(simulation.state_hash().unwrap(), before);
    }
}
