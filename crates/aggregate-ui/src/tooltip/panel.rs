use super::{TooltipContent, state::TooltipPhase};
use crate::{button::UiButton, components, fonts::UiFonts, motion::PanelEntrance, theme};
use bevy::prelude::*;

const RING_SEGMENTS: usize = 24;

#[derive(Component)]
pub(crate) struct TooltipPanel {
    pub source: Entity,
}

#[derive(Component)]
pub(crate) struct TooltipParent(pub Entity);

#[derive(Component)]
pub(crate) struct LockIndicator(pub Entity);

#[derive(Component)]
pub(crate) struct LockSegment {
    pub source: Entity,
    pub index: usize,
}

pub(super) fn segment_color(index: usize, progress: f32) -> Color {
    if (index as f32) < progress * RING_SEGMENTS as f32 {
        theme::ACCENT_BRIGHT
    } else {
        theme::BORDER
    }
}

pub(super) fn spawn_panel(
    commands: &mut Commands,
    fonts: &UiFonts,
    source: Entity,
    content: &TooltipContent,
    depth: usize,
    anchor: Rect,
    viewport: Vec2,
    phase: &TooltipPhase,
) {
    let width = 340_f32.min((viewport.x - 24.).max(1.));
    let estimated_height = 240. + content.links.len() as f32 * 58.;
    let right = anchor.max.x + 8.;
    let left = if right + width <= viewport.x - 12. {
        right
    } else {
        anchor.min.x - width - 8.
    };
    let left = left.clamp(12., (viewport.x - width - 12.).max(12.));
    let top = anchor
        .min
        .y
        .clamp(12., (viewport.y - estimated_height - 12.).max(12.));
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(width),
                max_height: px((viewport.y - top - 12.).max(1.)),
                overflow: Overflow::scroll_y(),
                padding: UiRect::all(px(20)),
                border: UiRect::all(px(1)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(theme::INK),
            crate::skin::PanelSkin::Tooltip,
            BorderColor::all(theme::ACCENT),
            theme::shadow(),
            GlobalZIndex(100 + depth as i32),
            bevy::ui::FocusPolicy::Block,
            crate::scroll::ScrollRegion::default(),
            crate::layout::UiPointerBlocker,
            TooltipPanel { source },
            PanelEntrance::new(Vec2::new(7., 0.)),
            UiTransform::default(),
        ))
        .id();
    components::text(
        commands,
        root,
        fonts,
        &content.title,
        20.,
        theme::ACCENT_BRIGHT,
        true,
    );
    components::text(
        commands,
        root,
        fonts,
        &content.body,
        16.,
        theme::TEXT,
        false,
    );
    for (index, link) in content.links.iter().enumerate() {
        let button = components::button(
            commands,
            root,
            fonts,
            &link.label,
            UiButton {
                enabled: *phase == TooltipPhase::Locked,
                ..UiButton::secondary(100 + depth as u32 * 100 + index as u32)
            },
        );
        commands
            .entity(button)
            .insert(((*link.content).clone(), TooltipParent(source)));
    }
    components::rule(commands, root);
    let status = components::node(
        commands,
        root,
        Node {
            align_items: AlignItems::Center,
            column_gap: px(10),
            ..default()
        },
    );
    let ring = components::node(
        commands,
        status,
        Node {
            width: px(30),
            height: px(30),
            flex_shrink: 0.,
            ..default()
        },
    );
    for index in 0..RING_SEGMENTS {
        let angle = index as f32 / RING_SEGMENTS as f32 * std::f32::consts::TAU
            - std::f32::consts::FRAC_PI_2;
        let segment = components::node(
            commands,
            ring,
            Node {
                position_type: PositionType::Absolute,
                left: px(13.5 + angle.cos() * 12.),
                top: px(13.5 + angle.sin() * 12.),
                width: px(3),
                height: px(3),
                border_radius: BorderRadius::MAX,
                ..default()
            },
        );
        commands.entity(segment).insert((
            BackgroundColor(segment_color(
                index,
                if *phase == TooltipPhase::Locked {
                    1.
                } else {
                    0.
                },
            )),
            LockSegment { source, index },
        ));
    }
    let label = components::text(
        commands,
        status,
        fonts,
        if *phase == TooltipPhase::Locked {
            &content.locked_label
        } else {
            &content.locking_label
        },
        13.,
        theme::ACCENT,
        false,
    );
    commands.entity(label).insert(LockIndicator(source));
    components::text(
        commands,
        root,
        fonts,
        &content.hint,
        13.,
        theme::MUTED,
        false,
    );
}
