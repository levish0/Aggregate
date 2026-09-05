use super::{ManagementLabel, ManagementList, ProjectProgressFill, layout};
use crate::{
    management::{ManagementSession, ManagementViewState, presentation},
    state::{InterfaceState, Screen},
};
use aggregate_ui::{components as ui, fonts::UiFonts, theme};
use aggregate_world::FacilityId;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct RenderedList {
    projects: Vec<FacilityId>,
    news_count: usize,
    initialized: bool,
}

/// Only list membership changes rebuild rows. Progress and totals update in place so
/// ticking cannot recreate buttons, reset scroll positions, or restart panel motion.
pub fn update_management_lists(
    mut commands: Commands,
    state: Res<InterfaceState>,
    session: Res<ManagementSession>,
    view: Res<ManagementViewState>,
    fonts: Res<UiFonts>,
    mut lists: Query<(
        Entity,
        &ManagementList,
        Option<&Children>,
        Option<&mut RenderedList>,
    )>,
    new_lists: Query<Entity, Added<ManagementList>>,
) {
    if state.screen != Screen::Management {
        return;
    }
    if !state.is_changed() && !session.is_changed() && !view.is_changed() && new_lists.is_empty() {
        return;
    }
    for (parent, kind, children, rendered) in &mut lists {
        let projects: Vec<_> = session
            .snapshot
            .construction_projects
            .iter()
            .filter(|project| project.province == view.selected_province)
            .map(|project| project.facility_id.clone())
            .collect();
        let unchanged = rendered.as_ref().is_some_and(|rendered| {
            rendered.initialized
                && match kind {
                    ManagementList::Projects => rendered.projects == projects,
                    ManagementList::News => rendered.news_count == session.news.len(),
                }
        });
        if unchanged {
            continue;
        }
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        match kind {
            ManagementList::Projects => {
                if projects.is_empty() {
                    ui::text(
                        &mut commands,
                        parent,
                        &fonts,
                        state.text("management-no-construction"),
                        14.,
                        theme::MUTED,
                        false,
                    );
                }
                for id in &projects {
                    let project = session
                        .snapshot
                        .construction_projects
                        .iter()
                        .find(|project| project.facility_id == *id)
                        .expect("selected project");
                    let row = layout::column(&mut commands, parent, 7.);
                    ui::text(
                        &mut commands,
                        row,
                        &fonts,
                        presentation::facility_name(&session, &state, &project.definition),
                        16.,
                        theme::TEXT,
                        true,
                    );
                    layout::label(
                        &mut commands,
                        row,
                        &fonts,
                        ManagementLabel::ProjectProgress(id.clone()),
                        13.,
                        theme::MUTED,
                    );
                    let track = ui::node(
                        &mut commands,
                        row,
                        Node {
                            width: percent(100),
                            height: px(5),
                            ..default()
                        },
                    );
                    commands.entity(track).insert(BackgroundColor(theme::INK));
                    let fill = ui::node(
                        &mut commands,
                        track,
                        Node {
                            width: percent(0),
                            height: percent(100),
                            ..default()
                        },
                    );
                    commands.entity(fill).insert((
                        BackgroundColor(theme::GOLD),
                        ProjectProgressFill(id.clone()),
                    ));
                }
            }
            ManagementList::News => {
                if session.news.is_empty() {
                    ui::text(
                        &mut commands,
                        parent,
                        &fonts,
                        state.text("management-no-news"),
                        14.,
                        theme::MUTED,
                        false,
                    );
                }
                for entry in session.news.iter().rev() {
                    let card = ui::panel(
                        &mut commands,
                        parent,
                        Node {
                            padding: UiRect::all(px(12)),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(8),
                            flex_shrink: 0.,
                            ..default()
                        },
                    );
                    ui::text(
                        &mut commands,
                        card,
                        &fonts,
                        state.format("management-day", &[("day", entry.day.to_string())]),
                        12.,
                        theme::GOLD,
                        true,
                    );
                    ui::text(
                        &mut commands,
                        card,
                        &fonts,
                        presentation::event_text(&session, &state, &entry.event),
                        14.,
                        theme::TEXT,
                        false,
                    );
                }
            }
        }
        commands.entity(parent).insert(RenderedList {
            projects,
            news_count: session.news.len(),
            initialized: true,
        });
    }
}
