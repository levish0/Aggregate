//! Monochrome surfaces derived from Lily's dark elevation tokens.
use bevy::prelude::*;
pub const INK: Color = Color::srgb(0.067, 0.067, 0.067);
pub const PANEL: Color = Color::srgba(0.110, 0.110, 0.110, 0.88);
pub const PANEL_LIGHT: Color = Color::srgba(0.19, 0.19, 0.19, 0.94);
pub const ACCENT: Color = Color::srgb(0.80, 0.80, 0.80);
pub const ACCENT_BRIGHT: Color = Color::srgb(0.94, 0.94, 0.94);
pub const BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.22);
pub const TEXT: Color = Color::srgb(0.937, 0.937, 0.937);
pub const MUTED: Color = Color::srgb(0.68, 0.68, 0.68);
pub const DISABLED: Color = Color::srgb(0.40, 0.40, 0.40);
pub const TITLE_BAR: Color = Color::srgba(0.08, 0.08, 0.08, 0.94);

pub fn panel_gradient() -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient {
        angle: 0.25,
        stops: vec![PANEL_LIGHT.into(), PANEL.into()],
        ..default()
    })
}
pub fn shadow() -> BoxShadow {
    BoxShadow(vec![ShadowStyle {
        color: Color::BLACK.with_alpha(0.30),
        x_offset: px(0),
        y_offset: px(8),
        spread_radius: px(0),
        blur_radius: px(16),
    }])
}
