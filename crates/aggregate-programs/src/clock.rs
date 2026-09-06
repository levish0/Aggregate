use bevy_ecs::prelude::Resource;

/// Whole elapsed days since the scenario start. Rendering never advances this clock.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SimulationClock {
    pub day: u64,
}

impl SimulationClock {
    pub fn day(self) -> u64 {
        self.day
    }
}
