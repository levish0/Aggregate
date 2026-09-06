use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

pub const PROGRAM_API_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgramDependency {
    pub id: String,
    pub version: VersionReq,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgramManifest {
    /// Authored namespaced key, for example `community.health`; not an entity UUID.
    pub id: String,
    pub version: Version,
    pub api_version: u32,
    pub state_schema_version: u32,
    pub dependencies: Vec<ProgramDependency>,
    pub conflicts: Vec<String>,
}

impl ProgramManifest {
    pub fn validate(&self) -> Result<(), String> {
        let valid_id = |id: &str| {
            id.contains('.')
                && id.split('.').all(|part| {
                    !part.is_empty()
                        && part.bytes().all(|byte| {
                            byte.is_ascii_lowercase()
                                || byte.is_ascii_digit()
                                || byte == b'_'
                                || byte == b'-'
                        })
                })
        };
        if !valid_id(&self.id)
            || self.api_version != PROGRAM_API_VERSION
            || self.state_schema_version == 0
        {
            return Err(format!(
                "invalid identity, API or state schema for program {}",
                self.id
            ));
        }
        let mut dependencies = std::collections::BTreeSet::new();
        for dependency in &self.dependencies {
            if !valid_id(&dependency.id)
                || dependency.id == self.id
                || !dependencies.insert(&dependency.id)
            {
                return Err(format!(
                    "invalid or duplicate dependency {} in {}",
                    dependency.id, self.id
                ));
            }
        }
        for conflict in &self.conflicts {
            if !valid_id(conflict) || conflict == &self.id {
                return Err(format!("invalid conflict {conflict} in {}", self.id));
            }
        }
        Ok(())
    }
}
