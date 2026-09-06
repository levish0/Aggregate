use aggregate_world::{FacilityId, GoodId, ProvinceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvinceDayReport {
    pub province: ProvinceId,
    pub population: u64,
    pub available_workers: u64,
    pub production_workers: u64,
    pub construction_workers: u64,
    pub food_required: u64,
    pub food_consumed: u64,
    pub food_shortfall: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityDayReport {
    pub facility: FacilityId,
    /// Initial allocation. Workers blocked by inputs may subsequently build instead.
    pub assigned_workers: u64,
    pub active_workers: u64,
    pub inputs: BTreeMap<GoodId, u64>,
    pub outputs: BTreeMap<GoodId, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructionDayReport {
    pub facility: FacilityId,
    pub requested_workers: u64,
    pub active_workers: u64,
    pub remaining_construction_points: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GoodsFlowCause {
    ProductionInput { facility: FacilityId },
    ProductionOutput { facility: FacilityId },
    HouseholdConsumption,
    ConstructionCost { facility: FacilityId },
}

/// A positive amount with an explicit source/sink cause; no translated text is stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoodsFlow {
    pub province: ProvinceId,
    pub good: GoodId,
    pub amount: u64,
    pub cause: GoodsFlowCause,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SimulationEvent {
    ConstructionStarted {
        province: ProvinceId,
        facility: FacilityId,
    },
    ConstructionCompleted {
        province: ProvinceId,
        facility: FacilityId,
    },
    FoodShortfall {
        province: ProvinceId,
        good: GoodId,
        amount: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayReport {
    pub day: u64,
    pub provinces: Vec<ProvinceDayReport>,
    pub facilities: Vec<FacilityDayReport>,
    pub constructions: Vec<ConstructionDayReport>,
    pub goods_flows: Vec<GoodsFlow>,
    pub events: Vec<SimulationEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    pub sequence: u64,
    pub goods_flows: Vec<GoodsFlow>,
    pub event: SimulationEvent,
}
