//! Draggable, resizable inspection windows in logical UI coordinates.
use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Component)]
pub struct FloatingWindow {
    pub key: String,
    pub minimum_size: Vec2,
}

#[derive(Component)]
pub struct WindowDragHandle(pub Entity);
#[derive(Component)]
pub struct WindowResizeHandle(pub Entity);

#[derive(Clone, Copy)]
struct WindowGeometry {
    position: Vec2,
    size: Vec2,
}
struct WindowCapture {
    entity: Entity,
    pointer: Vec2,
    geometry: WindowGeometry,
    resizing: bool,
}

#[derive(Resource, Default)]
pub struct WindowInteraction {
    capture: Option<WindowCapture>,
    positions: BTreeMap<String, WindowGeometry>,
    next_order: i32,
}

pub fn interact(
    mut commands: Commands,
    windows: Query<&Window>,
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
        Option<&WindowDragHandle>,
        Option<&WindowResizeHandle>,
    )>,
    buttons: Query<(&ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let Ok(window) = windows.single() else {
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
    if mouse.just_pressed(MouseButton::Left) {
        let target = floating
            .iter()
            .filter(|(_, _, node, transform, _, _)| {
                node.size().min_element() > 0.
                    && Rect::from_center_size(transform.translation, node.size()).contains(cursor)
            })
            .max_by_key(|(_, _, _, _, _, order)| order.map_or(0, |order| order.0));
        if let Some((entity, _, node, transform, _, _)) = target {
            interaction.next_order += 1;
            commands
                .entity(entity)
                .insert(GlobalZIndex(20 + interaction.next_order));
            let handle = handles.iter().find_map(|(node, transform, drag, resize)| {
                if !Rect::from_center_size(transform.translation, node.size()).contains(cursor) {
                    return None;
                }
                if resize.is_some_and(|handle| handle.0 == entity) {
                    Some(true)
                } else if drag.is_some_and(|handle| handle.0 == entity) {
                    Some(false)
                } else {
                    None
                }
            });
            let over_button = buttons.iter().any(|(node, transform)| {
                node.size().min_element() > 0.
                    && Rect::from_center_size(transform.translation, node.size()).contains(cursor)
            });
            if let Some(resizing) = handle
                && (resizing || !over_button)
            {
                interaction.capture = Some(WindowCapture {
                    entity,
                    pointer,
                    resizing,
                    geometry: WindowGeometry {
                        position: (transform.translation - node.size() / 2.) / factor,
                        size: node.size() / factor,
                    },
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
        if capture.resizing {
            geometry.size = (geometry.size + delta)
                .max(window.minimum_size)
                .min((viewport - geometry.position).max(window.minimum_size));
        } else {
            geometry.position = (geometry.position + delta)
                .clamp(Vec2::ZERO, (viewport - geometry.size).max(Vec2::ZERO));
        }
        apply_geometry(&mut node, geometry);
        interaction.positions.insert(window.key.clone(), geometry);
    }
    for (_, window, computed, _, mut node, _) in &mut floating {
        if computed.size().min_element() == 0.
            && let Some(geometry) = interaction.positions.get(&window.key)
        {
            apply_geometry(&mut node, *geometry);
        }
    }
}

fn apply_geometry(node: &mut Node, geometry: WindowGeometry) {
    node.left = px(geometry.position.x);
    node.top = px(geometry.position.y);
    node.right = Val::Auto;
    node.bottom = Val::Auto;
    node.width = px(geometry.size.x);
    node.height = px(geometry.size.y);
}
