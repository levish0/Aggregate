use super::{ManagementLabel, ProjectProgressFill};
use crate::{
    management::{
        ManagementAction, ManagementSession, ManagementViewState, SessionFeedback, presentation,
    },
    state::{InterfaceState, Screen},
};
use aggregate_ui::{button::UiButton, theme};
use bevy::prelude::*;

pub fn update_management_labels(
    state: Res<InterfaceState>,
    session: Res<ManagementSession>,
    view: Res<ManagementViewState>,
    mut labels: Query<(&mut Text, &mut TextColor, Option<&ManagementLabel>)>,
    mut buttons: Query<(&ManagementAction, &Children, &mut UiButton)>,
    mut progress: Query<(&ProjectProgressFill, &mut Node)>,
    new_labels: Query<Entity, Added<ManagementLabel>>,
) {
    if state.screen != Screen::Management {
        return;
    }
    if !state.is_changed() && !session.is_changed() && !view.is_changed() && new_labels.is_empty() {
        return;
    }
    let Some(province) = session
        .snapshot
        .provinces
        .iter()
        .find(|province| province.id == view.selected_province)
    else {
        return;
    };
    let report = session.last_report.as_ref().and_then(|report| {
        report
            .provinces
            .iter()
            .find(|report| report.province == province.id)
    });
    for (mut text, mut color, binding) in &mut labels {
        let Some(binding) = binding else {
            continue;
        };
        let value = match binding {
            ManagementLabel::Country => session
                .snapshot
                .countries
                .iter()
                .find(|country| country.id == session.player_country)
                .map(|country| {
                    presentation::content_name(&state, "country", &country.id.0, &country.name)
                })
                .unwrap_or_default(),
            ManagementLabel::Day => state.format(
                "management-day",
                &[("day", session.snapshot.day.to_string())],
            ),
            ManagementLabel::Population | ManagementLabel::Workforce => {
                let total: u64 = session
                    .snapshot
                    .population_groups
                    .iter()
                    .filter(|group| {
                        session.snapshot.provinces.iter().any(|province| {
                            province.id == group.province
                                && province.country == session.player_country
                        })
                    })
                    .map(|group| {
                        if matches!(binding, ManagementLabel::Population) {
                            group.population
                        } else {
                            group.workforce
                        }
                    })
                    .sum();
                total.to_string()
            }
            ManagementLabel::Province => {
                presentation::province_name(&session, &state, &province.id)
            }
            ManagementLabel::Allocation => report
                .map(|report| {
                    state.format(
                        "management-allocation",
                        &[
                            ("production", report.production_workers.to_string()),
                            ("construction", report.construction_workers.to_string()),
                            (
                                "idle",
                                (report.available_workers
                                    - report.production_workers
                                    - report.construction_workers)
                                    .to_string(),
                            ),
                            ("workers", report.available_workers.to_string()),
                        ],
                    )
                })
                .unwrap_or_else(|| {
                    let workers: u64 = session
                        .snapshot
                        .population_groups
                        .iter()
                        .filter(|group| group.province == province.id)
                        .map(|group| group.workforce)
                        .sum();
                    state.format(
                        "management-before-allocation",
                        &[("workers", workers.to_string())],
                    )
                }),
            ManagementLabel::ReportDay => session
                .last_report
                .as_ref()
                .map(|report| {
                    state.format("management-report-day", &[("day", report.day.to_string())])
                })
                .unwrap_or_else(|| state.text("management-no-report")),
            ManagementLabel::FoodShortfall => report
                .map(|report| {
                    state.format(
                        "management-food",
                        &[
                            ("required", report.food_required.to_string()),
                            ("consumed", report.food_consumed.to_string()),
                            ("shortfall", report.food_shortfall.to_string()),
                        ],
                    )
                })
                .unwrap_or_else(|| state.text("management-no-report")),
            ManagementLabel::Feedback => {
                let next_color = if matches!(session.feedback, SessionFeedback::Error(_)) {
                    Color::srgb(1., 0.57, 0.45)
                } else {
                    theme::GOLD_BRIGHT
                };
                if color.0 != next_color {
                    color.0 = next_color;
                }
                presentation::feedback(&session, &state)
            }
            ManagementLabel::Stock(good) => province
                .stockpile
                .get(good)
                .copied()
                .unwrap_or(0)
                .to_string(),
            ManagementLabel::StockChange(good) => {
                presentation::daily_goods_change(&session, &province.id, good)
                    .map(|change| {
                        state.format(
                            "management-stock-change",
                            &[("amount", format!("{change:+}"))],
                        )
                    })
                    .unwrap_or_else(|| "—".into())
            }
            ManagementLabel::FacilityStaffing(definition) => {
                let facilities: Vec<_> = session
                    .snapshot
                    .facilities
                    .iter()
                    .filter(|facility| {
                        facility.province == province.id && facility.definition == *definition
                    })
                    .collect();
                let level: u64 = facilities.iter().map(|facility| facility.level).sum();
                let active = session
                    .last_report
                    .as_ref()
                    .map(|report| {
                        report
                            .facilities
                            .iter()
                            .filter(|result| {
                                facilities
                                    .iter()
                                    .any(|facility| facility.id == result.facility)
                            })
                            .map(|result| result.active_workers)
                            .sum::<u64>()
                            .to_string()
                    })
                    .unwrap_or_else(|| "—".into());
                state.format(
                    "management-facility-staffing",
                    &[("level", level.to_string()), ("workers", active)],
                )
            }
            ManagementLabel::ProjectProgress(id) => session
                .snapshot
                .construction_projects
                .iter()
                .find(|project| project.facility_id == *id)
                .map(|project| {
                    state.format(
                        "management-project-progress",
                        &[
                            ("work", project.remaining_worker_days.to_string()),
                            ("workers", project.requested_workers.to_string()),
                        ],
                    )
                })
                .unwrap_or_default(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (action, children, mut button) in &mut buttons {
        match action {
            ManagementAction::SelectProvince(id) => {
                let selected = *id == view.selected_province;
                if button.selected != selected {
                    button.selected = selected;
                }
            }
            ManagementAction::ToggleRunning => {
                let value = state.text(if session.running {
                    "management-pause"
                } else {
                    "management-play"
                });
                for child in children.iter() {
                    if let Ok((mut text, _, _)) = labels.get_mut(child)
                        && text.0 != value
                    {
                        text.0.clone_from(&value);
                    }
                }
            }
            _ => {}
        }
    }
    for (id, mut node) in &mut progress {
        if let Some(project) = session
            .snapshot
            .construction_projects
            .iter()
            .find(|project| project.facility_id == id.0)
            && let Some(definition) = session
                .definitions
                .facilities
                .iter()
                .find(|definition| definition.id == project.definition)
        {
            let completed = 1.
                - project.remaining_worker_days as f64 / definition.construction.worker_days as f64;
            let width = percent((completed * 100.) as f32);
            if node.width != width {
                node.width = width;
            }
        }
    }
}
