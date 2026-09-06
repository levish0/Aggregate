use super::{super::MapInspectionPanel, InspectionRoot};
use aggregate_ui::{components as ui, layout::UiPointerBlocker};
use bevy::prelude::*;

pub fn build(commands: &mut Commands, root: Entity) {
    let panel = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(92),
            top: px(94),
            bottom: px(140),
            width: px(390),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    commands.entity(panel).insert((
        MapInspectionPanel,
        UiPointerBlocker,
        InspectionRoot,
        aggregate_ui::window::FloatingWindow {
            key: "territory-inspection".into(),
            minimum_size: Vec2::new(340., 300.),
        },
    ));
}
