use super::{ManagementSession, ManagementViewState};
use crate::state::{InterfaceState, Screen};
use aggregate_ui::button::{ButtonActivated, UiButton};
use aggregate_world::{FacilityDefinitionId, ProvinceId};
use bevy::prelude::*;

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum ManagementAction {
    SelectProvince(ProvinceId),
    StartConstruction(FacilityDefinitionId),
    StepDay,
    ToggleRunning,
}

pub fn apply_management_actions(
    mut activated: MessageReader<ButtonActivated>,
    actions: Query<(&ManagementAction, &UiButton)>,
    state: Res<InterfaceState>,
    mut session: ResMut<ManagementSession>,
    mut view: ResMut<ManagementViewState>,
) {
    for event in activated.read() {
        if state.screen != Screen::Management {
            continue;
        }
        let Ok((action, button)) = actions.get(event.0) else {
            continue;
        };
        if !button.enabled {
            continue;
        }
        match action {
            ManagementAction::SelectProvince(province) => {
                if session
                    .snapshot
                    .provinces
                    .iter()
                    .any(|item| item.id == *province && item.country == session.player_country)
                {
                    view.selected_province = province.clone();
                }
            }
            ManagementAction::StartConstruction(definition) => {
                session.start_construction(&view.selected_province, definition)
            }
            ManagementAction::StepDay => {
                session.running = false;
                session.step();
            }
            ManagementAction::ToggleRunning => session.running = !session.running,
        }
    }
}

/// One day per real second, with at most one day per rendered frame. Slow frames do not
/// create a catch-up burst. Leaving management pauses the session, preserving its state.
pub fn advance_running_session(
    state: Res<InterfaceState>,
    mut session: ResMut<ManagementSession>,
    time: Res<Time<Real>>,
    mut elapsed: Local<f32>,
) {
    if state.screen != Screen::Management || !session.running {
        if session.running {
            session.running = false;
        }
        *elapsed = 0.;
        return;
    }
    *elapsed += time.delta_secs();
    if *elapsed >= 1. {
        *elapsed = 0.;
        session.step();
    }
}
