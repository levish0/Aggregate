use super::components::metric;
use crate::{
    management::{
        ManagementAction, ManagementSession, construction::construction_site, presentation,
    },
    state::InterfaceState,
};
use aggregate_ui::{button::UiButton, components as ui, fonts::UiFonts, theme};
use aggregate_world::ProvinceId;
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn build(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    interface: &InterfaceState,
    session: &ManagementSession,
    provinces: &BTreeSet<ProvinceId>,
    has_statistics: bool,
) {
    if !has_statistics {
        return;
    }
    let mut levels = BTreeMap::new();
    let mut stocks = BTreeMap::new();
    for facility in session
        .snapshot
        .facilities
        .iter()
        .filter(|facility| provinces.contains(&facility.province))
    {
        *levels.entry(&facility.definition).or_insert(0u128) += u128::from(facility.level);
    }
    for province in session
        .snapshot
        .provinces
        .iter()
        .filter(|province| provinces.contains(&province.id))
    {
        for (good, quantity) in &province.stockpile {
            *stocks.entry(good).or_insert(0u128) += u128::from(*quantity);
        }
    }
    for (good, quantity) in stocks {
        metric(
            commands,
            parent,
            fonts,
            &presentation::good_name(session, interface, good),
            quantity.to_string(),
        );
    }
    ui::rule(commands, parent);
    let owned = session.snapshot.provinces.iter().any(|province| {
        provinces.contains(&province.id) && province.country == session.player_country
    });
    for (order, definition) in session.definitions.facilities.iter().enumerate() {
        let name = presentation::facility_name(session, interface, &definition.id);
        metric(
            commands,
            parent,
            fonts,
            &name,
            levels.get(&definition.id).copied().unwrap_or(0).to_string(),
        );
        if owned {
            let site = construction_site(session, provinces, definition);
            let mut style = UiButton::secondary(400 + order as u32);
            style.enabled = site.is_some();
            let button = ui::button(
                commands,
                parent,
                fonts,
                &interface.format("inspection-build-level", &[("building", name)]),
                style,
            );
            if let Some(province) = site {
                commands
                    .entity(button)
                    .insert(ManagementAction::StartConstructionAt {
                        province,
                        definition: definition.id.clone(),
                    });
            }
            ui::text(
                commands,
                parent,
                fonts,
                presentation::recipe_description(session, interface, definition),
                12.,
                theme::MUTED,
                false,
            );
        }
    }
    if owned {
        ui::text(
            commands,
            parent,
            fonts,
            interface.text("inspection-build-site"),
            12.,
            theme::MUTED,
            false,
        );
    }
    ui::rule(commands, parent);
    for project in session
        .snapshot
        .construction_projects
        .iter()
        .filter(|project| provinces.contains(&project.province))
    {
        let active = session
            .last_report
            .as_ref()
            .and_then(|report| {
                report
                    .constructions
                    .iter()
                    .find(|report| report.facility == project.facility_id)
            })
            .map(|report| report.active_workers)
            .unwrap_or(0);
        metric(
            commands,
            parent,
            fonts,
            &presentation::facility_name(session, interface, &project.definition),
            interface.format(
                "inspection-project-progress",
                &[
                    ("work", project.remaining_worker_days.to_string()),
                    ("active", active.to_string()),
                    ("requested", project.requested_workers.to_string()),
                ],
            ),
        );
    }
    if let crate::management::SessionFeedback::Error(error) = &session.feedback {
        ui::text(
            commands,
            parent,
            fonts,
            error.to_string(),
            13.,
            theme::TEXT,
            false,
        );
    }
}
