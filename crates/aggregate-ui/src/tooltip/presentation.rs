use super::{
    TooltipContent, TooltipSettings, TooltipState,
    panel::{self, LockIndicator, LockSegment, TooltipPanel, TooltipParent},
    state::TooltipPhase,
};
use crate::{
    button::{ButtonActivated, KeyboardFocus, UiButton},
    fonts::UiFonts,
};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    input_focus::InputFocusVisible,
    prelude::*,
};

#[derive(QueryData)]
pub(crate) struct TooltipTarget {
    entity: Entity,
    interaction: &'static Interaction,
    content: &'static TooltipContent,
    node: &'static ComputedNode,
    transform: &'static UiGlobalTransform,
    parent: Option<&'static TooltipParent>,
}

#[derive(SystemParam)]
pub(crate) struct TooltipInput<'w> {
    focus: Res<'w, KeyboardFocus>,
    focus_visible: Res<'w, InputFocusVisible>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
}

pub(crate) fn update_tooltips(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    input: TooltipInput,
    time: Res<Time<Real>>,
    window: Single<&Window>,
    scale: Res<UiScale>,
    settings: Res<TooltipSettings>,
    targets: Query<TooltipTarget>,
    panels: Query<(Entity, &TooltipPanel, &ComputedNode, &UiGlobalTransform)>,
    mut labels: Query<(&mut Text, &LockIndicator)>,
    mut segments: Query<(&mut BackgroundColor, &LockSegment)>,
    mut links: Query<(&TooltipParent, &mut UiButton)>,
    mut activated: MessageReader<ButtonActivated>,
    mut state: ResMut<TooltipState>,
) {
    state.dismissed_this_frame = false;
    state.retain_sources(|entity| targets.contains(entity));
    let over_panel = window.physical_cursor_position().and_then(|cursor| {
        panels
            .iter()
            .filter(|(_, _, node, transform)| physical_rect(node, transform).contains(cursor))
            .filter_map(|(_, panel, _, _)| {
                state
                    .entries
                    .iter()
                    .position(|entry| entry.source == panel.source)
                    .map(|depth| (depth, panel.source))
            })
            .max_by_key(|(depth, _)| *depth)
            .map(|(_, source)| source)
    });
    let hovered = targets
        .iter()
        .filter(|target| {
            matches!(
                *target.interaction,
                Interaction::Hovered | Interaction::Pressed
            ) && target.parent.is_none_or(|parent| {
                state
                    .entries
                    .iter()
                    .any(|entry| entry.source == parent.0 && entry.phase == TooltipPhase::Locked)
            })
        })
        .max_by_key(|target| target.parent.is_some())
        .map(|target| target.entity);
    let active = hovered.or_else(|| {
        if over_panel.is_none() && input.focus_visible.0 {
            input.focus.0.filter(|entity| targets.contains(*entity))
        } else {
            None
        }
    });
    if let Some(source) = active
        && let Ok(target) = targets.get(source)
    {
        state.enter(source, target.parent.map(|parent| parent.0));
    }
    for event in activated.read() {
        if let Ok(target) = targets.get(event.0) {
            state.activate(event.0, target.parent.map(|parent| parent.0));
        }
    }
    if input.keys.just_pressed(KeyCode::Escape) {
        state.dismiss_deepest();
    } else if input.mouse.just_pressed(MouseButton::Left)
        && over_panel.is_none()
        && hovered.is_none()
    {
        state.clear();
    } else {
        state.advance(active, over_panel, time.delta_secs(), &settings);
    }
    for (entity, panel, _, _) in &panels {
        if !state
            .entries
            .iter()
            .any(|entry| entry.source == panel.source && entry.phase != TooltipPhase::Waiting)
        {
            commands.entity(entity).despawn();
        }
    }
    for (depth, entry) in state.entries.iter().enumerate() {
        if entry.phase == TooltipPhase::Waiting {
            continue;
        }
        let Ok(target) = targets.get(entry.source) else {
            continue;
        };
        if !panels
            .iter()
            .any(|(_, panel, _, _)| panel.source == entry.source)
        {
            let physical = physical_rect(target.node, target.transform);
            let factor = target.node.inverse_scale_factor();
            panel::spawn_panel(
                &mut commands,
                &fonts,
                entry.source,
                target.content,
                depth,
                Rect::from_corners(physical.min * factor, physical.max * factor),
                Vec2::new(window.width(), window.height()) / scale.0,
                &entry.phase,
            );
        }
    }
    for (mut text, indicator) in &mut labels {
        if let Some(entry) = state
            .entries
            .iter()
            .find(|entry| entry.source == indicator.0)
            && let Ok(target) = targets.get(entry.source)
        {
            let value = if entry.phase == TooltipPhase::Locked {
                &target.content.locked_label
            } else {
                &target.content.locking_label
            };
            if text.0 != *value {
                text.0.clone_from(value);
            }
        }
    }
    for (mut color, segment) in &mut segments {
        if let Some(entry) = state
            .entries
            .iter()
            .find(|entry| entry.source == segment.source)
        {
            color.set_if_neq(BackgroundColor(panel::segment_color(
                segment.index,
                entry.progress(&settings),
            )));
        }
    }
    for (parent, mut button) in &mut links {
        button.enabled = state
            .entries
            .iter()
            .any(|entry| entry.source == parent.0 && entry.phase == TooltipPhase::Locked);
    }
}

fn physical_rect(node: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    Rect::from_center_size(transform.translation, node.size())
}
