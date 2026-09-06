use crate::{
    button::{ButtonLabel, ButtonMotion, ButtonTone, UiButton},
    fonts::UiFonts,
    theme,
};
use bevy::prelude::*;

pub fn node(commands: &mut Commands, parent: Entity, layout: Node) -> Entity {
    let entity = commands.spawn(layout).id();
    commands.entity(parent).add_child(entity);
    entity
}

pub fn text(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    value: impl Into<String>,
    size: f32,
    color: Color,
    strong: bool,
) -> Entity {
    let entity = commands
        .spawn((
            Text::new(value),
            TextFont {
                font: if strong {
                    fonts.semibold.clone()
                } else {
                    fonts.regular.clone()
                }
                .into(),
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            Node {
                flex_shrink: 0.,
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

pub fn panel(commands: &mut Commands, parent: Entity, layout: Node) -> Entity {
    let entity = node(
        commands,
        parent,
        Node {
            border: UiRect::all(px(1)),
            ..layout
        },
    );
    commands.entity(entity).insert((
        crate::skin::PanelSkin::Panel,
        theme::panel_gradient(),
        BorderColor::all(theme::BORDER),
        theme::shadow(),
    ));
    entity
}

pub fn rule(commands: &mut Commands, parent: Entity) {
    let entity = node(
        commands,
        parent,
        Node {
            width: percent(100),
            height: px(1),
            flex_shrink: 0.,
            margin: UiRect::vertical(px(8)),
            ..default()
        },
    );
    commands
        .entity(entity)
        .insert(BackgroundGradient::from(LinearGradient {
            angle: std::f32::consts::FRAC_PI_2,
            stops: vec![
                theme::ACCENT.with_alpha(0.1).into(),
                theme::ACCENT.with_alpha(0.65).into(),
                theme::ACCENT.with_alpha(0.1).into(),
            ],
            ..default()
        }));
}

pub fn button(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    label: &str,
    properties: UiButton,
) -> Entity {
    let entity = node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(46),
            flex_shrink: 1.,
            padding: UiRect::axes(px(18), px(11)),
            border: UiRect::all(px(1)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(3)),
            ..default()
        },
    );
    commands.entity(entity).insert((
        Button,
        properties,
        ButtonMotion::default(),
        BackgroundColor(theme::PANEL),
        BorderColor::all(theme::BORDER),
    ));
    let color = if !properties.enabled {
        theme::DISABLED
    } else if properties.tone == ButtonTone::Primary {
        theme::ACCENT_BRIGHT
    } else {
        theme::TEXT
    };
    let label = text(
        commands,
        entity,
        fonts,
        label,
        17.,
        color,
        properties.tone == ButtonTone::Primary,
    );
    commands
        .entity(label)
        .insert((ButtonLabel, UiTransform::default(), ZIndex(1)));
    entity
}

/// A small original geometric seal, built from native UI nodes rather than a font glyph.
pub fn seal(commands: &mut Commands, parent: Entity, size: f32) {
    let outer = node(
        commands,
        parent,
        Node {
            width: px(size),
            height: px(size),
            flex_shrink: 0.,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::MAX,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    );
    commands
        .entity(outer)
        .insert((BorderColor::all(theme::ACCENT), BackgroundColor(theme::INK)));
    let inner = node(
        commands,
        outer,
        Node {
            width: percent(78),
            height: percent(78),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::MAX,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    );
    commands
        .entity(inner)
        .insert(BorderColor::all(theme::BORDER));
    for angle in [0., std::f32::consts::FRAC_PI_2] {
        let diamond = node(
            commands,
            inner,
            Node {
                position_type: PositionType::Absolute,
                width: px(size * 0.3),
                height: px(size * 0.3),
                border: UiRect::all(px(1.5)),
                ..default()
            },
        );
        commands.entity(diamond).insert((
            BorderColor::all(theme::ACCENT_BRIGHT),
            UiTransform::from_rotation(Rot2::radians(angle + 0.785)),
        ));
    }
}
