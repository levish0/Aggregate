use super::{ManagementSession, ManagementViewState};
use crate::state::{InterfaceState, Screen};
use aggregate_ui::button::{ButtonActivated, UiButton};
use aggregate_world::{FacilityDefinitionId, ProvinceId};
use bevy::prelude::*;

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum ManagementAction {
    SelectProvince(ProvinceId),
    StartConstruction(FacilityDefinitionId),
    StartConstructionAt { province: ProvinceId, definition: FacilityDefinitionId },
    StepDay,
    ToggleRunning,
    SetSpeed(super::SimulationSpeed),
}

pub fn apply_management_actions(
    mut activated: MessageReader<ButtonActivated>,
    actions: Query<(&ManagementAction, &UiButton)>,
    state: Res<InterfaceState>,
    mut session: ResMut<ManagementSession>,
    mut view: ResMut<ManagementViewState>,
) {
    for event in activated.read() {
        if state.screen != Screen::Management && !(state.screen == Screen::WorldMap && session.geographic) {
            continue;
        }
        let Ok((action, button)) = actions.get(event.0) else {
            continue;
        };
        if !button.enabled {
            continue;
        }
        info!(?action, province = %view.selected_province, day = session.snapshot.day, "Management action requested");
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
            ManagementAction::StartConstructionAt { province, definition } => session.start_construction(province, definition),
            ManagementAction::StepDay => {
                session.running = false;
                session.step();
            }
            ManagementAction::ToggleRunning => session.running = !session.running,
            ManagementAction::SetSpeed(speed) => session.speed = *speed,
        }
    }
}
