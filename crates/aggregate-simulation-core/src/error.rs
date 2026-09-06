use aggregate_scenario::ScenarioError;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum SimulationError {
    Scenario(ScenarioError),
    CommandRejected(String),
    InsufficientConstructionGoods {
        province: aggregate_world::ProvinceId,
        good: aggregate_world::GoodId,
        required: u64,
        available: u64,
    },
    DayFailed {
        day: u64,
        phase: &'static str,
        reason: String,
    },
    InvalidSave(String),
    InvalidReplay(String),
    InvalidModules(String),
}

impl fmt::Display for SimulationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scenario(error) => error.fmt(formatter),
            Self::CommandRejected(reason) => write!(formatter, "command rejected: {reason}"),
            Self::InsufficientConstructionGoods {
                province,
                good,
                required,
                available,
            } => write!(
                formatter,
                "command rejected: province {province}, good {good}: construction needs {required}, available {available}"
            ),
            Self::DayFailed { day, phase, reason } => {
                write!(formatter, "day {day}, {phase}: {reason}")
            }
            Self::InvalidSave(reason) => write!(formatter, "invalid save: {reason}"),
            Self::InvalidReplay(reason) => write!(formatter, "invalid replay: {reason}"),
            Self::InvalidModules(reason) => write!(formatter, "invalid modules: {reason}"),
        }
    }
}
impl Error for SimulationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Scenario(error) => Some(error),
            _ => None,
        }
    }
}
impl From<ScenarioError> for SimulationError {
    fn from(value: ScenarioError) -> Self {
        Self::Scenario(value)
    }
}
