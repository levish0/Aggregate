use crate::{
    HealthParameters,
    model::{HealthGroup, HealthState},
};
use aggregate_programs::*;
use aggregate_world::*;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct PublicHealthProgram {
    pub parameters: HealthParameters,
}

impl SimulationProgram for PublicHealthProgram {
    fn manifest(&self) -> ProgramManifest {
        ProgramManifest {
            id: "aggregate.public_health".into(),
            version: semver::Version::new(0, 1, 0),
            api_version: PROGRAM_API_VERSION,
            state_schema_version: 1,
            dependencies: vec![],
            conflicts: vec![],
        }
    }
    fn initialize(&self, world: &WorldSnapshot) -> Result<Value, String> {
        if self.parameters.transmission_per_thousand > 1000
            || self.parameters.recovery_per_thousand > 1000
            || self.parameters.initial_infected_per_thousand > 1000
        {
            return Err("health parameters must be between 0 and 1000".into());
        }
        let groups = world
            .population_groups
            .iter()
            .map(|group| {
                let infected = (u128::from(group.population)
                    * u128::from(self.parameters.initial_infected_per_thousand)
                    / 1000) as u64;
                (
                    group.id.clone(),
                    HealthGroup {
                        susceptible: group.population.saturating_sub(infected),
                        infected,
                        recovered: 0,
                        exposure_remainder: 0,
                        recovery_remainder: 0,
                    },
                )
            })
            .collect();
        serde_json::to_value(HealthState {
            parameters: self.parameters.clone(),
            groups,
        })
        .map_err(|error| error.to_string())
    }
    fn validate_state(&self, world: &WorldSnapshot, state: &Value) -> Result<(), String> {
        let state: HealthState =
            serde_json::from_value(state.clone()).map_err(|error| error.to_string())?;
        let p = &state.parameters;
        if p.transmission_per_thousand > 1000
            || p.recovery_per_thousand > 1000
            || p.initial_infected_per_thousand > 1000
            || state.groups.len() != world.population_groups.len()
        {
            return Err("invalid health parameters or population group membership".into());
        }
        for group in &world.population_groups {
            let health = state.groups.get(&group.id).ok_or("missing health group")?;
            if u128::from(health.susceptible)
                + u128::from(health.infected)
                + u128::from(health.recovered)
                != u128::from(group.population)
                || health.exposure_remainder >= 1_000_000
                || health.recovery_remainder >= 1000
            {
                return Err(format!("invalid health compartments for {}", group.id));
            }
        }
        Ok(())
    }
    fn plan_day(&self, context: &ProgramContext<'_>, state: &Value) -> Result<ProgramPlan, String> {
        let mut next: HealthState =
            serde_json::from_value(state.clone()).map_err(|error| error.to_string())?;
        // Mixing occurs within each province. Infection neither appears spontaneously
        // nor crosses closed populations; interprovince transport can be a dependency later.
        let mut totals = BTreeMap::<&ProvinceId, (u128, u128)>::new();
        for group in &context.world.population_groups {
            let total = totals.entry(&group.province).or_default();
            total.0 += u128::from(group.population);
            total.1 += u128::from(next.groups[&group.id].infected);
        }
        let mut limits = BTreeMap::new();
        for group in &context.world.population_groups {
            let health = next
                .groups
                .get_mut(&group.id)
                .ok_or("missing health group")?;
            let (population, infected) = totals[&group.province];
            if population == 0 {
                continue;
            }
            let pressure = infected * 1_000_000 / population
                * u128::from(next.parameters.transmission_per_thousand)
                / 1000;
            let exposure =
                u128::from(health.susceptible) * pressure + u128::from(health.exposure_remainder);
            let infections = (exposure / 1_000_000).min(u128::from(health.susceptible)) as u64;
            health.exposure_remainder = (exposure % 1_000_000) as u64;
            let recovery = u128::from(health.infected)
                * u128::from(next.parameters.recovery_per_thousand)
                + u128::from(health.recovery_remainder);
            let recoveries = (recovery / 1000).min(u128::from(health.infected)) as u64;
            health.recovery_remainder = (recovery % 1000) as u64;
            health.susceptible -= infections;
            health.infected = health.infected - recoveries + infections;
            health.recovered += recoveries;
            let effective = u128::from(group.workforce)
                * u128::from(group.population - health.infected)
                / u128::from(group.population);
            limits.insert(group.id.clone(), effective as u64);
        }
        Ok(ProgramPlan {
            next_state: serde_json::to_value(next).map_err(|error| error.to_string())?,
            workforce_limits: limits,
        })
    }
    fn inspect(
        &self,
        scope: &InspectionScope,
        world: &WorldSnapshot,
        state: &Value,
    ) -> Result<Vec<InspectionSection>, String> {
        let state: HealthState =
            serde_json::from_value(state.clone()).map_err(|error| error.to_string())?;
        let provinces: BTreeSet<_> = match scope {
            InspectionScope::Country(country) => world
                .provinces
                .iter()
                .filter(|province| &province.country == country)
                .map(|province| &province.id)
                .collect(),
            InspectionScope::State { provinces, .. } => provinces.iter().collect(),
            InspectionScope::Province(province) => BTreeSet::from([province]),
        };
        let mut counts = [0u64; 3];
        for group in world
            .population_groups
            .iter()
            .filter(|group| provinces.contains(&group.province))
        {
            let health = &state.groups[&group.id];
            for (total, value) in
                counts
                    .iter_mut()
                    .zip([health.susceptible, health.infected, health.recovered])
            {
                *total = total
                    .checked_add(value)
                    .ok_or("health inspection total overflow")?;
            }
        }
        Ok(vec![InspectionSection {
            id: "aggregate.public_health.population".into(),
            title_key: "program-health-title".into(),
            metrics: [
                "program-health-susceptible",
                "program-health-infected",
                "program-health-recovered",
            ]
            .into_iter()
            .zip(counts)
            .map(|(key, count)| InspectionMetric {
                label_key: key.into(),
                value: MetricValue::Quantity(count),
            })
            .collect(),
        }])
    }
}
