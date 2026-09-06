use crate::{ModuleContext, ModuleManifest, SimulationModule};
use aggregate_world::{PopulationGroupId, WorldSnapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedModuleState {
    pub manifest: ModuleManifest,
    pub payload: Value,
}

struct EnabledModule {
    implementation: Arc<dyn SimulationModule>,
    saved: SavedModuleState,
}

#[derive(Default)]
pub struct ModuleRuntime {
    modules: Vec<EnabledModule>,
}

pub struct PreparedModules {
    pub workforce_limits: BTreeMap<PopulationGroupId, u64>,
    states: Vec<Value>,
}

impl ModuleRuntime {
    /// Supplied modules are the enabled loadout. Dependencies must also be enabled.
    pub fn initialize(
        implementations: Vec<Arc<dyn SimulationModule>>,
        world: &WorldSnapshot,
    ) -> Result<Self, String> {
        let modules = resolve(implementations)?.into_iter().map(|(manifest, implementation)| {
            let payload = implementation.initialize(world).map_err(|error| format!("{} initialization: {error}", manifest.id))?;
            implementation.validate_state(world, &payload).map_err(|error| format!("{} state: {error}", manifest.id))?;
            tracing::info!(module = %manifest.id, version = %manifest.version, "Module enabled");
            Ok(EnabledModule { implementation, saved: SavedModuleState { manifest, payload } })
        }).collect::<Result<Vec<_>, String>>()?;
        Ok(Self { modules })
    }

    /// Save requirements choose the enabled loadout; other installed providers stay off.
    pub fn restore(
        available: Vec<Arc<dyn SimulationModule>>,
        saved: Vec<SavedModuleState>,
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
                return Err(format!("duplicate installed module {}", manifest.id));
            }
        }
        let mut snapshots = BTreeMap::new();
        let mut enabled = Vec::new();
        for state in saved {
            state.manifest.validate()?;
            let id = state.manifest.id.clone();
            let implementation = providers
                .remove(&id)
                .ok_or_else(|| format!("save requires unavailable module {id}"))?;
            if implementation.manifest() != state.manifest {
                return Err(format!(
                    "saved module {id} version/schema/manifest does not match; migration required"
                ));
            }
            implementation
                .validate_state(world, &state.payload)
                .map_err(|error| format!("{id} saved state: {error}"))?;
            if snapshots.insert(id.clone(), state).is_some() {
                return Err(format!("duplicate saved module {id}"));
            }
            enabled.push(implementation);
        }
        let modules = resolve(enabled)?
            .into_iter()
            .map(|(manifest, implementation)| EnabledModule {
                implementation,
                saved: snapshots
                    .remove(&manifest.id)
                    .expect("resolved saved module"),
            })
            .collect();
        Ok(Self { modules })
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    pub fn snapshot(&self) -> Vec<SavedModuleState> {
        self.modules
            .iter()
            .map(|module| module.saved.clone())
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all, fields(day))]
    pub fn prepare_day(&self, world: &WorldSnapshot, day: u64) -> Result<PreparedModules, String> {
        let previous: BTreeMap<_, _> = self
            .modules
            .iter()
            .map(|module| (module.saved.manifest.id.as_str(), &module.saved.payload))
            .collect();
        let workforce: BTreeMap<_, _> = world
            .population_groups
            .iter()
            .map(|group| (&group.id, group.workforce))
            .collect();
        let mut proposed = PreparedModules {
            workforce_limits: BTreeMap::new(),
            states: Vec::new(),
        };
        for module in &self.modules {
            let id = &module.saved.manifest.id;
            let span = tracing::debug_span!("module_calculation", module = %id, version = %module.saved.manifest.version);
            let _entered = span.enter();
            let dependencies = module
                .saved
                .manifest
                .dependencies
                .iter()
                .map(|dependency| (dependency.id.as_str(), previous[dependency.id.as_str()]))
                .collect();
            let context = ModuleContext {
                day,
                world,
                dependencies,
            };
            let plan = module
                .implementation
                .plan_day(&context, &module.saved.payload)
                .map_err(|error| format!("{id} calculation: {error}"))?;
            module
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
    pub fn commit(&mut self, proposed: PreparedModules) {
        for (module, state) in self.modules.iter_mut().zip(proposed.states) {
            module.saved.payload = state;
        }
    }
}

type ResolvedModule = (ModuleManifest, Arc<dyn SimulationModule>);

fn resolve(implementations: Vec<Arc<dyn SimulationModule>>) -> Result<Vec<ResolvedModule>, String> {
    let mut modules = BTreeMap::new();
    for implementation in implementations {
        let manifest = implementation.manifest();
        manifest.validate()?;
        if modules
            .insert(manifest.id.clone(), (manifest.clone(), implementation))
            .is_some()
        {
            return Err(format!("duplicate enabled module {}", manifest.id));
        }
    }
    for (manifest, _) in modules.values() {
        for dependency in &manifest.dependencies {
            let Some((required, _)) = modules.get(&dependency.id) else {
                return Err(format!(
                    "{} requires disabled/missing module {}",
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
            if modules.contains_key(conflict) {
                return Err(format!("{} conflicts with {conflict}", manifest.id));
            }
        }
    }
    let mut resolved = Vec::new();
    let mut completed = BTreeSet::new();
    while !modules.is_empty() {
        let ready = modules
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
                "module dependency cycle: {}",
                modules.keys().cloned().collect::<Vec<_>>().join(", ")
            ));
        };
        completed.insert(id.clone());
        resolved.push(modules.remove(&id).expect("ready module"));
    }
    Ok(resolved)
}
