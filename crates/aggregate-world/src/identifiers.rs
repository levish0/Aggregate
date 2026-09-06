use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

/// Persistent identity of a country's administrative portion of a geographic region.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StateId(pub Uuid);

impl From<Uuid> for StateId {
    fn from(value: Uuid) -> Self { Self(value) }
}

impl FromStr for StateId {
    type Err = uuid::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> { Uuid::parse_str(value).map(Self) }
}

impl fmt::Display for StateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(formatter) }
}

/// Persistent country identity, independent of its name, tag or ECS allocation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CountryId(pub Uuid);

impl From<Uuid> for CountryId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl FromStr for CountryId {
    type Err = uuid::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}

impl fmt::Display for CountryId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Persistent province identity. Raster colors and render indices are separate lookup keys.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProvinceId(pub Uuid);

impl From<Uuid> for ProvinceId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl FromStr for ProvinceId {
    type Err = uuid::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}

/// Persistent administrative/geographic grouping of map provinces.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegionId(pub Uuid);

impl From<Uuid> for RegionId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl FromStr for RegionId {
    type Err = uuid::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}

impl fmt::Display for RegionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ProvinceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Persistent identity of one population-group instance, retained across save/resume.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PopulationGroupId(pub Uuid);

impl From<Uuid> for PopulationGroupId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl FromStr for PopulationGroupId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}

impl fmt::Display for PopulationGroupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Persistent facility identity assigned before construction and retained on completion.
/// Commands record the supplied UUID so replay never generates a replacement identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FacilityId(pub Uuid);

impl From<Uuid> for FacilityId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl FromStr for FacilityId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}

impl fmt::Display for FacilityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Authored facility kind such as `grain_farm`; many instances share one definition.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FacilityDefinitionId(pub String);

impl From<&str> for FacilityDefinitionId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl fmt::Display for FacilityDefinitionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Authored good kind such as `grain`, not an individual item or inventory batch.
/// Human-readable keys let recipes and presets refer to the same good definition.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GoodId(pub String);

impl From<&str> for GoodId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl fmt::Display for GoodId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
