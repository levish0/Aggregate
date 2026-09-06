//! Small composable layout primitives built on Bevy's native flex layout.
use crate::{components, scroll::ScrollRegion};
use bevy::prelude::*;

#[derive(Component)]
pub struct UiPointerBlocker;

pub fn row(commands: &mut Commands, parent: Entity, gap: f32) -> Entity {
    components::node(
        commands,
        parent,
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(gap),
            align_items: AlignItems::Center,
            ..default()
        },
    )
}

pub fn column(commands: &mut Commands, parent: Entity, gap: f32) -> Entity {
    components::node(
        commands,
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(gap),
            ..default()
        },
    )
}

pub fn scroll_area(commands: &mut Commands, parent: Entity, layout: Node) -> Entity {
    let entity = components::node(
        commands,
        parent,
        Node {
            overflow: Overflow::scroll_y(),
            min_height: px(0),
            ..layout
        },
    );
    commands.entity(entity).insert((
        ScrollRegion::default(),
        ScrollPosition::default(),
        UiPointerBlocker,
    ));
    entity
}
