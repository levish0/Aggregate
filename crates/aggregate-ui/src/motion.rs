//! Small stateful transitions using Bevy's easing curves. No independent animation engine.
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct MotionPreferences {
    pub reduced: bool,
}

#[derive(Debug, Clone)]
pub struct MotionValue {
    current: f32,
    start: f32,
    target: f32,
    elapsed: f32,
    duration: f32,
    easing: EaseFunction,
}

impl MotionValue {
    pub(crate) fn current(&self) -> f32 {
        self.current
    }
    pub fn new(value: f32) -> Self {
        Self {
            current: value,
            start: value,
            target: value,
            elapsed: 1.,
            duration: 1.,
            easing: EaseFunction::QuinticOut,
        }
    }

    /// Retarget from the displayed value, never from a stale start or queued animation.
    pub fn retarget(&mut self, target: f32, duration: f32, easing: EaseFunction) {
        if self.target == target {
            return;
        }
        self.start = self.current;
        self.target = target;
        self.elapsed = 0.;
        self.duration = duration;
        self.easing = easing;
    }

    pub fn advance(&mut self, seconds: f32, reduced: bool) -> f32 {
        self.elapsed = (self.elapsed + seconds).min(self.duration);
        if reduced || self.duration <= 0. || self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.current = self.target;
        } else {
            self.current = EasingCurve::new(self.start, self.target, self.easing)
                .sample_clamped(self.elapsed / self.duration);
        }
        self.current
    }
}

#[derive(Component)]
pub struct PanelEntrance {
    progress: MotionValue,
    offset: Vec2,
}

impl PanelEntrance {
    pub fn new(offset: Vec2) -> Self {
        let mut progress = MotionValue::new(0.);
        progress.retarget(1., 0.34, EaseFunction::QuinticOut);
        Self { progress, offset }
    }
}

pub fn animate_panels(
    time: Res<Time<Real>>,
    preferences: Res<MotionPreferences>,
    mut panels: Query<(&mut PanelEntrance, &mut UiTransform)>,
) {
    for (mut entrance, mut transform) in &mut panels {
        let progress = entrance
            .progress
            .advance(time.delta_secs(), preferences.reduced);
        transform.translation = Val2::px(
            entrance.offset.x * (1. - progress),
            entrance.offset.y * (1. - progress),
        );
        transform.scale = Vec2::splat(0.99 + 0.01 * progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_transition_keeps_its_current_value_and_reaches_new_target() {
        let mut value = MotionValue::new(0.);
        value.retarget(1., 0.3, EaseFunction::QuinticOut);
        let before = value.advance(0.06, false);
        value.retarget(0., 0.3, EaseFunction::QuinticOut);
        assert_eq!(value.advance(0., false), before);
        assert_eq!(value.advance(0.3, false), 0.);
    }

    #[test]
    fn reduced_motion_finishes_immediately_and_frame_rates_agree() {
        let mut slow = MotionValue::new(0.);
        slow.retarget(1., 0.3, EaseFunction::QuinticOut);
        let mut fast = slow.clone();
        for _ in 0..3 {
            slow.advance(0.04, false);
        }
        for _ in 0..12 {
            fast.advance(0.01, false);
        }
        assert!((slow.current - fast.current).abs() < 0.00001);
        assert_eq!(slow.advance(0., true), 1.);
        assert_eq!(slow.advance(0.01, false), 1.);
    }
}
