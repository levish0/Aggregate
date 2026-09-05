use std::{error::Error, fmt};

use aggregate_world::FacilityId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EconomyError {
    ArithmeticOverflow { calculation: &'static str },
    DuplicateLaborRequest { facility_id: FacilityId },
    InvalidProductionDefinition { reason: &'static str },
    InvalidFacilityLevel,
}

impl fmt::Display for EconomyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArithmeticOverflow { calculation } => {
                write!(
                    formatter,
                    "economic quantity overflow while calculating {calculation}"
                )
            }
            Self::DuplicateLaborRequest { facility_id } => {
                write!(
                    formatter,
                    "duplicate labor request for facility {facility_id}"
                )
            }
            Self::InvalidProductionDefinition { reason } => {
                write!(formatter, "invalid production definition: {reason}")
            }
            Self::InvalidFacilityLevel => {
                formatter.write_str("a producing facility must have at least one completed level")
            }
        }
    }
}

impl Error for EconomyError {}
