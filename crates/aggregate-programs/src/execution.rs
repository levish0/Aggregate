use crate::{SimulationCommand,SimulationError,CommandOutcome,DayReport};
use aggregate_world::Scenario;
use bevy_ecs::world::World;

/// Program-owned ECS schedules. Preparation must only write private staging resources.
/// Commit runs only after every enabled program prepared successfully; it must be infallible.
/// Runtime scratch data is rebuilt on load; persistent data belongs in world snapshots or
/// the program's validated saved payload. This is a trusted Rust contract, not a sandbox.
pub trait ProgramExecution: Send + Sync {
    fn prepare(&mut self, world: &mut World) -> Result<(), (&'static str,String)>;
    fn commit(&mut self, world: &mut World) -> Option<DayReport>;
    fn execute_command(&mut self, _world: &mut World, _command: &SimulationCommand, _sequence: u64, _scenario: &Scenario) -> Result<CommandOutcome,SimulationError> {
        Err(SimulationError::CommandRejected("program has no handler for this command".into()))
    }
}
