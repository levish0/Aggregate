use aggregate_world::*;
use std::collections::BTreeMap;

/// Small authored economy for the geographic sandbox; independent of test fixtures.
pub fn definitions() -> ContentDefinitions {
    let goods = [("grain", "Grain"), ("timber", "Timber"), ("tools", "Tools")]
        .into_iter()
        .map(|(id, name)| GoodDefinition {
            id: GoodId(id.into()),
            name: name.into(),
        })
        .collect();
    let amounts = |items: &[(&str, u64)]| -> BTreeMap<GoodId, u64> {
        items
            .iter()
            .map(|(key, value)| (GoodId((*key).into()), *value))
            .collect()
    };
    let facility = |id: &str,
                    name: &str,
                    workers,
                    inputs: &[(&str, u64)],
                    outputs: &[(&str, u64)],
                    work,
                    max_workers,
                    cost: &[(&str, u64)]| FacilityDefinition {
        id: FacilityDefinitionId(id.into()),
        name: name.into(),
        workers_per_level: workers,
        construction_points_per_worker_day: 0,
        inputs_per_worker_day: amounts(inputs),
        outputs_per_worker_day: amounts(outputs),
        construction: ConstructionDefinition {
            construction_points: work,
            max_workers,
            goods: amounts(cost),
        },
    };
    let mut construction_sector = facility("construction_sector", "Construction sector", 10, &[("timber", 2), ("tools", 1)], &[], 60, 10, &[("timber", 30), ("tools", 15)]);
    construction_sector.construction_points_per_worker_day = 4;
    ContentDefinitions {
        goods,
        facilities: vec![
            construction_sector,
            facility(
                "grain_farm",
                "Grain farm",
                15,
                &[],
                &[("grain", 3)],
                20,
                10,
                &[("timber", 10), ("tools", 5)],
            ),
            facility(
                "logging_camp",
                "Logging camp",
                2,
                &[],
                &[("timber", 2)],
                15,
                5,
                &[("tools", 5)],
            ),
            facility(
                "tool_workshop",
                "Tool workshop",
                3,
                &[("timber", 1)],
                &[("tools", 1)],
                25,
                5,
                &[("timber", 15), ("tools", 5)],
            ),
        ],
    }
}
