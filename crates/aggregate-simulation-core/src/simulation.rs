use crate::{SimulationClock, TickOverflow};
use bevy_ecs::prelude::*;

/// Owns the authoritative World. Calls to `step` are independent of wall-clock time.
/// Domain schedules will be added after their resource and phase contracts are defined.
pub struct Simulation {
    world: World,
    schedule: Schedule,
}

impl Default for Simulation {
    fn default() -> Self {
        let mut world = World::new();
        world.init_resource::<SimulationClock>();
        let mut schedule = Schedule::default();
        schedule.add_systems(advance_clock);
        Self { world, schedule }
    }
}

impl Simulation {
    pub fn clock(&self) -> SimulationClock {
        *self.world.resource::<SimulationClock>()
    }

    pub fn step(&mut self) -> Result<SimulationClock, TickOverflow> {
        self.clock().tick.checked_add(1).ok_or(TickOverflow)?;
        self.schedule.run(&mut self.world);
        Ok(self.clock())
    }
}

fn advance_clock(mut clock: ResMut<SimulationClock>) {
    clock.tick += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worlds_advance_independently_without_a_renderer() {
        let mut first = Simulation::default();
        let second = Simulation::default();
        for tick in 1..=100 {
            assert_eq!(first.step().unwrap().tick(), tick);
        }
        assert_eq!(second.clock().tick(), 0);
    }

    #[test]
    fn overflow_fails_without_mutating_the_world() {
        let mut simulation = Simulation::default();
        simulation.world.resource_mut::<SimulationClock>().tick = u64::MAX;
        assert_eq!(simulation.step(), Err(TickOverflow));
        assert_eq!(simulation.clock().tick(), u64::MAX);
    }
}
