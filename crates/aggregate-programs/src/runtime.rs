use crate::{ProgramContext, ProgramManifest, SimulationProgram};
use aggregate_world::{PopulationGroupId, WorldSnapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedProgramState {
    pub manifest: ProgramManifest,
    pub payload: Value,
}

struct EnabledProgram {
    implementation: Arc<dyn SimulationProgram>,
    saved: SavedProgramState,
    execution: Option<Box<dyn crate::ProgramExecution>>,
}

#[derive(Default)]
pub struct ProgramRuntime {
    programs: Vec<EnabledProgram>,
}

pub struct PreparedPrograms {
    pub workforce_limits: BTreeMap<PopulationGroupId, u64>,
    states: Vec<Value>,
}

impl ProgramRuntime {
    pub fn install(&mut self, world: &mut bevy_ecs::world::World) -> Result<(), String> {
        for program in &mut self.programs {
            program.execution = program
                .implementation
                .install(world)
                .map_err(|error| format!("{} installation: {error}", program.saved.manifest.id))?;
        }
        Ok(())
    }

    pub fn prepare_ecs(
        &mut self,
        world: &mut bevy_ecs::world::World,
    ) -> Result<(), (&'static str, String)> {
        for program in &mut self.programs {
            if let Some(execution) = &mut program.execution {
                execution.prepare(world).map_err(|(phase, error)| {
                    (phase, format!("{}: {error}", program.saved.manifest.id))
                })?;
            }
        }
        Ok(())
    }

    pub fn commit_ecs(&mut self, world: &mut bevy_ecs::world::World, day: u64) -> crate::DayReport {
        let mut report = crate::DayReport {
            day,
            provinces: vec![],
            facilities: vec![],
            constructions: vec![],
            goods_flows: vec![],
            events: vec![],
        };
        for program in &mut self.programs {
            if let Some(execution) = &mut program.execution
                && let Some(mut contribution) = execution.commit(world)
            {
                report.provinces.append(&mut contribution.provinces);
                report.facilities.append(&mut contribution.facilities);
                report.constructions.append(&mut contribution.constructions);
                report.goods_flows.append(&mut contribution.goods_flows);
                report.events.append(&mut contribution.events);
            }
        }
        report
    }

    pub fn execute_command(
        &mut self,
        world: &mut bevy_ecs::world::World,
        command: &crate::SimulationCommand,
        sequence: u64,
        scenario: &aggregate_world::Scenario,
    ) -> Result<crate::CommandOutcome, crate::SimulationError> {
        let id = command.program_id();
        let execution = self
            .programs
            .iter_mut()
            .find(|program| program.saved.manifest.id == id)
            .and_then(|program| program.execution.as_mut())
            .ok_or_else(|| {
                crate::SimulationError::CommandRejected(format!(
                    "command requires enabled program {id}"
                ))
            })?;
        execution.execute_command(world, command, sequence, scenario)
    }

    /// Content only comes from the enabled loadout; dependencies are resolved first.
    pub fn collect_definitions(
        implementations: Vec<Arc<dyn SimulationProgram>>,
    ) -> Result<aggregate_world::ContentDefinitions, String> {
        let mut goods = BTreeMap::new();
        let mut facilities = BTreeMap::new();
        for (manifest, implementation) in resolve(implementations)? {
            let content = implementation.definitions();
            for good in content.goods {
                if goods.insert(good.id.clone(), good).is_some() {
                    return Err(format!("{} registers a duplicate good", manifest.id));
                }
            }
            for facility in content.facilities {
                if facilities.insert(facility.id.clone(), facility).is_some() {
                    return Err(format!("{} registers a duplicate facility", manifest.id));
                }
            }
        }
        Ok(aggregate_world::ContentDefinitions {
            goods: goods.into_values().collect(),
            facilities: facilities.into_values().collect(),
        })
    }
    /// Supplied programs are the enabled loadout. Dependencies must also be enabled.
    pub fn initialize(
        implementations: Vec<Arc<dyn SimulationProgram>>,
        world: &WorldSnapshot,
    ) -> Result<Self, String> {
        let programs = resolve(implementations)?.into_iter().map(|(manifest, implementation)| {
            let payload = implementation.initialize(world).map_err(|error| format!("{} initialization: {error}", manifest.id))?;
            implementation.validate_state(world, &payload).map_err(|error| format!("{} state: {error}", manifest.id))?;
            tracing::info!(program = %manifest.id, version = %manifest.version, "Program enabled");
            Ok(EnabledProgram { implementation, saved: SavedProgramState { manifest, payload }, execution: None })
        }).collect::<Result<Vec<_>, String>>()?;
        Ok(Self { programs })
    }

    /// Save requirements choose the enabled loadout; other installed providers stay off.
    pub fn restore(
        available: Vec<Arc<dyn SimulationProgram>>,
        saved: Vec<SavedProgramState>,
        world: &WorldSnapshot,
    ) -> Result<Self, String> {
        let mut providers = BTreeMap::new();
        for implementation in available {
            let manifest = implementation.manifest();
            manifest.validate()?;
            if providers
                .insert(manifest.id.clone(), implementation)
                .is_some()
            {
                return Err(format!("duplicate installed program {}", manifest.id));
            }
        }
        let mut snapshots = BTreeMap::new();
        let mut enabled = Vec::new();
        for state in saved {
            state.manifest.validate()?;
            let id = state.manifest.id.clone();
            let implementation = providers
                .remove(&id)
                .ok_or_else(|| format!("save requires unavailable program {id}"))?;
            if implementation.manifest() != state.manifest {
                return Err(format!(
                    "saved program {id} version/schema/manifest does not match; migration required"
                ));
            }
            implementation
                .validate_state(world, &state.payload)
                .map_err(|error| format!("{id} saved state: {error}"))?;
            if snapshots.insert(id.clone(), state).is_some() {
                return Err(format!("duplicate saved program {id}"));
            }
            enabled.push(implementation);
        }
        let programs = resolve(enabled)?
            .into_iter()
            .map(|(manifest, implementation)| EnabledProgram {
                implementation,
                execution: None,
                saved: snapshots
                    .remove(&manifest.id)
                    .expect("resolved saved program"),
            })
            .collect();
        Ok(Self { programs })
    }

    pub fn is_empty(&self) -> bool {
        self.programs.is_empty()
    }

    pub fn inspect(
        &self,
        scope: &crate::InspectionScope,
        world: &WorldSnapshot,
    ) -> Result<Vec<crate::InspectionSection>, String> {
        let mut sections = Vec::new();
        let mut identities = BTreeSet::new();
        for program in &self.programs {
            let id = &program.saved.manifest.id;
            for section in program
                .implementation
                .inspect(scope, world, &program.saved.payload)
                .map_err(|error| format!("{id} inspection: {error}"))?
            {
                if !section.id.starts_with(&format!("{id}."))
                    || !identities.insert(section.id.clone())
                {
                    return Err(format!(
                        "{id} has an invalid or duplicate inspection section {}",
                        section.id
                    ));
                }
                sections.push(section);
            }
        }
        Ok(sections)
    }

    pub fn snapshot(&self) -> Vec<SavedProgramState> {
        self.programs
            .iter()
            .map(|program| program.saved.clone())
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all, fields(day))]
    pub fn prepare_day(&self, world: &WorldSnapshot, day: u64) -> Result<PreparedPrograms, String> {
        let previous: BTreeMap<_, _> = self
            .programs
            .iter()
            .map(|program| (program.saved.manifest.id.as_str(), &program.saved.payload))
            .collect();
        let workforce: BTreeMap<_, _> = world
            .population_groups
            .iter()
            .map(|group| (&group.id, group.workforce))
            .collect();
        let mut proposed = PreparedPrograms {
            workforce_limits: BTreeMap::new(),
            states: Vec::new(),
        };
        for program in &self.programs {
            let id = &program.saved.manifest.id;
            let span = tracing::debug_span!("program_calculation", program = %id, version = %program.saved.manifest.version);
            let _entered = span.enter();
            let dependencies = program
                .saved
                .manifest
                .dependencies
                .iter()
                .map(|dependency| (dependency.id.as_str(), previous[dependency.id.as_str()]))
                .collect();
            let context = ProgramContext {
                day,
                world,
                dependencies,
            };
            let plan = program
                .implementation
                .plan_day(&context, &program.saved.payload)
                .map_err(|error| format!("{id} calculation: {error}"))?;
            program
                .implementation
                .validate_state(world, &plan.next_state)
                .map_err(|error| format!("{id} proposed state: {error}"))?;
            for (group, limit) in plan.workforce_limits {
                if workforce
                    .get(&group)
                    .is_none_or(|available| limit > *available)
                {
                    return Err(format!(
                        "{id} invalid workforce limit for population group {group}"
                    ));
                }
                proposed
                    .workforce_limits
                    .entry(group)
                    .and_modify(|current| *current = (*current).min(limit))
                    .or_insert(limit);
            }
            proposed.states.push(plan.next_state);
        }
        Ok(proposed)
    }

    /// Called only after the core day commits. No validation or fallible work remains.
    pub fn commit(&mut self, proposed: PreparedPrograms) {
        for (program, state) in self.programs.iter_mut().zip(proposed.states) {
            program.saved.payload = state;
        }
    }
}

type ResolvedProgram = (ProgramManifest, Arc<dyn SimulationProgram>);

fn resolve(
    implementations: Vec<Arc<dyn SimulationProgram>>,
) -> Result<Vec<ResolvedProgram>, String> {
    let mut programs = BTreeMap::new();
    for implementation in implementations {
        let manifest = implementation.manifest();
        manifest.validate()?;
        if programs
            .insert(manifest.id.clone(), (manifest.clone(), implementation))
            .is_some()
        {
            return Err(format!("duplicate enabled program {}", manifest.id));
        }
    }
    for (manifest, _) in programs.values() {
        for dependency in &manifest.dependencies {
            let Some((required, _)) = programs.get(&dependency.id) else {
                return Err(format!(
                    "{} requires disabled/missing program {}",
                    manifest.id, dependency.id
                ));
            };
            if !dependency.version.matches(&required.version) {
                return Err(format!(
                    "{} requires {} {}, found {}",
                    manifest.id, dependency.id, dependency.version, required.version
                ));
            }
        }
        for conflict in &manifest.conflicts {
            if programs.contains_key(conflict) {
                return Err(format!("{} conflicts with {conflict}", manifest.id));
            }
        }
    }
    let mut resolved = Vec::new();
    let mut completed = BTreeSet::new();
    while !programs.is_empty() {
        let ready = programs
            .iter()
            .find(|(_, (manifest, _))| {
                manifest
                    .dependencies
                    .iter()
                    .all(|dependency| completed.contains(&dependency.id))
            })
            .map(|(id, _)| id.clone());
        let Some(id) = ready else {
            return Err(format!(
                "program dependency cycle: {}",
                programs.keys().cloned().collect::<Vec<_>>().join(", ")
            ));
        };
        completed.insert(id.clone());
        resolved.push(programs.remove(&id).expect("ready program"));
    }
    Ok(resolved)
}
