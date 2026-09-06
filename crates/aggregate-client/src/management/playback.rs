use super::{ManagementAction, ManagementSession};
use crate::state::{InterfaceState, Screen};
use aggregate_ui::{button::UiButton, components as ui, fonts::UiFonts};
use bevy::prelude::*;

/// Wall-clock pacing only. Every speed executes the same complete simulation day.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SimulationSpeed {
    #[default]
    One,
    Two,
    Three,
    Four,
    Five,
}

impl SimulationSpeed {
    pub const ALL: [Self; 5] = [Self::One, Self::Two, Self::Three, Self::Four, Self::Five];

    fn seconds_per_day(self) -> f32 {
        match self {
            Self::One => 1.,
            Self::Two => 0.5,
            Self::Three => 0.2,
            Self::Four => 0.1,
            Self::Five => 0.,
        }
    }
}

pub fn speed_controls(commands: &mut Commands, parent: Entity, fonts: &UiFonts, session: &ManagementSession, enabled: bool, order: u32) {
    for (index, speed) in SimulationSpeed::ALL.into_iter().enumerate() {
        let slot = ui::node(commands, parent, Node { width: px(40), flex_shrink: 0., ..default() });
        let mut style = UiButton::secondary(order + index as u32);
        style.enabled = enabled;
        style.selected = session.speed == speed;
        let button = ui::button(commands, slot, fonts, &(index + 1).to_string(), style);
        commands.entity(button).insert(ManagementAction::SetSpeed(speed));
    }
}

pub fn update_speed_controls(session: Res<ManagementSession>, mut buttons: Query<(&ManagementAction, &mut UiButton)>) {
    for (action, mut button) in &mut buttons {
        if let ManagementAction::SetSpeed(speed) = action {
            let selected = session.speed == *speed;
            if button.selected != selected { button.selected = selected; }
        }
    }
}

/// No catch-up burst after a stall. Speed five advances at most once per rendered
/// frame, so input handling continues between days even when the simulation is slow.
pub fn advance_running_session(
    state: Res<InterfaceState>, mut session: ResMut<ManagementSession>, time: Res<Time<Real>>,
    setup: Option<Res<crate::world_setup::WorldSetup>>,
    mut elapsed: Local<f32>, mut previous: Local<Option<(uuid::Uuid, SimulationSpeed)>>,
) {
    let key = (session.session_id, session.speed);
    if previous.as_ref() != Some(&key) { *elapsed = 0.; *previous = Some(key); }
    if !(state.screen == Screen::Management || (state.screen == Screen::WorldMap && session.geographic))
        || !session.running || setup.is_some_and(|setup| setup.open)
    {
        if session.running { session.running = false; }
        *elapsed = 0.;
        return;
    }
    *elapsed += time.delta_secs();
    if session.is_busy() { *elapsed = 0.; return; }
    if *elapsed >= session.speed.seconds_per_day() {
        *elapsed = 0.;
        session.step();
    }
}
