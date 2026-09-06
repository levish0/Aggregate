use bevy::prelude::*;

/// Durations use real seconds, independently of simulation speed or pause.
#[derive(Resource)]
pub struct TooltipSettings {
    pub show_delay: f32,
    pub lock_duration: f32,
    pub departure_grace: f32,
}

impl Default for TooltipSettings {
    fn default() -> Self {
        Self {
            show_delay: 0.25,
            lock_duration: 0.85,
            departure_grace: 0.18,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum TooltipPhase {
    Waiting,
    Visible,
    Locked,
}

pub(super) struct TooltipEntry {
    pub source: Entity,
    pub phase: TooltipPhase,
    elapsed: f32,
    away: f32,
    lockable: bool,
}

impl TooltipEntry {
    pub fn progress(&self, settings: &TooltipSettings) -> f32 {
        if self.phase == TooltipPhase::Locked {
            return 1.;
        }
        if settings.lock_duration <= 0. {
            return 1.;
        }
        ((self.elapsed - settings.show_delay) / settings.lock_duration).clamp(0., 1.)
    }
}

#[derive(Resource, Default)]
pub struct TooltipState {
    pub(super) entries: Vec<TooltipEntry>,
    suppressed: Option<Entity>,
    /// The client must not also navigate backwards for this Escape press.
    pub dismissed_this_frame: bool,
}

impl TooltipState {
    pub(super) fn enter(&mut self, source: Entity, parent: Option<Entity>, lockable: bool) {
        if self.suppressed == Some(source)
            || self.entries.iter().any(|entry| entry.source == source)
        {
            return;
        }
        if let Some(parent) = parent {
            let Some(index) = self
                .entries
                .iter()
                .position(|entry| entry.source == parent && entry.phase == TooltipPhase::Locked)
            else {
                return;
            };
            self.entries.truncate(index + 1);
        } else {
            self.entries.clear();
        }
        self.entries.push(TooltipEntry {
            source,
            phase: TooltipPhase::Waiting,
            elapsed: 0.,
            away: 0.,
            lockable,
        });
    }

    pub(super) fn activate(&mut self, source: Entity, parent: Option<Entity>) {
        self.suppressed = None;
        self.enter(source, parent, true);
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.source == source) {
            entry.phase = TooltipPhase::Locked;
        }
    }

    pub(super) fn advance(
        &mut self,
        active: Option<Entity>,
        over_panel: Option<Entity>,
        seconds: f32,
        settings: &TooltipSettings,
    ) {
        if active != self.suppressed {
            self.suppressed = None;
        }
        let Some(entry) = self.entries.last_mut() else {
            return;
        };
        if entry.phase == TooltipPhase::Locked {
            return;
        }
        if active == Some(entry.source) || over_panel == Some(entry.source) {
            entry.away = 0.;
            entry.elapsed += seconds;
            entry.phase = if entry.lockable
                && entry.elapsed >= settings.show_delay + settings.lock_duration
            {
                TooltipPhase::Locked
            } else if entry.elapsed >= settings.show_delay {
                TooltipPhase::Visible
            } else {
                TooltipPhase::Waiting
            };
        } else {
            entry.away += seconds;
            if entry.away >= settings.departure_grace {
                self.entries.pop();
            }
        }
    }

    pub(super) fn dismiss_deepest(&mut self) {
        if let Some(entry) = self.entries.pop() {
            self.suppressed = Some(entry.source);
            self.dismissed_this_frame = true;
        }
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.suppressed = None;
    }

    pub(super) fn retain_sources(&mut self, exists: impl Fn(Entity) -> bool) {
        if let Some(index) = self.entries.iter().position(|entry| !exists(entry.source)) {
            self.entries.truncate(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entity(index: u32) -> Entity {
        Entity::from_raw_u32(index).unwrap()
    }

    #[test]
    fn dwell_displays_then_locks_and_survives_departure() {
        let settings = TooltipSettings::default();
        let source = entity(1);
        let mut state = TooltipState::default();
        state.enter(source, None, true);
        state.advance(Some(source), None, 0.1, &settings);
        assert_eq!(state.entries[0].phase, TooltipPhase::Waiting);
        state.advance(Some(source), None, 0.4, &settings);
        assert_eq!(state.entries[0].phase, TooltipPhase::Visible);
        assert!(state.entries[0].progress(&settings) > 0.);
        state.advance(None, Some(source), 0.7, &settings);
        assert_eq!(state.entries[0].phase, TooltipPhase::Locked);
        state.advance(None, None, 10., &settings);
        assert_eq!(state.entries.len(), 1);
    }

    #[test]
    fn short_hint_never_locks_and_disappears_after_departure() {
        let settings = TooltipSettings::default();
        let source = entity(1);
        let mut state = TooltipState::default();
        state.enter(source, None, false);
        state.advance(Some(source), None, 10., &settings);
        assert_eq!(state.entries[0].phase, TooltipPhase::Visible);
        state.advance(None, None, settings.departure_grace, &settings);
        assert!(state.entries.is_empty());
    }

    #[test]
    fn nesting_requires_locked_parent_and_escape_dismisses_only_deepest() {
        let (root, child) = (entity(1), entity(2));
        let mut state = TooltipState::default();
        state.enter(root, None, true);
        state.enter(child, Some(root), true);
        assert_eq!(state.entries.len(), 1);
        state.activate(root, None);
        state.activate(child, Some(root));
        assert_eq!(state.entries.len(), 2);
        state.dismiss_deepest();
        state.enter(child, Some(root), true);
        assert_eq!(state.entries.len(), 1);
        assert!(state.dismissed_this_frame);
        state.activate(child, Some(root));
        assert_eq!(state.entries.len(), 2);
    }

    #[test]
    fn transient_departure_and_parent_removal_clean_up_descendants() {
        let settings = TooltipSettings::default();
        let (root, child) = (entity(1), entity(2));
        let mut state = TooltipState::default();
        state.enter(root, None, true);
        state.advance(Some(root), None, 0.4, &settings);
        state.advance(None, None, 0.1, &settings);
        assert_eq!(state.entries.len(), 1);
        state.advance(None, None, 0.1, &settings);
        assert!(state.entries.is_empty());
        state.activate(root, None);
        state.activate(child, Some(root));
        state.retain_sources(|source| source != root);
        assert!(state.entries.is_empty());
    }
}
