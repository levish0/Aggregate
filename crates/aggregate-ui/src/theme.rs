//! Shared palette and framing. Screen composition belongs to the client.
use bevy::prelude::*;

pub const INK: Color = Color::srgb(0.035, 0.065, 0.070);
pub const PANEL: Color = Color::srgb(0.070, 0.115, 0.116);
pub const PANEL_LIGHT: Color = Color::srgb(0.105, 0.166, 0.164);
pub const GOLD: Color = Color::srgb(0.73, 0.59, 0.36);
pub const GOLD_BRIGHT: Color = Color::srgb(0.94, 0.81, 0.53);
pub const BORDER: Color = Color::srgb(0.32, 0.31, 0.23);
pub const TEXT: Color = Color::srgb(0.88, 0.87, 0.80);
pub const MUTED: Color = Color::srgb(0.59, 0.65, 0.61);
pub const DISABLED: Color = Color::srgb(0.34, 0.40, 0.38);
pub const BURGUNDY: Color = Color::srgb(0.24, 0.10, 0.13);

pub fn panel_gradient() -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient {
        angle: 0.25,
        stops: vec![PANEL_LIGHT.into(), PANEL.into(), INK.into()],
        ..default()
    })
}

pub fn shadow() -> BoxShadow {
    BoxShadow(vec![ShadowStyle {
        color: Color::BLACK.with_alpha(0.55),
        x_offset: px(0),
        y_offset: px(12),
        spread_radius: px(2),
        blur_radius: px(22),
    }])
}
