use crate::motion::{MotionPreferences, MotionValue};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    ui::ComputedStackIndex,
};

/// A vertically scrollable Node whose wheel input and motion are owned by the UI layer.
#[derive(Component)]
pub struct ScrollRegion {
    pub(crate) target: f32,
    pub(crate) motion: MotionValue,
}

impl Default for ScrollRegion {
    fn default() -> Self {
        Self {
            target: 0.,
            motion: MotionValue::new(0.),
        }
    }
}

pub fn scroll_regions(
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window>,
    scale: Res<UiScale>,
    time: Res<Time<Real>>,
    preferences: Res<MotionPreferences>,
    mut regions: Query<(
        Entity,
        &ComputedNode,
        &UiGlobalTransform,
        &ComputedStackIndex,
        &mut ScrollRegion,
        &mut ScrollPosition,
    )>,
) {
    // Only the frontmost region consumes the wheel, including a tooltip above a page.
    let pointed = window.physical_cursor_position().and_then(|cursor| {
        regions
            .iter()
            .filter(|(_, node, transform, _, _, _)| {
                Rect::from_center_size(transform.translation, node.size()).contains(cursor)
            })
            .max_by_key(|(_, _, _, stack, _, _)| stack.0)
            .map(|(entity, _, _, _, _, _)| entity)
    });
    for event in wheel.read() {
        if let Some(entity) = pointed
            && let Ok((_, node, _, _, mut region, _)) = regions.get_mut(entity)
        {
            let delta = -event.y
                * if event.unit == MouseScrollUnit::Line {
                    38.
                } else {
                    1.
                }
                / scale.0;
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
            region.target = (region.target + delta).clamp(0., maximum);
        }
    }
    for (_, node, _, _, mut region, mut position) in &mut regions {
        let maximum =
            ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
        region.target = region.target.min(maximum);
        let target = region.target;
        region
            .motion
            .retarget(target, 0.22, EaseFunction::QuinticOut);
        position.y = region
            .motion
            .advance(time.delta_secs(), preferences.reduced)
            .clamp(0., maximum);
    }
}
