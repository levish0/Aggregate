use aggregate_programs::*;
use aggregate_world::*;
use bevy_ecs::prelude::*;
use serde_json::{Value, json};

pub struct EconomyProgram;
impl SimulationProgram for EconomyProgram {
    fn manifest(&self) -> ProgramManifest {
        ProgramManifest {
            id: "aggregate.economy".into(),
            version: Version::new(0, 1, 0),
            api_version: PROGRAM_API_VERSION,
            state_schema_version: 1,
            dependencies: vec![],
            conflicts: vec![],
        }
    }
    fn definitions(&self) -> ContentDefinitions {
        crate::content::definitions()
    }
    fn initialize(&self, _: &WorldSnapshot) -> Result<Value, String> {
        Ok(json!({}))
    }
    fn validate_state(&self, _: &WorldSnapshot, state: &Value) -> Result<(), String> {
        if state.as_object().is_some_and(|object| object.is_empty()) {
            Ok(())
        } else {
            Err("economic state is stored in typed world components".into())
        }
    }
    fn plan_day(&self, _: &ProgramContext<'_>, state: &Value) -> Result<ProgramPlan, String> {
        Ok(ProgramPlan {
            next_state: state.clone(),
            workforce_limits: Default::default(),
        })
    }
    fn install(&self, world: &mut World) -> Result<Option<Box<dyn ProgramExecution>>, String> {
        world.init_resource::<crate::day_work::DayWork>();
        let mut commit = Schedule::default();
        commit.add_systems(crate::schedule::commit::commit_day);
        Ok(Some(Box::new(EconomyExecution {
            prepare: crate::schedule::create_schedule(),
            commit,
        })))
    }
}

struct EconomyExecution {
    prepare: Schedule,
    commit: Schedule,
}
impl ProgramExecution for EconomyExecution {
    fn prepare(&mut self, world: &mut World) -> Result<(), (&'static str, String)> {
        self.prepare.run(world);
        if let Some(failure) = world
            .resource_mut::<crate::day_work::DayWork>()
            .failure
            .take()
        {
            return Err(failure);
        }
        Ok(())
    }
    fn commit(&mut self, world: &mut World) -> Option<DayReport> {
        self.commit.run(world);
        world
            .resource_mut::<crate::day_work::DayWork>()
            .report
            .take()
    }
    fn execute_command(
        &mut self,
        world: &mut World,
        command: &SimulationCommand,
        sequence: u64,
        scenario: &Scenario,
    ) -> Result<CommandOutcome, SimulationError> {
        crate::command::execute_command(world, command, sequence, scenario)
    }
}
