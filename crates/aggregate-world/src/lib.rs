//! Portable definitions and state shared by simulation modules and file formats.
//!
//! These types describe data only. Validation belongs to `aggregate-scenario`,
//! and state transitions belong to the Rust simulation modules.

mod definitions;
mod identifiers;
mod scenario;
mod snapshot_index;
mod state;
pub use snapshot_index::{CountryStatistics, WorldSnapshotIndex};

pub use definitions::{
    ConstructionDefinition, ContentDefinitions, FacilityDefinition, GoodDefinition, WorldRules,
};
pub use identifiers::{
    CountryId, FacilityDefinitionId, FacilityId, GoodId, PopulationGroupId, ProvinceId, RegionId,
    StateId,
};
pub use scenario::{SCENARIO_SCHEMA_VERSION, Scenario};
pub use state::{
    ConstructionProjectState, CountryState, FacilityState, PopulationGroupState, ProvinceState,
    WorldSnapshot,
};
