use crate::{
    motion::{MotionPreferences, MotionValue},
    theme,
};
use bevy::{
    input_focus::{FocusCause, InputFocus, InputFocusVisible},
    prelude::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTone {
    Primary,
    Secondary,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct UiButton {
    pub enabled: bool,
    pub selected: bool,
    pub tone: ButtonTone,
    pub tab_order: u32,
}

impl UiButton {
    pub fn primary(tab_order: u32) -> Self {
        Self {
            enabled: true,
            selected: false,
            tone: ButtonTone::Primary,
            tab_order,
        }
    }
    pub fn secondary(tab_order: u32) -> Self {
        Self {
            tone: ButtonTone::Secondary,
            ..Self::primary(tab_order)
        }
    }
}

#[derive(Message)]
pub struct ButtonActivated(pub Entity);

#[derive(Resource, Default)]
pub struct KeyboardFocus(pub Option<Entity>);

#[derive(Component)]
pub struct ButtonLabel;

#[derive(Component)]
pub struct ButtonMotion {
    hover: MotionValue,
    press: MotionValue,
    flash: f32,
}

impl Default for ButtonMotion {
    fn default() -> Self {
        Self {
            hover: MotionValue::new(0.),
            press: MotionValue::new(0.),
            flash: 0.,
        }
    }
}

pub fn keyboard_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Query<(Entity, &UiButton)>,
    mut focus: ResMut<KeyboardFocus>,
    mut input_focus: ResMut<InputFocus>,
    mut focus_visible: ResMut<InputFocusVisible>,
    mut activated: MessageWriter<ButtonActivated>,
) {
    if focus.0.is_some_and(|entity| buttons.get(entity).is_err()) {
        focus.0 = None;
        input_focus.clear();
    }
    if keys.just_pressed(KeyCode::Tab) {
        let mut available: Vec<_> = buttons
            .iter()
            .filter(|(_, button)| button.enabled)
            .collect();
        available.sort_by_key(|(entity, button)| (button.tab_order, entity.to_bits()));
        if !available.is_empty() {
            let reverse = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            let current = available
                .iter()
                .position(|(entity, _)| Some(*entity) == focus.0);
            let index = next_index(current, available.len(), reverse);
            focus.0 = Some(available[index].0);
            input_focus.set(available[index].0, FocusCause::Navigated);
            focus_visible.0 = true;
        }
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space])
        && let Some(entity) = focus.0
        && let Ok((_, button)) = buttons.get(entity)
        && button.enabled
    {
        activated.write(ButtonActivated(entity));
    }
}

fn next_index(current: Option<usize>, length: usize, reverse: bool) -> usize {
    match (current, reverse) {
        (Some(index), true) => (index + length - 1) % length,
        (Some(index), false) => (index + 1) % length,
        (None, true) => length - 1,
        (None, false) => 0,
    }
}

pub fn pointer_interaction(
    mouse: Res<ButtonInput<MouseButton>>,
    buttons: Query<(Entity, &Interaction, &UiButton), Changed<Interaction>>,
    mut activated: MessageWriter<ButtonActivated>,
    mut focus: ResMut<KeyboardFocus>,
    mut input_focus: ResMut<InputFocus>,
    mut focus_visible: ResMut<InputFocusVisible>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        focus_visible.0 = false;
    }
    for (entity, interaction, button) in &buttons {
        if *interaction == Interaction::Pressed && button.enabled {
            focus.0 = Some(entity);
            input_focus.set(entity, FocusCause::Pressed);
            focus_visible.0 = false;
            activated.write(ButtonActivated(entity));
        }
    }
}

pub fn animate_buttons(
    focus: Res<KeyboardFocus>,
    time: Res<Time<Real>>,
    preferences: Res<MotionPreferences>,
    mut activated: MessageReader<ButtonActivated>,
    mut buttons: Query<(
        Entity,
        &Interaction,
        &UiButton,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ButtonMotion,
        &Children,
    )>,
    mut labels: Query<(&mut UiTransform, &mut TextColor), With<ButtonLabel>>,
) {
    for event in activated.read() {
        if let Ok((_, _, _, _, _, mut motion, _)) = buttons.get_mut(event.0) {
            motion.flash = 1.;
        }
    }
    for (entity, interaction, button, mut background, mut border, mut motion, children) in
        &mut buttons
    {
        let hovering =
            button.enabled && (focus.0 == Some(entity) || *interaction == Interaction::Hovered);
        let pressing = button.enabled && *interaction == Interaction::Pressed;
        motion.hover.retarget(
            if hovering { 1. } else { 0. },
            0.28,
            EaseFunction::QuinticOut,
        );
        motion.press.retarget(
            if pressing { 1. } else { 0. },
            if pressing { 0.09 } else { 0.48 },
            if pressing {
                EaseFunction::QuinticOut
            } else {
                EaseFunction::ElasticOut
            },
        );
        let hover = motion.hover.advance(time.delta_secs(), preferences.reduced);
        let press = motion.press.advance(time.delta_secs(), preferences.reduced);
        motion.flash = if preferences.reduced {
            0.
        } else {
            (motion.flash - time.delta_secs() / 0.45).max(0.)
        };
        let (base_fill, base_outline) = if !button.enabled {
            (theme::INK, theme::BORDER.with_alpha(0.5))
        } else if button.selected || button.tone == ButtonTone::Primary {
            (Color::srgb(0.14, 0.21, 0.16), theme::GOLD)
        } else {
            (theme::PANEL, theme::BORDER)
        };
        let fill = base_fill
            .mix(&theme::PANEL_LIGHT, hover)
            .mix(&theme::GOLD, motion.flash.powi(3) * 0.22);
        let outline = base_outline.mix(&theme::GOLD_BRIGHT, hover);
        background.set_if_neq(BackgroundColor(fill));
        border.set_if_neq(BorderColor::all(outline));
        // Only the label moves. The button's hitbox and surrounding layout stay stable.
        for child in children {
            if let Ok((mut transform, mut color)) = labels.get_mut(*child) {
                color.set_if_neq(TextColor(if !button.enabled {
                    theme::DISABLED
                } else if button.tone == ButtonTone::Primary {
                    theme::GOLD_BRIGHT
                } else {
                    theme::TEXT
                }));
                transform.scale = Vec2::splat(if preferences.reduced {
                    1.
                } else {
                    1. + hover * 0.015 - press * 0.04
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_wraps_in_both_directions() {
        assert_eq!(next_index(None, 3, false), 0);
        assert_eq!(next_index(None, 3, true), 2);
        assert_eq!(next_index(Some(2), 3, false), 0);
        assert_eq!(next_index(Some(0), 3, true), 2);
    }

    #[test]
    fn pointer_does_not_activate_disabled_buttons() {
        let mut app = App::new();
        app.init_resource::<KeyboardFocus>()
            .init_resource::<InputFocus>()
            .init_resource::<InputFocusVisible>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_message::<ButtonActivated>()
            .add_systems(Update, pointer_interaction);
        app.world_mut().spawn((
            Interaction::Pressed,
            UiButton {
                enabled: false,
                ..UiButton::primary(0)
            },
        ));
        app.update();
        assert!(
            app.world()
                .resource::<Messages<ButtonActivated>>()
                .is_empty()
        );
    }
}
