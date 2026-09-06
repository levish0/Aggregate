use super::{super::MapInspectionPanel, InspectionRoot};
use aggregate_ui::{components as ui, layout::UiPointerBlocker};
use bevy::prelude::*;

pub fn build(commands: &mut Commands, root: Entity) {
    let panel = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(60),
            top: px(84),
            bottom: px(48),
            width: px(390),
            max_width: percent(34),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    commands
        .entity(panel)
        .insert((MapInspectionPanel, UiPointerBlocker, InspectionRoot));
}
