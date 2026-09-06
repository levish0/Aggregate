use crate::{
    management::{
        ManagementAction, ManagementSession, construction::construction_site, presentation,
    },
    state::InterfaceState,
};
use aggregate_ui::{
    button::UiButton, components as ui, fonts::UiFonts, icon::Icon, theme, tooltip::TooltipContent,
};
use aggregate_world::{FacilityDefinitionId, ProvinceId};
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Component)]
pub struct BuildingScope(BTreeSet<ProvinceId>);
#[derive(Component)]
pub struct BuildingCard(FacilityDefinitionId);
#[derive(Component)]
pub struct BuildingButton(FacilityDefinitionId);
#[derive(Component)]
pub struct BuildingLabel(FacilityDefinitionId, BuildingField);
#[derive(Component)]
pub struct ConstructionFeedback;
#[derive(Clone, Copy)]
enum BuildingField {
    Level,
    Workforce,
    Output,
}

pub(super) fn build(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    interface: &InterfaceState,
    session: &ManagementSession,
    provinces: &BTreeSet<ProvinceId>,
    has_statistics: bool,
    sectors_only: bool,
) {
    if !has_statistics {
        return;
    }
    let grid = ui::node(
        commands,
        parent,
        Node {
            width: percent(100),
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(8),
            row_gap: px(8),
            ..default()
        },
    );
    commands
        .entity(grid)
        .insert(BuildingScope(provinces.clone()));
    let owned = session.snapshot.provinces.iter().any(|province| {
        provinces.contains(&province.id) && province.country == session.player_country
    });
    for (order, definition) in session
        .definitions
        .facilities
        .iter()
        .filter(|definition| !sectors_only || definition.construction_points_per_worker_day > 0)
        .enumerate()
    {
        let name = presentation::facility_name(session, interface, &definition.id);
        let card = ui::panel(
            commands,
            grid,
            Node {
                width: percent(if sectors_only { 100. } else { 48. }),
                min_height: px(if sectors_only { 90. } else { 170. }),
                padding: UiRect::all(px(10)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                flex_shrink: 0.,
                ..default()
            },
        );
        let header = aggregate_ui::layout::row(commands, card, 8.);
        commands.entity(header).insert(Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        });
        aggregate_ui::icon::icon(commands, header, Icon::Buildings, 24., theme::TEXT);
        let title = ui::text(commands, header, fonts, &name, 14., theme::TEXT, true);
        commands.entity(title).insert(Node {
            flex_grow: 1.,
            ..default()
        });
        for field in [
            BuildingField::Level,
            BuildingField::Workforce,
            BuildingField::Output,
        ] {
            let parent = if sectors_only && matches!(field, BuildingField::Level) {
                header
            } else if matches!(field, BuildingField::Workforce) {
                let row = aggregate_ui::layout::row(commands, card, 5.);
                aggregate_ui::icon::icon(commands, row, Icon::Population, 14., theme::TEXT);
                row
            } else {
                card
            };
            let label = ui::text(
                commands,
                parent,
                fonts,
                "",
                if matches!(field, BuildingField::Level) {
                    16.
                } else {
                    12.
                },
                theme::TEXT,
                false,
            );
            commands
                .entity(label)
                .insert(BuildingLabel(definition.id.clone(), field));
        }
        commands.entity(card).insert((
            BuildingCard(definition.id.clone()),
            TooltipContent {
                title: name,
                body: presentation::recipe_description(session, interface, definition),
                hint: String::new(),
                locking_label: interface.text("tooltip-locking"),
                locked_label: interface.text("tooltip-locked"),
                links: vec![],
            },
        ));
        if owned {
            let button = ui::button(
                commands,
                if sectors_only { header } else { card },
                fonts,
                "+",
                UiButton {
                    enabled: false,
                    ..UiButton::secondary(400 + order as u32)
                },
            );
            if sectors_only {
                commands.entity(button).insert(Node {
                    width: px(28),
                    height: px(28),
                    padding: UiRect::ZERO,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_shrink: 0.,
                    ..default()
                });
            }
            commands
                .entity(button)
                .insert(BuildingButton(definition.id.clone()));
        }
    }
    let feedback = ui::text(commands, parent, fonts, "", 12., theme::TEXT, false);
    commands.entity(feedback).insert(ConstructionFeedback);
}

pub fn update(
    mut commands: Commands,
    session: Res<ManagementSession>,
    interface: Res<InterfaceState>,
    scopes: Query<(Entity, &BuildingScope)>,
    mut labels: Query<(&BuildingLabel, &mut Text), Without<ConstructionFeedback>>,
    mut cards: Query<(&BuildingCard, &mut TooltipContent)>,
    mut buttons: Query<(Entity, &BuildingButton, &mut UiButton)>,
    mut feedback: Query<&mut Text, With<ConstructionFeedback>>,
    mut previous: Local<Option<(Entity, uuid::Uuid, u64)>>,
) {
    let Ok((root, scope)) = scopes.single() else {
        return;
    };
    let key = (root, session.session_id, session.revision);
    if previous.as_ref() == Some(&key) {
        return;
    }
    *previous = Some(key);
    let production = super::production_summary::summarize(&session, &scope.0);
    let mut levels = BTreeMap::<FacilityDefinitionId, u128>::new();
    let mut queued = BTreeMap::<FacilityDefinitionId, u128>::new();
    for facility in session
        .snapshot
        .facilities
        .iter()
        .filter(|item| scope.0.contains(&item.province))
    {
        *levels.entry(facility.definition.clone()).or_default() += u128::from(facility.level);
    }
    for project in session
        .snapshot
        .construction_projects
        .iter()
        .filter(|item| scope.0.contains(&item.province))
    {
        *queued.entry(project.definition.clone()).or_default() += 1;
    }
    for (id, pending) in &session.pending_construction {
        if scope.0.contains(&pending.province)
            && !session
                .snapshot
                .construction_projects
                .iter()
                .any(|project| project.facility_id == *id)
        {
            *queued.entry(pending.definition.clone()).or_default() += 1;
        }
    }
    let definitions: BTreeMap<_, _> = session
        .definitions
        .facilities
        .iter()
        .map(|item| (&item.id, item))
        .collect();
    let quantities = |goods: &BTreeMap<aggregate_world::GoodId, u128>| {
        goods
            .iter()
            .map(|(good, amount)| {
                format!(
                    "{} {amount}",
                    presentation::good_name(&session, &interface, good)
                )
            })
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let empty = super::production_summary::ProductionSummary::default();
    for (label, mut text) in &mut labels {
        let summary = production.get(&label.0).unwrap_or(&empty);
        let level = levels.get(&label.0).copied().unwrap_or(0);
        let count = queued.get(&label.0).copied().unwrap_or(0);
        let value = match label.1 {
            BuildingField::Level => {
                if count > 0 {
                    format!("{level} → {}", level + count)
                } else {
                    interface.format("inspection-building-level", &[("level", level.to_string())])
                }
            }
            BuildingField::Workforce => {
                if session.last_report.is_some() {
                    format!("{} / {}", summary.active_workers, summary.capacity)
                } else {
                    format!("— / {}", summary.capacity)
                }
            }
            BuildingField::Output => {
                if session.last_report.is_none() {
                    String::new()
                } else if definitions[&label.0].construction_points_per_worker_day > 0 {
                    interface.format(
                        "inspection-construction-service",
                        &[("work", summary.construction_points.to_string())],
                    )
                } else {
                    quantities(&summary.outputs)
                }
            }
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (card, mut tooltip) in &mut cards {
        let summary = production.get(&card.0).unwrap_or(&empty);
        let mut body = presentation::recipe_description(&session, &interface, definitions[&card.0]);
        if session.last_report.is_some() {
            body.push_str(&format!(
                "\n\n{}\n{}",
                interface.format(
                    "inspection-production-input",
                    &[("inputs", quantities(&summary.inputs))]
                ),
                interface.format(
                    "inspection-production-output",
                    &[("outputs", quantities(&summary.outputs))]
                )
            ));
        }
        if tooltip.body != body {
            tooltip.body = body;
        }
    }
    for (entity, button, mut style) in &mut buttons {
        let site = construction_site(&session, &scope.0, definitions[&button.0]);
        if style.enabled != site.is_some() {
            style.enabled = site.is_some();
        }
        if let Some(province) = site {
            commands
                .entity(entity)
                .insert(ManagementAction::StartConstructionAt {
                    province,
                    definition: button.0.clone(),
                });
        } else {
            commands.entity(entity).remove::<ManagementAction>();
        }
    }
    for mut text in &mut feedback {
        let value = if matches!(
            &session.feedback,
            crate::management::SessionFeedback::Error(_)
        ) {
            presentation::feedback(&session, &interface)
        } else {
            String::new()
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}
