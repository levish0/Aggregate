use aggregate_programs::{
    CommandOutcome, GoodsFlow, GoodsFlowCause, SimulationCommand, SimulationError, SimulationEvent,
    world_storage::{self, ConstructionProject, DefinitionRegistry, Facility, Province},
};
use aggregate_scenario::validate_world_state;
use aggregate_world::*;
use bevy_ecs::prelude::*;

/// Commands execute serially between days. Validate every input and reserve every cost before
/// committing anything. The same entry point will serve player, AI and automation commands.
pub(crate) fn execute_command(
    world: &mut World,
    command: &SimulationCommand,
    sequence: u64,
    scenario: &Scenario,
) -> Result<CommandOutcome, SimulationError> {
    let SimulationCommand::StartConstruction {
        country,
        province,
        facility,
        definition,
        workers,
        production_priority,
    } = command
    else {
        return Err(SimulationError::CommandRejected(
            "unsupported economy command".into(),
        ));
    };
    let reject = |reason: String| SimulationError::CommandRejected(reason);
    if facility.0.is_nil() {
        return Err(reject("facility ID must not be the nil UUID".into()));
    }
    if world
        .query::<&Facility>()
        .iter(world)
        .any(|existing| existing.0.id == *facility)
        || world
            .query::<&ConstructionProject>()
            .iter(world)
            .any(|project| project.0.facility_id == *facility)
    {
        return Err(reject(format!("facility ID {facility} is already in use")));
    }
    let definition_data = world
        .resource::<DefinitionRegistry>()
        .facilities
        .get(definition)
        .cloned()
        .ok_or_else(|| reject(format!("unknown facility definition {definition}")))?;
    if *workers == 0 || *workers > definition_data.construction.max_workers {
        return Err(reject(format!(
            "requested construction workers must be in 1..={}",
            definition_data.construction.max_workers
        )));
    }
    let (province_entity, mut stockpile) = world
        .query::<(Entity, &Province)>()
        .iter(world)
        .find(|(_, value)| value.0.id == *province)
        .map(|(entity, value)| (entity, value.0.clone()))
        .ok_or_else(|| reject(format!("unknown province {province}")))?;
    if stockpile.country != *country {
        return Err(reject(format!(
            "country {country} does not control province {province}"
        )));
    }
    let mut goods_flows = Vec::new();
    for (good, amount) in &definition_data.construction.goods {
        let available = stockpile.stockpile.get(good).copied().unwrap_or(0);
        if available < *amount {
            return Err(SimulationError::InsufficientConstructionGoods {
                province: province.clone(),
                good: good.clone(),
                required: *amount,
                available,
            });
        }
        stockpile.stockpile.insert(good.clone(), available - amount);
        goods_flows.push(GoodsFlow {
            province: province.clone(),
            good: good.clone(),
            amount: *amount,
            cause: GoodsFlowCause::ConstructionCost {
                facility: facility.clone(),
            },
        });
    }
    let project = ConstructionProjectState {
        facility_id: facility.clone(),
        province: province.clone(),
        definition: definition.clone(),
        requested_workers: *workers,
        remaining_construction_points: definition_data.construction.construction_points,
        production_priority: *production_priority,
    };
    // Validate shared totals, including the capacity of pending buildings, through the same
    // boundary as save loading. Accepting a command must not make its own save unloadable.
    let mut proposed_state = world_storage::snapshot(world);
    proposed_state.construction_projects.push(project.clone());
    validate_world_state(&scenario.definitions, &scenario.rules, &proposed_state)
        .map_err(|error| reject(error.to_string()))?;
    // Only now is the validated transaction published to the authoritative World.
    world
        .get_mut::<Province>(province_entity)
        .expect("validated province remains present")
        .0 = stockpile;
    world.spawn(ConstructionProject(project));
    Ok(CommandOutcome {
        sequence,
        goods_flows,
        event: SimulationEvent::ConstructionStarted {
            province: province.clone(),
            facility: facility.clone(),
        },
    })
}
