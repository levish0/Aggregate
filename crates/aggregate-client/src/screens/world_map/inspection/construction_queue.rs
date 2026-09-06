use crate::{
    management::{ManagementSession, presentation},
    state::InterfaceState,
};
use aggregate_ui::{components as ui, fonts::UiFonts, theme};
use aggregate_world::{FacilityId, ProvinceId};
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Component)]
pub struct QueueScope {
    provinces: BTreeSet<ProvinceId>,
    page: usize,
}
#[derive(Component)]
pub struct QueueBody;
#[derive(Component)]
pub struct QueueStatus(FacilityId);
#[derive(Component)]
pub struct QueueProgress(FacilityId);
#[derive(Component)]
pub struct QueueProgressBar(FacilityId);

pub(super) fn build(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    interface: &InterfaceState,
    session: &ManagementSession,
    provinces: &BTreeSet<ProvinceId>,
    page: usize,
) {
    super::buildings::build(
        commands, parent, fonts, interface, session, provinces, true, true,
    );
    ui::rule(commands, parent);
    let root = ui::node(
        commands,
        parent,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
    );
    commands.entity(root).insert(QueueScope {
        provinces: provinces.clone(),
        page,
    });
}

pub fn update(
    mut commands: Commands,
    session: Res<ManagementSession>,
    interface: Res<InterfaceState>,
    fonts: Res<UiFonts>,
    scopes: Query<(Entity, &QueueScope)>,
    bodies: Query<Entity, With<QueueBody>>,
    mut statuses: Query<(&QueueStatus, &mut Text), Without<QueueProgress>>,
    mut progress: Query<(&QueueProgress, &mut Text)>,
    mut bars: Query<(&QueueProgressBar, &mut Node)>,
    mut rendered: Local<Option<(Entity, Vec<FacilityId>)>>,
    mut previous: Local<Option<(Entity, u64)>>,
) {
    let Ok((root, scope)) = scopes.single() else {
        return;
    };
    let revision = (root, session.revision);
    if previous.as_ref() == Some(&revision) {
        return;
    }
    *previous = Some(revision);
    let mut rows = BTreeMap::new();
    for project in session
        .snapshot
        .construction_projects
        .iter()
        .filter(|project| scope.provinces.contains(&project.province))
    {
        rows.insert(
            project.facility_id.clone(),
            (&project.province, &project.definition, Some(project)),
        );
    }
    for (id, pending) in &session.pending_construction {
        if scope.provinces.contains(&pending.province) {
            rows.entry(id.clone())
                .or_insert((&pending.province, &pending.definition, None));
        }
    }
    let reports: BTreeMap<_, _> = session
        .last_report
        .iter()
        .flat_map(|report| &report.constructions)
        .map(|report| (&report.facility, report))
        .collect();
    let values = |id: &FacilityId| {
        let (_, definition, project) = rows[id];
        let status = reports
            .get(id)
            .map(|report| {
                interface.format(
                    "inspection-construction-applied",
                    &[
                        (
                            "work",
                            (u128::from(report.active_workers)
                                + u128::from(report.sector_construction_points))
                            .to_string(),
                        ),
                        ("amount", report.sector_construction_points.to_string()),
                    ],
                )
            })
            .unwrap_or_else(|| interface.text("management-project-awaiting-allocation"));
        let total = session
            .definitions
            .facilities
            .iter()
            .find(|item| item.id == *definition)
            .map(|item| item.construction.construction_points)
            .unwrap_or(1);
        let fraction = project.map_or(0., |project| {
            1. - project.remaining_construction_points as f64 / total as f64
        });
        (status, fraction)
    };
    let key = (root, rows.keys().cloned().collect::<Vec<_>>());
    if rendered.as_ref() != Some(&key) {
        *rendered = Some(key);
        for entity in &bodies {
            commands.entity(entity).despawn();
        }
        let body = ui::node(
            &mut commands,
            root,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
        );
        commands.entity(body).insert(QueueBody);
        if rows.is_empty() {
            ui::text(
                &mut commands,
                body,
                &fonts,
                interface.text("inspection-queue-empty"),
                14.,
                theme::MUTED,
                false,
            );
        }
        let range = super::content::list_page(
            &mut commands,
            body,
            &fonts,
            &interface,
            scope.page,
            rows.len(),
        );
        for (id, (province, definition, _)) in rows.iter().skip(range.start).take(range.len()) {
            let (status, fraction) = values(id);
            let card = ui::panel(
                &mut commands,
                body,
                Node {
                    width: percent(100),
                    padding: UiRect::all(px(7)),
                    row_gap: px(4),
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.,
                    ..default()
                },
            );
            let row = aggregate_ui::layout::row(&mut commands, card, 8.);
            commands.entity(row).insert(Node {
                width: percent(100),
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            });
            let name = format!(
                "{} · {}",
                presentation::province_name(&session, &interface, province),
                presentation::facility_name(&session, &interface, definition)
            );
            let title = ui::text(&mut commands, row, &fonts, name, 13., theme::TEXT, true);
            commands.entity(title).insert(Node {
                flex_grow: 1.,
                flex_shrink: 1.,
                ..default()
            });
            let label = ui::text(
                &mut commands,
                row,
                &fonts,
                format!("{:.0}%", fraction * 100.),
                12.,
                theme::TEXT,
                true,
            );
            commands.entity(label).insert(QueueProgress(id.clone()));
            let label = ui::text(
                &mut commands,
                card,
                &fonts,
                status,
                11.,
                theme::MUTED,
                false,
            );
            commands.entity(label).insert(QueueStatus(id.clone()));
            let track = ui::node(
                &mut commands,
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
                &mut commands,
                track,
                Node {
                    width: percent((fraction * 100.) as f32),
                    height: percent(100),
                    ..default()
                },
            );
            commands
                .entity(fill)
                .insert((BackgroundColor(theme::TEXT), QueueProgressBar(id.clone())));
        }
        return;
    }
    for (label, mut text) in &mut statuses {
        let value = values(&label.0).0;
        if text.0 != value {
            text.0 = value;
        }
    }
    for (label, mut text) in &mut progress {
        let value = format!("{:.0}%", values(&label.0).1 * 100.);
        if text.0 != value {
            text.0 = value;
        }
    }
    for (bar, mut node) in &mut bars {
        let width = percent((values(&bar.0).1 * 100.) as f32);
        if node.width != width {
            node.width = width;
        }
    }
}
