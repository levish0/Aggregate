//! Composable single-select: root, trigger, content and items; no application dependency.
#[cfg(test)]
mod tests;
use crate::{
    button::{ButtonActivated, KeyboardFocus, UiButton},
    components,
    fonts::UiFonts,
    layout::{self, UiPointerBlocker},
    theme,
};
use bevy::{
    input_focus::{FocusCause, InputFocus},
    prelude::*,
};

#[derive(Component)]
pub struct Select {
    pub value: String,
    pub open: bool,
}
#[derive(Component)]
pub struct SelectTrigger {
    pub root: Entity,
}
#[derive(Component)]
pub struct SelectContent {
    pub root: Entity,
}
#[derive(Component)]
pub struct SelectItem {
    pub root: Entity,
    pub value: String,
    pub label: String,
}
#[derive(Message)]
pub struct SelectChanged {
    pub root: Entity,
    pub value: String,
}
#[derive(Resource, Default)]
pub struct SelectInteractionState {
    pub dismissed_this_frame: bool,
    pub any_open: bool,
}

pub fn root(
    commands: &mut Commands,
    parent: Entity,
    value: impl Into<String>,
    width: f32,
) -> Entity {
    let root = components::node(
        commands,
        parent,
        Node {
            width: px(width),
            position_type: PositionType::Relative,
            ..default()
        },
    );
    commands.entity(root).insert((
        Select {
            value: value.into(),
            open: false,
        },
        UiPointerBlocker,
    ));
    root
}

pub fn trigger(
    commands: &mut Commands,
    root: Entity,
    fonts: &UiFonts,
    label: &str,
    tab_order: u32,
) -> Entity {
    let entity = components::button(commands, root, fonts, label, UiButton::secondary(tab_order));
    commands.entity(entity).insert(SelectTrigger { root });
    let chevron = components::node(
        commands,
        entity,
        Node {
            width: px(7),
            height: px(7),
            position_type: PositionType::Absolute,
            right: px(15),
            border: UiRect {
                bottom: px(1.5),
                right: px(1.5),
                ..default()
            },
            ..default()
        },
    );
    commands.entity(chevron).insert((
        BorderColor::all(theme::ACCENT),
        UiTransform::from_rotation(Rot2::degrees(45.)),
    ));
    entity
}

pub fn content(commands: &mut Commands, root: Entity) -> Entity {
    content_at(commands, root, false)
}

/// Open above a bottom-mounted control, or below a normal form control.
pub fn content_at(commands: &mut Commands, root: Entity, above: bool) -> Entity {
    let entity = layout::scroll_area(
        commands,
        root,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: if above { Val::Auto } else { percent(100) },
            bottom: if above { percent(100) } else { Val::Auto },
            width: percent(100),
            max_height: px(280),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(5)),
            row_gap: px(3),
            border: UiRect::all(px(1)),
            ..default()
        },
    );
    commands.entity(entity).insert((
        SelectContent { root },
        crate::skin::PanelSkin::Panel,
        GlobalZIndex(100),
        BackgroundColor(theme::PANEL),
        BorderColor::all(theme::ACCENT),
        theme::shadow(),
    ));
    entity
}

pub fn item(
    commands: &mut Commands,
    content: Entity,
    root: Entity,
    fonts: &UiFonts,
    label: &str,
    value: impl Into<String>,
    tab_order: u32,
) -> Entity {
    let mut button = UiButton::secondary(tab_order);
    button.enabled = false;
    let entity = components::button(commands, content, fonts, label, button);
    commands.entity(entity).insert(SelectItem {
        root,
        value: value.into(),
        label: label.into(),
    });
    entity
}

pub fn interact(
    mut events: MessageReader<ButtonActivated>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    triggers: Query<(Entity, &SelectTrigger)>,
    mut roots: Query<(Entity, &mut Select, &ComputedNode, &UiGlobalTransform)>,
    mut contents: Query<(&SelectContent, &mut Node, &ComputedNode, &UiGlobalTransform)>,
    mut items: Query<(Entity, &SelectItem, &mut UiButton)>,
    mut changed: MessageWriter<SelectChanged>,
    mut focus: ResMut<KeyboardFocus>,
    mut input_focus: ResMut<InputFocus>,
    mut state: ResMut<SelectInteractionState>,
) {
    state.dismissed_this_frame = false;
    let events: Vec<_> = events.read().map(|event| event.0).collect();
    for entity in &events {
        if let Ok((_, trigger)) = triggers.get(*entity) {
            let will_open = roots
                .get(trigger.root)
                .is_ok_and(|(_, select, _, _)| !select.open);
            for (root, mut select, _, _) in &mut roots {
                select.open = root == trigger.root && will_open;
            }
            if will_open {
                let value = roots.get(trigger.root).unwrap().1.value.clone();
                if let Some((entity, _, _)) = items
                    .iter()
                    .find(|(_, item, _)| item.root == trigger.root && item.value == value)
                {
                    focus.0 = Some(entity);
                    input_focus.set(entity, FocusCause::Navigated);
                }
            }
        } else if let Ok((_, item, _)) = items.get(*entity)
            && let Ok((_, mut select, _, _)) = roots.get_mut(item.root)
            && select.open
        {
            select.value = item.value.clone();
            select.open = false;
            changed.write(SelectChanged {
                root: item.root,
                value: item.value.clone(),
            });
            restore_trigger(item.root, &triggers, &mut focus, &mut input_focus);
        }
    }
    for (root, mut select, node, transform) in &mut roots {
        if !select.open {
            continue;
        }
        let dismiss_key = keys.any_just_pressed([KeyCode::Escape, KeyCode::Tab]);
        let outside = mouse.just_pressed(MouseButton::Left)
            && !events.iter().any(|entity| {
                triggers
                    .get(*entity)
                    .is_ok_and(|(_, trigger)| trigger.root == root)
            })
            && window.physical_cursor_position().is_some_and(|cursor| {
                !Rect::from_center_size(transform.translation, node.size()).contains(cursor)
                    && !contents.iter().any(|(content, _, node, transform)| {
                        content.root == root
                            && Rect::from_center_size(transform.translation, node.size())
                                .contains(cursor)
                    })
            });
        if dismiss_key || outside {
            select.open = false;
            state.dismissed_this_frame = keys.just_pressed(KeyCode::Escape);
            restore_trigger(root, &triggers, &mut focus, &mut input_focus);
        } else if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::ArrowUp]) {
            let mut options: Vec<_> = items
                .iter()
                .filter(|(_, item, _)| item.root == root)
                .map(|(entity, _, button)| (button.tab_order, entity))
                .collect();
            options.sort_by_key(|(order, _)| *order);
            let options: Vec<_> = options.into_iter().map(|(_, entity)| entity).collect();
            if !options.is_empty() {
                let current = options
                    .iter()
                    .position(|entity| Some(*entity) == focus.0)
                    .unwrap_or(0);
                let next = if keys.just_pressed(KeyCode::ArrowUp) {
                    (current + options.len() - 1) % options.len()
                } else {
                    (current + 1) % options.len()
                };
                focus.0 = Some(options[next]);
                input_focus.set(options[next], FocusCause::Navigated);
            }
        }
    }
    state.any_open = roots.iter().any(|(_, select, _, _)| select.open);
    for (content, mut node, _, _) in &mut contents {
        node.display = if roots
            .get(content.root)
            .is_ok_and(|(_, select, _, _)| select.open)
        {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (_, item, mut button) in &mut items {
        button.enabled = roots
            .get(item.root)
            .is_ok_and(|(_, select, _, _)| select.open);
        button.selected = roots
            .get(item.root)
            .is_ok_and(|(_, select, _, _)| select.value == item.value);
    }
}

pub fn update_labels(
    roots: Query<&Select>,
    triggers: Query<(&SelectTrigger, &Children)>,
    items: Query<&SelectItem>,
    mut labels: Query<&mut Text, With<crate::button::ButtonLabel>>,
) {
    for (trigger, children) in &triggers {
        if let Ok(select) = roots.get(trigger.root)
            && let Some(item) = items
                .iter()
                .find(|item| item.root == trigger.root && item.value == select.value)
        {
            for child in children {
                if let Ok(mut text) = labels.get_mut(*child) {
                    let value = item.label.clone();
                    if text.0 != value {
                        text.0 = value;
                    }
                }
            }
        }
    }
}

fn restore_trigger(
    root: Entity,
    triggers: &Query<(Entity, &SelectTrigger)>,
    focus: &mut KeyboardFocus,
    input_focus: &mut InputFocus,
) {
    if let Some((entity, _)) = triggers.iter().find(|(_, trigger)| trigger.root == root) {
        focus.0 = Some(entity);
        input_focus.set(entity, FocusCause::Navigated);
    }
}
