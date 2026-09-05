use bevy_ecs::prelude::Resource;
use std::{error::Error, fmt};

/// A logical step, deliberately not yet assigned an hour/day/month duration.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SimulationClock {
    pub(crate) tick: u64,
}

impl SimulationClock {
    pub fn tick(self) -> u64 {
        self.tick
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickOverflow;

impl fmt::Display for TickOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("simulation tick exhausted its representable range")
    }
}

impl Error for TickOverflow {}
