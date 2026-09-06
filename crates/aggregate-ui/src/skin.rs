//! Texture-free translucent surfaces, independent of layout and input.
use crate::theme;
use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub enum PanelSkin { Panel, Tooltip }

pub fn apply_surfaces(mut commands: Commands, panels: Query<(Entity, &PanelSkin), Added<PanelSkin>>) {
    for (entity, skin) in &panels {
        commands.entity(entity).remove::<BackgroundGradient>().insert((
            BackgroundColor(match skin { PanelSkin::Panel => theme::PANEL, PanelSkin::Tooltip => theme::PANEL.with_alpha(0.96) }),
            BorderColor::all(theme::BORDER),
        ));
    }
}
