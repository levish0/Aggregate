use crate::{
    management::{
        ManagementAction, ManagementSession, construction::construction_site, presentation,
    },
    state::InterfaceState,
};
use aggregate_ui::{
    button::UiButton, components as ui, fonts::UiFonts, theme, tooltip::TooltipContent,
};
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
    let mut queues = BTreeMap::new();
    for facility in session
        .snapshot
        .facilities
        .iter()
        .filter(|facility| provinces.contains(&facility.province))
    {
        *levels.entry(&facility.definition).or_insert(0u128) += u128::from(facility.level);
    }
    for project in session
        .snapshot
        .construction_projects
        .iter()
        .filter(|project| provinces.contains(&project.province))
    {
        *queues.entry(&project.definition).or_insert(0u64) += 1;
    }
    let owned = session.snapshot.provinces.iter().any(|province| {
        provinces.contains(&province.id) && province.country == session.player_country
    });
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
    for (order, definition) in session.definitions.facilities.iter().enumerate() {
        let name = presentation::facility_name(session, interface, &definition.id);
        let card = ui::panel(
            commands,
            grid,
            Node {
                width: percent(48),
                min_height: px(170),
                padding: UiRect::all(px(10)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                flex_shrink: 0.,
                ..default()
            },
        );
        aggregate_ui::icon::icon(
            commands,
            card,
            aggregate_ui::icon::Icon::Buildings,
            32.,
            theme::TEXT,
        );
        ui::text(commands, card, fonts, &name, 14., theme::TEXT, true);
        ui::text(
            commands,
            card,
            fonts,
            interface.format(
                "inspection-building-level",
                &[(
                    "level",
                    levels.get(&definition.id).copied().unwrap_or(0).to_string(),
                )],
            ),
            18.,
            theme::TEXT,
            true,
        );
        if let Some(count) = queues.get(&definition.id) {
            ui::text(
                commands,
                card,
                fonts,
                format!("{} · {count}", interface.text("inspection-construction")),
                12.,
                theme::MUTED,
                false,
            );
        }
        commands.entity(card).insert(TooltipContent {
            title: name,
            body: presentation::recipe_description(session, interface, definition),
            hint: String::new(),
            locking_label: interface.text("tooltip-locking"),
            locked_label: interface.text("tooltip-locked"),
            links: vec![],
        });
        if owned {
            let site = construction_site(session, provinces, definition);
            let mut style = UiButton::secondary(400 + order as u32);
            style.enabled = site.is_some();
            let button = ui::button(commands, card, fonts, "+", style);
            if let Some(province) = site {
                commands
                    .entity(button)
                    .insert(ManagementAction::StartConstructionAt {
                        province,
                        definition: definition.id.clone(),
                    });
            } else {
                ui::text(
                    commands,
                    card,
                    fonts,
                    interface.text("inspection-material-shortage"),
                    11.,
                    theme::MUTED,
                    false,
                );
            }
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
    if let crate::management::SessionFeedback::Error(_) = &session.feedback {
        ui::text(
            commands,
            parent,
            fonts,
            presentation::feedback(session, interface),
            13.,
            theme::TEXT,
            false,
        );
    }
}
