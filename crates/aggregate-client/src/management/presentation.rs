use super::{ManagementSession, SessionFeedback};
use crate::state::InterfaceState;
use aggregate_simulation_core::{GoodsFlowCause, SimulationEvent};
use aggregate_world::{FacilityDefinition, FacilityDefinitionId, FacilityId, GoodId, ProvinceId};

pub fn content_name(state: &InterfaceState, kind: &str, id: &str, fallback: &str) -> String {
    state
        .localization
        .text(&format!("world-{kind}-{}", id.replace('_', "-")))
        .unwrap_or_else(|_| fallback.to_owned())
}

pub fn province_name(
    session: &ManagementSession,
    state: &InterfaceState,
    id: &ProvinceId,
) -> String {
    let province = session
        .index
        .provinces
        .get(id)
        .map(|position| &session.snapshot.provinces[*position]);
    province
        .map(|province| {
            province
                .name_key
                .as_ref()
                .and_then(|key| state.localization.text(key).ok())
                .unwrap_or_else(|| province.name.clone())
        })
        .unwrap_or_else(|| id.to_string())
}

pub fn facility_name(
    session: &ManagementSession,
    state: &InterfaceState,
    id: &FacilityDefinitionId,
) -> String {
    let name = session
        .definitions
        .facilities
        .iter()
        .find(|item| item.id == *id)
        .map(|item| item.name.as_str())
        .unwrap_or(&id.0);
    content_name(state, "facility", &id.0, name)
}

pub fn good_name(session: &ManagementSession, state: &InterfaceState, id: &GoodId) -> String {
    let name = session
        .definitions
        .goods
        .iter()
        .find(|item| item.id == *id)
        .map(|item| item.name.as_str())
        .unwrap_or(&id.0);
    content_name(state, "good", &id.0, name)
}

pub fn recipe_description(
    session: &ManagementSession,
    state: &InterfaceState,
    definition: &FacilityDefinition,
) -> String {
    let quantities = |goods: &std::collections::BTreeMap<GoodId, u64>| {
        if goods.is_empty() {
            return state.text("management-no-inputs");
        }
        goods
            .iter()
            .map(|(good, amount)| format!("{} {amount}", good_name(session, state, good)))
            .collect::<Vec<_>>()
            .join(" · ")
    };
    state.format(
        "management-recipe",
        &[
            ("cost", quantities(&definition.construction.goods)),
            ("workers", definition.construction.max_workers.to_string()),
            ("work", definition.construction.worker_days.to_string()),
            ("inputs", quantities(&definition.inputs_per_worker_day)),
            ("outputs", quantities(&definition.outputs_per_worker_day)),
        ],
    )
}

pub fn feedback(session: &ManagementSession, state: &InterfaceState) -> String {
    match &session.feedback {
        SessionFeedback::Ready => state.text("management-ready"),
        SessionFeedback::ConstructionStarted(definition) => state.format(
            "management-command-accepted",
            &[("facility", facility_name(session, state, definition))],
        ),
        SessionFeedback::Error(
            aggregate_simulation_core::SimulationError::InsufficientConstructionGoods {
                province,
                good,
                required,
                available,
            },
        ) => state.format(
            "management-insufficient-goods",
            &[
                ("province", province_name(session, state, province)),
                ("good", good_name(session, state, good)),
                ("required", required.to_string()),
                ("available", available.to_string()),
            ],
        ),
        SessionFeedback::Error(error) => {
            state.format("management-command-error", &[("error", error.to_string())])
        }
    }
}

pub fn instance_definition<'a>(
    session: &'a ManagementSession,
    id: &FacilityId,
) -> Option<&'a FacilityDefinitionId> {
    session
        .snapshot
        .facilities
        .iter()
        .find(|facility| facility.id == *id)
        .map(|facility| &facility.definition)
        .or_else(|| {
            session
                .snapshot
                .construction_projects
                .iter()
                .find(|project| project.facility_id == *id)
                .map(|project| &project.definition)
        })
}

pub fn event_text(
    session: &ManagementSession,
    state: &InterfaceState,
    event: &SimulationEvent,
) -> String {
    match event {
        SimulationEvent::ConstructionStarted { province, facility }
        | SimulationEvent::ConstructionCompleted { province, facility } => {
            let name = instance_definition(session, facility)
                .map(|id| facility_name(session, state, id))
                .unwrap_or_else(|| facility.to_string());
            state.format(
                if matches!(event, SimulationEvent::ConstructionStarted { .. }) {
                    "management-news-started"
                } else {
                    "management-news-completed"
                },
                &[
                    ("province", province_name(session, state, province)),
                    ("facility", name),
                ],
            )
        }
        SimulationEvent::FoodShortfall {
            province,
            good,
            amount,
        } => state.format(
            "management-news-shortage",
            &[
                ("province", province_name(session, state, province)),
                ("good", good_name(session, state, good)),
                ("amount", amount.to_string()),
            ],
        ),
    }
}

pub fn daily_goods_change(
    session: &ManagementSession,
    province: &ProvinceId,
    good: &GoodId,
) -> Option<i128> {
    session.last_report.as_ref().map(|report| {
        report
            .goods_flows
            .iter()
            .filter(|flow| flow.province == *province && flow.good == *good)
            .map(|flow| match flow.cause {
                GoodsFlowCause::ProductionOutput { .. } => i128::from(flow.amount),
                _ => -i128::from(flow.amount),
            })
            .sum()
    })
}
