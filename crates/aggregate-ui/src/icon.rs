//! Heroicons 24 solid, retained as SVG sources and rendered at 4x for native UI.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Government,
    Buildings,
    Population,
    Globe,
    Map,
    Settings,
    Close,
    Programs,
    Chart,
    Education,
    Health,
    Military,
    Goods,
    Queue,
    Play,
    Pause,
}

impl Icon {
    fn path(self) -> &'static str {
        match self {
            Self::Government => "icons/heroicons/solid/building-library.png",
            Self::Buildings => "icons/heroicons/solid/building-office-2.png",
            Self::Population => "icons/heroicons/solid/users.png",
            Self::Globe => "icons/heroicons/solid/globe-americas.png",
            Self::Map => "icons/heroicons/solid/map.png",
            Self::Settings => "icons/heroicons/solid/cog-6-tooth.png",
            Self::Close => "icons/heroicons/solid/x-mark.png",
            Self::Programs => "icons/heroicons/solid/squares-2x2.png",
            Self::Chart => "icons/heroicons/solid/chart-bar.png",
            Self::Education => "icons/heroicons/solid/academic-cap.png",
            Self::Health => "icons/heroicons/solid/heart.png",
            Self::Military => "icons/heroicons/solid/shield-check.png",
            Self::Goods => "icons/heroicons/solid/cube.png",
            Self::Queue => "icons/heroicons/solid/queue-list.png",
            Self::Play => "icons/heroicons/solid/play.png",
            Self::Pause => "icons/heroicons/solid/pause.png",
        }
    }
}

pub fn icon(
    commands: &mut Commands,
    parent: Entity,
    icon: Icon,
    size: f32,
    color: Color,
) -> Entity {
    let entity = commands
        .spawn((
            icon,
            ImageNode { color, ..default() },
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

pub fn load(server: Res<AssetServer>, mut icons: Query<(&Icon, &mut ImageNode), Changed<Icon>>) {
    for (icon, mut image) in &mut icons {
        image.image = server.load(icon.path());
    }
}
