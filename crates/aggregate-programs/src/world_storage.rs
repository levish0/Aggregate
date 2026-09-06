use crate::{
    SimulationClock,
};
use aggregate_world::*;
use bevy_ecs::prelude::*;
use std::collections::BTreeMap;

#[derive(Component)]
pub struct Country(pub CountryState);
#[derive(Component)]
pub struct Province(pub ProvinceState);
#[derive(Component)]
pub struct PopulationGroup(pub PopulationGroupState);
#[derive(Component)]
pub struct Facility(pub FacilityState);
#[derive(Component)]
pub struct ConstructionProject(pub ConstructionProjectState);

#[derive(Resource)]
pub struct DefinitionRegistry {
    pub facilities: BTreeMap<FacilityDefinitionId, FacilityDefinition>,
}
#[derive(Resource)]
pub struct SimulationRules(pub WorldRules);

#[derive(Resource, Default)]
pub struct WorkforceLimits(pub BTreeMap<PopulationGroupId, u64>);

pub fn create_world(scenario: &Scenario, state: &WorldSnapshot) -> World {
    let mut world = World::new();
    world.insert_resource(SimulationClock { day: state.day });
    world.insert_resource(SimulationRules(scenario.rules.clone()));
    world.insert_resource(DefinitionRegistry {
        facilities: scenario
            .definitions
            .facilities
            .iter()
            .map(|definition| (definition.id.clone(), definition.clone()))
            .collect(),
    });
    world.init_resource::<WorkforceLimits>();
    for country in &state.countries {
        world.spawn(Country(country.clone()));
    }
    for province in &state.provinces {
        world.spawn(Province(province.clone()));
    }
    for group in &state.population_groups {
        world.spawn(PopulationGroup(group.clone()));
    }
    for facility in &state.facilities {
        world.spawn(Facility(facility.clone()));
    }
    for project in &state.construction_projects {
        world.spawn(ConstructionProject(project.clone()));
    }
    world
}

pub fn snapshot(world: &mut World) -> WorldSnapshot {
    let mut state = WorldSnapshot {
        day: world.resource::<SimulationClock>().day(),
        countries: world
            .query::<&Country>()
            .iter(world)
            .map(|country| country.0.clone())
            .collect(),
        provinces: world
            .query::<&Province>()
            .iter(world)
            .map(|province| province.0.clone())
            .collect(),
        population_groups: world
            .query::<&PopulationGroup>()
            .iter(world)
            .map(|group| group.0.clone())
            .collect(),
        facilities: world
            .query::<&Facility>()
            .iter(world)
            .map(|facility| facility.0.clone())
            .collect(),
        construction_projects: world
            .query::<&ConstructionProject>()
            .iter(world)
            .map(|project| project.0.clone())
            .collect(),
    };
    state.normalize();
    state
}
