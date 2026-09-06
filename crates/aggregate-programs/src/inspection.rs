use aggregate_world::{CountryId, ProvinceId, StateId};

/// Domain scope; render entities and screen coordinates never enter a program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InspectionScope {
    Country(CountryId),
    State {
        id: StateId,
        country: CountryId,
        provinces: Vec<ProvinceId>,
    },
    Province(ProvinceId),
}

#[derive(Clone)]
pub enum MetricValue {
    Quantity(u64),
    Ratio { numerator: u64, denominator: u64 },
    Localized(String),
    Unavailable,
}

#[derive(Clone)]
pub struct InspectionMetric {
    pub label_key: String,
    pub value: MetricValue,
}

/// A program contributes facts; the native client owns layout and localization.
#[derive(Clone)]
pub struct InspectionSection {
    pub id: String,
    pub title_key: String,
    pub metrics: Vec<InspectionMetric>,
}
