//! Draggable, resizable inspection windows in logical UI coordinates.
use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};
use std::collections::BTreeMap;
mod geometry;
use geometry::{ResizeEdges, WindowGeometry, apply_geometry};

#[derive(Component)]
pub struct FloatingWindow {
    pub key: String,
    pub minimum_size: Vec2,
}

#[derive(Component)]
pub struct WindowDragHandle(pub Entity);
struct WindowCapture {
    entity: Entity,
    pointer: Vec2,
    geometry: WindowGeometry,
    edges: Option<ResizeEdges>,
}

#[derive(Resource, Default)]
pub struct WindowInteraction {
    capture: Option<WindowCapture>,
    positions: BTreeMap<String, WindowGeometry>,
    next_order: i32,
    cursor: Option<SystemCursorIcon>,
}

impl WindowInteraction {
    pub fn is_captured(&self) -> bool { self.capture.is_some() }
}

pub fn interact(
    mut commands: Commands,
    windows: Query<(Entity, &Window)>,
    mouse: Res<ButtonInput<MouseButton>>,
    scale: Res<UiScale>,
    mut interaction: ResMut<WindowInteraction>,
    mut floating: Query<(
        Entity,
        &FloatingWindow,
        &ComputedNode,
        &UiGlobalTransform,
        &mut Node,
        Option<&GlobalZIndex>,
    )>,
    handles: Query<(
        &ComputedNode,
        &UiGlobalTransform,
        &WindowDragHandle,
    )>,
    buttons: Query<(&ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let Ok((window_entity, window)) = windows.single() else {
        return;
    };
    let factor = window.scale_factor() * scale.0;
    let viewport = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    ) / factor;
    if !mouse.pressed(MouseButton::Left) {
        interaction.capture = None;
    }
    let Some(cursor) = window.physical_cursor_position() else {
        return;
    };
    let pointer = cursor / factor;
    let mut cursor_icon = SystemCursorIcon::Default;
    {
        let target = floating
            .iter()
            .filter(|(_, _, node, transform, _, _)| {
                node.size().min_element() > 0.
                    && Rect::from_center_size(transform.translation, node.size()).contains(cursor)
            })
            .max_by_key(|(_, _, _, _, _, order)| order.map_or(0, |order| order.0));
        if let Some((entity, _, node, transform, _, _)) = target {
            let geometry = WindowGeometry {
                position: (transform.translation - node.size() / 2.) / factor,
                size: node.size() / factor,
            };
            let edges = ResizeEdges::at(pointer, geometry);
            let over_title = handles.iter().any(|(node, transform, drag)| {
                drag.0 == entity && Rect::from_center_size(transform.translation, node.size()).contains(cursor)
            });
            let over_button = buttons.iter().any(|(node, transform)| {
                node.size().min_element() > 0.
                    && Rect::from_center_size(transform.translation, node.size()).contains(cursor)
            });
            cursor_icon = edges.map_or_else(|| if over_title && !over_button { SystemCursorIcon::Grab } else { SystemCursorIcon::Default }, ResizeEdges::cursor);
            if mouse.just_pressed(MouseButton::Left) {
                interaction.next_order += 1;
                commands.entity(entity).insert(GlobalZIndex(20 + interaction.next_order));
            }
            if mouse.just_pressed(MouseButton::Left) && (edges.is_some() || (over_title && !over_button)) {
                interaction.capture = Some(WindowCapture {
                    entity,
                    pointer,
                    edges,
                    geometry,
                });
            }
        }
    }
    if let Some(capture) = &interaction.capture {
        let entity = capture.entity;
        let Ok((_, window, _, _, mut node, _)) = floating.get_mut(entity) else {
            interaction.capture = None;
            return;
        };
        let delta = pointer - capture.pointer;
        let mut geometry = capture.geometry;
        if let Some(edges) = capture.edges {
            geometry = edges.resize(geometry, delta, window.minimum_size, viewport);
            cursor_icon = edges.cursor();
        } else {
            cursor_icon = SystemCursorIcon::Grabbing;
            geometry.position = (geometry.position + delta)
                .clamp(Vec2::ZERO, (viewport - geometry.size).max(Vec2::ZERO));
        }
        apply_geometry(&mut node, geometry);
        interaction.positions.insert(window.key.clone(), geometry);
    }
    if interaction.cursor != Some(cursor_icon) {
        commands.entity(window_entity).insert(CursorIcon::System(cursor_icon));
        interaction.cursor = Some(cursor_icon);
    }
    for (_, window, computed, _, mut node, _) in &mut floating {
        if computed.size().min_element() == 0.
            && let Some(geometry) = interaction.positions.get(&window.key)
        {
            apply_geometry(&mut node, *geometry);
        }
    }
}
