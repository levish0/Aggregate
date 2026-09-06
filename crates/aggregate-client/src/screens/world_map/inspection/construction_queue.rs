use super::components::metric;
use crate::{
    management::{ManagementSession, presentation},
    state::InterfaceState,
};
use aggregate_ui::{components as ui, fonts::UiFonts, theme};
use aggregate_world::ProvinceId;
use bevy::prelude::*;
use std::collections::BTreeSet;

pub(super) fn build(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    interface: &InterfaceState,
    session: &ManagementSession,
    provinces: &BTreeSet<ProvinceId>,
    page: usize,
) {
    let projects: Vec<_> = session
        .snapshot
        .construction_projects
        .iter()
        .filter(|project| provinces.contains(&project.province))
        .collect();
    if projects.is_empty() {
        ui::text(
            commands,
            parent,
            fonts,
            interface.text("inspection-queue-empty"),
            14.,
            theme::MUTED,
            false,
        );
        return;
    }
    let range = super::content::list_page(commands, parent, fonts, interface, page, projects.len());
    for project in &projects[range] {
        let card = ui::panel(
            commands,
            parent,
            Node {
                width: percent(100),
                padding: UiRect::all(px(12)),
                row_gap: px(7),
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.,
                ..default()
            },
        );
        ui::text(
            commands,
            card,
            fonts,
            presentation::facility_name(session, interface, &project.definition),
            16.,
            theme::TEXT,
            true,
        );
        ui::text(
            commands,
            card,
            fonts,
            presentation::province_name(session, interface, &project.province),
            12.,
            theme::MUTED,
            false,
        );
        let allocation = session.last_report.as_ref().and_then(|report| {
            report
                .constructions
                .iter()
                .find(|report| report.facility == project.facility_id)
        });
        let status = allocation
            .map(|report| {
                interface.format(
                    if report.active_workers < report.requested_workers {
                        "management-project-labor-shortage"
                    } else {
                        "management-project-workers"
                    },
                    &[
                        ("active", report.active_workers.to_string()),
                        ("requested", report.requested_workers.to_string()),
                    ],
                )
            })
            .unwrap_or_else(|| interface.text("management-project-awaiting-allocation"));
        ui::text(commands, card, fonts, status, 12., theme::MUTED, false);
        if let Some(definition) = session
            .definitions
            .facilities
            .iter()
            .find(|definition| definition.id == project.definition)
        {
            let fraction = 1.
                - project.remaining_worker_days as f64 / definition.construction.worker_days as f64;
            metric(
                commands,
                card,
                fonts,
                &interface.text("inspection-progress"),
                format!("{:.0}%", fraction * 100.),
            );
            let track = ui::node(
                commands,
                card,
                Node {
                    width: percent(100),
                    height: px(4),
                    ..default()
                },
            );
            commands
                .entity(track)
                .insert(BackgroundColor(theme::MUTED.with_alpha(0.2)));
            let fill = ui::node(
                commands,
                track,
                Node {
                    width: percent((fraction * 100.) as f32),
                    height: percent(100),
                    ..default()
                },
            );
            commands.entity(fill).insert(BackgroundColor(theme::TEXT));
        }
    }
}
