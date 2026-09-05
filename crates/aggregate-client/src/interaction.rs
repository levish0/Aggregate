use crate::state::{InterfaceAction, InterfaceState, Screen};
use aggregate_ui::{
    button::{ButtonActivated, KeyboardFocus},
    tooltip::TooltipState,
};
use bevy::prelude::*;

pub fn apply_actions(
    mut activated: MessageReader<ButtonActivated>,
    actions: Query<&InterfaceAction>,
    mut state: ResMut<InterfaceState>,
    keys: Res<ButtonInput<KeyCode>>,
    tooltip: Res<TooltipState>,
    mut focus: ResMut<KeyboardFocus>,
    mut exit: MessageWriter<AppExit>,
) {
    let requested: Vec<_> = activated
        .read()
        .filter_map(|event| actions.get(event.0).ok().copied())
        .collect();
    let escape = keys.just_pressed(KeyCode::Escape) && !tooltip.dismissed_this_frame;
    if requested.is_empty() && !escape {
        return;
    }
    for action in requested
        .into_iter()
        .chain(escape.then_some(InterfaceAction::Back))
    {
        match action {
            InterfaceAction::OpenManagement => state.screen = Screen::Management,
            InterfaceAction::OpenPreview => state.screen = Screen::Preview,
            InterfaceAction::OpenSettings => state.screen = Screen::Settings,
            InterfaceAction::Back => state.screen = Screen::MainMenu,
            InterfaceAction::Exit => {
                exit.write(AppExit::Success);
            }
            InterfaceAction::SwitchLanguage => {
                let language = state.localization.language().other();
                state.localization.set_language(language);
            }
            InterfaceAction::SelectTab(tab) => state.tab = tab,
            InterfaceAction::ScaleDown => state.scale = (state.scale - 0.1).max(0.8),
            InterfaceAction::ScaleUp => state.scale = (state.scale + 0.1).min(1.4),
            InterfaceAction::ResetScale => state.scale = 1.,
            InterfaceAction::ToggleMotion => state.reduced_motion = !state.reduced_motion,
            InterfaceAction::PrimaryExample => {
                state.primary_selected = !state.primary_selected;
                state.status_key = "status-selected";
            }
            InterfaceAction::SecondaryExample => state.status_key = "status-secondary",
        }
        if matches!(
            action,
            InterfaceAction::OpenPreview
                | InterfaceAction::OpenManagement
                | InterfaceAction::OpenSettings
                | InterfaceAction::Back
                | InterfaceAction::SwitchLanguage
                | InterfaceAction::SelectTab(_)
        ) {
            focus.0 = None;
        }
    }
}

pub fn update_live_labels(
    state: Res<InterfaceState>,
    mut labels: Query<(
        &mut Text,
        Option<&crate::screens::StatusLabel>,
        Option<&crate::screens::ScaleLabel>,
    )>,
    mut buttons: Query<(
        &InterfaceAction,
        &Children,
        &mut aggregate_ui::button::UiButton,
    )>,
    mut motion: ResMut<aggregate_ui::motion::MotionPreferences>,
) {
    if !state.is_changed() {
        return;
    }
    motion.reduced = state.reduced_motion;
    for (mut text, status, scale) in &mut labels {
        if status.is_some() {
            **text = state.text(state.status_key);
        }
        if scale.is_some() {
            **text = format!(
                "{}  ·  {:.0}%",
                state.text("settings-scale"),
                state.scale * 100.
            );
        }
    }
    for (action, children, mut button) in &mut buttons {
        if *action == InterfaceAction::ToggleMotion
            && let Ok((mut text, _, _)) = labels.get_mut(children[0])
        {
            **text = state.text(if state.reduced_motion {
                "settings-motion-reduced"
            } else {
                "settings-motion-full"
            });
        }
        if *action == InterfaceAction::PrimaryExample {
            button.selected = state.primary_selected;
            if let Ok((mut text, _, _)) = labels.get_mut(children[0]) {
                **text = state.text(if state.primary_selected {
                    "control-enabled"
                } else {
                    "control-primary"
                });
            }
        }
    }
}

pub fn responsive_scale(
    window: Single<&Window>,
    state: Res<InterfaceState>,
    mut ui_scale: ResMut<UiScale>,
) {
    // Keep the 1280x800 reference layout accessible on smaller windows and high DPI displays.
    let fit = (window.width() / 1280.).min(window.height() / 800.).min(1.);
    let next = fit * state.scale;
    if ui_scale.0 != next {
        ui_scale.0 = next;
    }
}
