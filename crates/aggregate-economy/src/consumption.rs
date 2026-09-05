use crate::EconomyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsumptionOutcome {
    pub required: u64,
    pub consumed: u64,
    pub shortfall: u64,
}

/// Calculate demand and unmet consumption without assigning demographic effects.
pub fn plan_consumption(
    population: u64,
    per_person_day: u64,
    available: u64,
) -> Result<ConsumptionOutcome, EconomyError> {
    let required =
        population
            .checked_mul(per_person_day)
            .ok_or(EconomyError::ArithmeticOverflow {
                calculation: "population consumption requirement",
            })?;
    let consumed = required.min(available);

    Ok(ConsumptionOutcome {
        required,
        consumed,
        shortfall: required - consumed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_shortfall_without_consuming_nonexistent_goods() {
        let outcome = plan_consumption(40, 3, 100).unwrap();
        assert_eq!(outcome.required, 120);
        assert_eq!(outcome.consumed, 100);
        assert_eq!(outcome.shortfall, 20);
        assert_eq!(outcome.required, outcome.consumed + outcome.shortfall);
    }

    #[test]
    fn consumes_only_demand_and_handles_zero_population() {
        assert_eq!(
            plan_consumption(40, 2, 100).unwrap(),
            ConsumptionOutcome {
                required: 80,
                consumed: 80,
                shortfall: 0,
            }
        );
        assert_eq!(plan_consumption(0, 2, 100).unwrap().consumed, 0);
        assert_eq!(plan_consumption(40, 0, 100).unwrap().required, 0);
    }

    #[test]
    fn rejects_requirement_overflow() {
        assert!(matches!(
            plan_consumption(u64::MAX, 2, u64::MAX),
            Err(EconomyError::ArithmeticOverflow { .. })
        ));
    }
}
