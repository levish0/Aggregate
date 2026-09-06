use crate::WorldInitializationSettings;
use aggregate_geography::GeographyCatalog;
use aggregate_world::*;
use std::collections::BTreeMap;

pub fn initialize_world(catalog: &GeographyCatalog, settings: &WorldInitializationSettings) -> Result<Scenario, String> {
    settings.validate()?;
    let definitions = crate::content::definitions();
    let regions: BTreeMap<_, _> = catalog.regions.iter().map(|region| (&region.id, &region.name)).collect();
    let countries = catalog.countries.iter().map(|country| CountryState {
        id: country.id.clone(), name: country.key.clone(), name_key: Some(format!("country-{}", country.key.to_lowercase())),
    }).collect();
    let mut snapshot = WorldSnapshot { day: 0, countries, provinces: Vec::new(), population_groups: Vec::new(), facilities: Vec::new(), construction_projects: Vec::new() };
    let population = settings.population_per_province;
    let workforce = population * settings.workforce_percent / 100;
    let definition = |key: &str| definitions.facilities.iter().find(|definition| definition.id.0 == key).ok_or_else(||format!("missing starting facility definition {key}"));
    let farm = definition("grain_farm")?;
    let logging = definition("logging_camp")?;
    let tools = definition("tool_workshop")?;
    for province in catalog.provinces.iter().filter(|province| !province.water && province.owner.is_some()) {
        let name = province.region.as_ref().and_then(|id| regions.get(id)).map(|name| name.as_str()).unwrap_or("Province");
        snapshot.provinces.push(ProvinceState {
            id: province.id.clone(), country: province.owner.clone().expect("owned province"),
            name: format!("{name} · {:06X}", province.raster_color), name_key: None,
            stockpile: BTreeMap::from([
                (GoodId("grain".into()), population * settings.food_reserve_days),
                (GoodId("timber".into()), settings.construction_goods_per_province),
                (GoodId("tools".into()), settings.construction_goods_per_province),
            ]),
        });
        snapshot.population_groups.push(PopulationGroupState {
            id: PopulationGroupId(uuid::Uuid::now_v7()), province: province.id.clone(), population, workforce,
        });
        // Enough staple capacity for the chosen population; the remaining workforce
        // supplies tools/timber and construction. No historic buildings are inferred.
        let farm_output = farm.workers_per_level * farm.outputs_per_worker_day[&GoodId("grain".into())];
        for (definition, level) in [(farm, population.div_ceil(farm_output)), (logging, 1), (tools, 1)] {
            snapshot.facilities.push(FacilityState {
                id: FacilityId(uuid::Uuid::now_v7()), province: province.id.clone(), definition: definition.id.clone(), level,
                production_priority: if definition.id == farm.id { 0 } else if definition.id == logging.id { 10 } else { 20 },
            });
        }
    }
    snapshot.normalize();
    let scenario = Scenario { schema_version: SCENARIO_SCHEMA_VERSION, id: "geographic-sandbox".into(), name: "Geographic sandbox with authored economic starting conditions".into(), rules: WorldRules { staple_good: GoodId("grain".into()), consumption_per_person_day: 1 }, definitions, initial_state: snapshot };
    aggregate_scenario::validate_scenario(&scenario).map_err(|error| error.to_string())?;
    Ok(scenario)
}
