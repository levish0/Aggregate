use aggregate_world::FacilityId;

use crate::EconomyError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaborRequest {
    pub facility_id: FacilityId,
    pub requested_workers: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaborAllocation {
    pub facility_id: FacilityId,
    pub allocated_workers: u64,
}

/// Share one workforce across operating facilities and construction projects.
///
/// Shortages are allocated proportionally using largest remainders. Persistent
/// facility IDs break ties, so request order cannot affect the result. Every
/// request receives an entry in the result, sorted by ID, including zero demands.
pub fn allocate_labor(
    available_workers: u64,
    requests: &[LaborRequest],
) -> Result<Vec<LaborAllocation>, EconomyError> {
    let mut ordered_requests: Vec<_> = requests.iter().collect();
    ordered_requests.sort_by(|left, right| left.facility_id.cmp(&right.facility_id));

    for pair in ordered_requests.windows(2) {
        if pair[0].facility_id == pair[1].facility_id {
            return Err(EconomyError::DuplicateLaborRequest {
                facility_id: pair[0].facility_id.clone(),
            });
        }
    }

    let total_demand = ordered_requests.iter().try_fold(0_u64, |total, request| {
        total
            .checked_add(request.requested_workers)
            .ok_or(EconomyError::ArithmeticOverflow {
                calculation: "total requested workforce",
            })
    })?;

    if total_demand <= available_workers {
        return Ok(ordered_requests
            .into_iter()
            .map(|request| LaborAllocation {
                facility_id: request.facility_id.clone(),
                allocated_workers: request.requested_workers,
            })
            .collect());
    }

    let mut allocations = Vec::with_capacity(ordered_requests.len());
    let mut remainders = Vec::with_capacity(ordered_requests.len());
    let mut unallocated_workers = available_workers;

    for (index, request) in ordered_requests.iter().enumerate() {
        let numerator = u128::from(available_workers) * u128::from(request.requested_workers);
        let allocated_workers = (numerator / u128::from(total_demand)) as u64;
        unallocated_workers -= allocated_workers;
        allocations.push(LaborAllocation {
            facility_id: request.facility_id.clone(),
            allocated_workers,
        });
        remainders.push((index, numerator % u128::from(total_demand)));
    }

    remainders.sort_by(
        |(left_index, left_remainder), (right_index, right_remainder)| {
            right_remainder.cmp(left_remainder).then_with(|| {
                allocations[*left_index]
                    .facility_id
                    .cmp(&allocations[*right_index].facility_id)
            })
        },
    );

    for (index, _) in remainders {
        if unallocated_workers == 0 {
            break;
        }
        allocations[index].allocated_workers += 1;
        unallocated_workers -= 1;
    }

    Ok(allocations)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facility_id(number: u128) -> FacilityId {
        FacilityId(uuid::Uuid::from_u128(number))
    }

    fn request(facility_number: u128, requested_workers: u64) -> LaborRequest {
        LaborRequest {
            facility_id: facility_id(facility_number),
            requested_workers,
        }
    }

    fn allocated_workers(allocations: &[LaborAllocation]) -> Vec<u64> {
        allocations
            .iter()
            .map(|allocation| allocation.allocated_workers)
            .collect()
    }

    #[test]
    fn apportions_shortages_by_largest_remainder_and_persistent_id() {
        let allocations =
            allocate_labor(7, &[request(3, 9), request(1, 5), request(2, 3)]).unwrap();
        assert_eq!(allocated_workers(&allocations), vec![2, 1, 4]);

        let tied = allocate_labor(1, &[request(2, 1), request(1, 1)]).unwrap();
        assert_eq!(allocated_workers(&tied), vec![1, 0]);
        assert_eq!(tied[0].facility_id, facility_id(1));
    }

    #[test]
    fn construction_and_production_share_the_same_workforce() {
        let requests = [request(1, 5), request(2, 10)];
        let allocations = allocate_labor(6, &requests).unwrap();
        assert_eq!(allocated_workers(&allocations), vec![2, 4]);
        assert_eq!(
            allocations
                .iter()
                .map(|allocation| allocation.allocated_workers)
                .sum::<u64>(),
            6
        );
    }

    #[test]
    fn request_permutations_do_not_change_assignments() {
        let requests = [request(1, 3), request(2, 5), request(3, 9)];
        let expected = allocate_labor(7, &requests).unwrap();
        for indices in [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ] {
            let permuted = indices.map(|index| requests[index].clone());
            assert_eq!(allocate_labor(7, &permuted).unwrap(), expected);
        }
    }

    #[test]
    fn never_exceeds_request_or_assigns_a_worker_twice() {
        for available in 0..=12 {
            for first in 0..=4 {
                for second in 0..=4 {
                    for third in 0..=4 {
                        let requests = [request(1, first), request(2, second), request(3, third)];
                        let allocations = allocate_labor(available, &requests).unwrap();
                        let assigned: u64 = allocations
                            .iter()
                            .map(|allocation| allocation.allocated_workers)
                            .sum();
                        assert_eq!(assigned, available.min(first + second + third));
                        for (allocation, request) in allocations.iter().zip(&requests) {
                            assert!(allocation.allocated_workers <= request.requested_workers);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn empty_zero_and_surplus_workforces_are_well_defined() {
        assert!(allocate_labor(10, &[]).unwrap().is_empty());
        let requests = [request(1, 0), request(2, 3)];
        assert_eq!(
            allocated_workers(&allocate_labor(0, &requests).unwrap()),
            vec![0, 0]
        );
        assert_eq!(
            allocated_workers(&allocate_labor(8, &requests).unwrap()),
            vec![0, 3]
        );
    }

    #[test]
    fn rejects_duplicate_requests_and_total_demand_overflow() {
        assert!(matches!(
            allocate_labor(10, &[request(1, 3), request(1, 2)]),
            Err(EconomyError::DuplicateLaborRequest { .. })
        ));
        assert!(matches!(
            allocate_labor(10, &[request(1, u64::MAX), request(2, 1)]),
            Err(EconomyError::ArithmeticOverflow { .. })
        ));
    }

    #[test]
    fn proportional_products_support_the_full_quantity_range() {
        let requests = [request(1, u64::MAX - 1), request(2, 1)];
        let allocations = allocate_labor(u64::MAX - 1, &requests).unwrap();
        assert_eq!(allocated_workers(&allocations), vec![u64::MAX - 2, 1]);
    }
}
