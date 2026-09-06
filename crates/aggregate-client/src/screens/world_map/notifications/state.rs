use crate::management::NotificationEntry;

pub(super) struct Toast {
    pub entry: NotificationEntry,
    pub remaining: f32,
}

#[derive(Default)]
pub(super) struct NotificationState {
    pub session: uuid::Uuid,
    pub seen: u64,
    pub active: Vec<Toast>,
}

impl NotificationState {
    pub fn advance(&mut self, seconds: f32, hovered: impl Fn(u64) -> bool) {
        for toast in &mut self.active {
            if !hovered(toast.entry.sequence) { toast.remaining -= seconds; }
        }
        self.active.retain(|toast| toast.remaining > 0.);
    }

    pub fn receive(&mut self, session: uuid::Uuid, entries: &[NotificationEntry]) {
        if self.session != session { *self = Self { session, ..Default::default() }; }
        for entry in entries.iter().filter(|entry| entry.sequence > self.seen) {
            self.active.push(Toast { entry: entry.clone(), remaining: 6. });
        }
        if let Some(entry) = entries.last() { self.seen = self.seen.max(entry.sequence); }
        if self.active.len() > 3 { self.active.drain(..self.active.len() - 3); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aggregate_simulation_core::SimulationEvent;

    #[test]
    fn notifications_expire_pause_on_hover_and_do_not_reappear_from_history() {
        let session = uuid::Uuid::now_v7();
        let entries: Vec<_> = (1..=4).map(|sequence| NotificationEntry {
            sequence, day: 1, event: SimulationEvent::FoodShortfall {
                province: "00000000-0000-0000-0000-000000000001".parse().unwrap(), good: "grain".into(), amount: 5,
            },
        }).collect();
        let mut state = NotificationState::default();
        state.receive(session, &entries);
        assert_eq!(state.active.len(), 3);
        state.advance(7., |sequence| sequence == 4);
        assert_eq!(state.active.len(), 1);
        state.receive(session, &entries);
        assert_eq!(state.active.len(), 1);
        state.advance(7., |_| false);
        assert!(state.active.is_empty());
        state.receive(uuid::Uuid::now_v7(), &[]);
        assert_eq!(state.seen, 0);
    }
}
