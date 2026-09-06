use crate::{
    ScenarioError,
    validation::{add_checked, require_positive, validate_name, validate_unique_uuid_identifiers},
};
use aggregate_world::{ContentDefinitions, CountryId, ProvinceId, WorldRules, WorldSnapshot};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn validate_state(
    definitions: &ContentDefinitions,
    rules: &WorldRules,
    state: &WorldSnapshot,
    path: &str,
) -> Result<(), ScenarioError> {
    validate_unique_uuid_identifiers(
        state.countries.iter().map(|country| country.id.0),
        &format!("{path}.countries"),
        "id",
    )?;
    validate_unique_uuid_identifiers(
        state.provinces.iter().map(|province| province.id.0),
        &format!("{path}.provinces"),
        "id",
    )?;
    validate_unique_uuid_identifiers(
        state.population_groups.iter().map(|group| group.id.0),
        &format!("{path}.population_groups"),
        "id",
    )?;
    validate_unique_uuid_identifiers(
        state.facilities.iter().map(|facility| facility.id.0),
        &format!("{path}.facilities"),
        "id",
    )?;
    validate_unique_uuid_identifiers(
        state
            .construction_projects
            .iter()
            .map(|project| project.facility_id.0),
        &format!("{path}.construction_projects"),
        "facility_id",
    )?;

    let known_goods: BTreeSet<_> = definitions.goods.iter().map(|good| &good.id).collect();
    let known_countries: BTreeSet<_> = state.countries.iter().map(|country| &country.id).collect();
    let known_provinces: BTreeMap<_, _> = state
        .provinces
        .iter()
        .map(|province| (&province.id, province))
        .collect();
    for (index, country) in state.countries.iter().enumerate() {
        validate_name(&country.name, &format!("{path}.countries[{index}].name"))?;
    }
    for (index, province) in state.provinces.iter().enumerate() {
        let province_path = format!("{path}.provinces[{index}]");
        validate_name(&province.name, &format!("{province_path}.name"))?;
        if !known_countries.contains(&province.country) {
            return Err(ScenarioError::new(
                format!("{province_path}.country"),
                format!("unknown country `{}`", province.country),
            ));
        }
        for good in province.stockpile.keys() {
            if !known_goods.contains(good) {
                return Err(ScenarioError::new(
                    format!("{province_path}.stockpile[{:?}]", good.0),
                    format!("unknown good `{good}`"),
                ));
            }
        }
    }

    let mut province_population: BTreeMap<&ProvinceId, PopulationTotals> = BTreeMap::new();
    let mut country_population: BTreeMap<&CountryId, PopulationTotals> = BTreeMap::new();
    for (index, group) in state.population_groups.iter().enumerate() {
        let group_path = format!("{path}.population_groups[{index}]");
        let province = known_provinces.get(&group.province).ok_or_else(|| {
            ScenarioError::new(
                format!("{group_path}.province"),
                format!("unknown province `{}`", group.province),
            )
        })?;
        if group.workforce > group.population {
            return Err(ScenarioError::new(
                format!("{group_path}.workforce"),
                "workforce cannot exceed population",
            ));
        }
        let consumption = group
            .population
            .checked_mul(rules.consumption_per_person_day)
            .ok_or_else(|| {
                ScenarioError::new(
                    format!("{group_path}.population"),
                    "daily consumption exceeds the supported unsigned 64-bit range",
                )
            })?;
        province_population
            .entry(&group.province)
            .or_default()
            .add(group.population, group.workforce, consumption, &group_path)?;
        country_population
            .entry(&province.country)
            .or_default()
            .add(group.population, group.workforce, consumption, &group_path)?;
    }

    let facility_definitions: BTreeMap<_, _> = definitions
        .facilities
        .iter()
        .map(|definition| (&definition.id, definition))
        .collect();
    let mut province_facility_capacity = BTreeMap::<&ProvinceId, u64>::new();
    let mut country_facility_capacity = BTreeMap::<&CountryId, u64>::new();
    for (index, facility) in state.facilities.iter().enumerate() {
        let facility_path = format!("{path}.facilities[{index}]");
        let province = known_provinces.get(&facility.province).ok_or_else(|| {
            ScenarioError::new(
                format!("{facility_path}.province"),
                format!("unknown province `{}`", facility.province),
            )
        })?;
        let definition = facility_definitions
            .get(&facility.definition)
            .ok_or_else(|| {
                ScenarioError::new(
                    format!("{facility_path}.definition"),
                    format!("unknown facility definition `{}`", facility.definition),
                )
            })?;
        require_positive(facility.level, &format!("{facility_path}.level"))?;
        let capacity = definition
            .workers_per_level
            .checked_mul(facility.level)
            .ok_or_else(|| {
                ScenarioError::new(
                    format!("{facility_path}.level"),
                    "facility worker capacity exceeds the supported unsigned 64-bit range",
                )
            })?;
        add_checked(
            province_facility_capacity
                .entry(&facility.province)
                .or_default(),
            capacity,
            &format!("{facility_path}.level"),
        )?;
        add_checked(
            country_facility_capacity
                .entry(&province.country)
                .or_default(),
            capacity,
            &format!("{facility_path}.level"),
        )?;
    }

    let mut province_current_labor_demand = province_facility_capacity.clone();
    let mut province_completion_labor_demand = province_facility_capacity.clone();
    let existing_facilities: BTreeSet<_> = state
        .facilities
        .iter()
        .map(|facility| &facility.id)
        .collect();
    for (index, project) in state.construction_projects.iter().enumerate() {
        let project_path = format!("{path}.construction_projects[{index}]");
        if existing_facilities.contains(&project.facility_id) {
            return Err(ScenarioError::new(
                format!("{project_path}.facility_id"),
                "construction identifier already belongs to an existing facility",
            ));
        }
        let province = known_provinces.get(&project.province).ok_or_else(|| {
            ScenarioError::new(
                format!("{project_path}.province"),
                format!("unknown province `{}`", project.province),
            )
        })?;
        let definition = facility_definitions
            .get(&project.definition)
            .ok_or_else(|| {
                ScenarioError::new(
                    format!("{project_path}.definition"),
                    format!("unknown facility definition `{}`", project.definition),
                )
            })?;
        require_positive(
            project.requested_workers,
            &format!("{project_path}.requested_workers"),
        )?;
        if project.requested_workers > definition.construction.max_workers {
            return Err(ScenarioError::new(
                format!("{project_path}.requested_workers"),
                "exceeds the definition's maximum construction workers",
            ));
        }
        require_positive(
            project.remaining_worker_days,
            &format!("{project_path}.remaining_worker_days"),
        )?;
        if project.remaining_worker_days > definition.construction.worker_days {
            return Err(ScenarioError::new(
                format!("{project_path}.remaining_worker_days"),
                "exceeds the definition's total construction work",
            ));
        }
        // Every project creates a level-one facility. Reserve representable
        // capacity for all completions before accepting any pending project.
        add_checked(
            province_facility_capacity
                .entry(&project.province)
                .or_default(),
            definition.workers_per_level,
            &format!("{project_path}.definition"),
        )?;
        add_checked(
            country_facility_capacity
                .entry(&province.country)
                .or_default(),
            definition.workers_per_level,
            &format!("{project_path}.definition"),
        )?;
        let construction_labor_demand =
            project.requested_workers.min(project.remaining_worker_days);
        add_checked(
            province_current_labor_demand
                .entry(&project.province)
                .or_default(),
            construction_labor_demand,
            &format!("{project_path}.requested_workers"),
        )?;
        // Mixed completion states must also fit: one project may increase labor
        // demand when finished while another continues to employ builders.
        add_checked(
            province_completion_labor_demand
                .entry(&project.province)
                .or_default(),
            construction_labor_demand.max(definition.workers_per_level),
            &project_path,
        )
        .map_err(|mut error| {
            error.message =
                "a construction completion could overflow province labor demand".to_owned();
            error
        })?;
    }
    Ok(())
}

#[derive(Default)]
struct PopulationTotals {
    population: u64,
    workforce: u64,
    consumption: u64,
}

impl PopulationTotals {
    fn add(
        &mut self,
        population: u64,
        workforce: u64,
        consumption: u64,
        path: &str,
    ) -> Result<(), ScenarioError> {
        add_checked(
            &mut self.population,
            population,
            &format!("{path}.population"),
        )?;
        add_checked(&mut self.workforce, workforce, &format!("{path}.workforce"))?;
        add_checked(
            &mut self.consumption,
            consumption,
            &format!("{path}.population"),
        )?;
        Ok(())
    }
}
