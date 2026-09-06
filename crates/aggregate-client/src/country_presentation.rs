//! Flag presentation resolves geography country identities without changing the player country.
use aggregate_map_view::LoadedWorldMap;
use aggregate_world::CountryId;
use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Resource)]
pub struct CountryFlags(BTreeMap<String, String>);

impl Default for CountryFlags {
    fn default() -> Self {
        Self(
            ron::from_str(include_str!("../../../assets/flags/countries.ron"))
                .expect("bundled country flag mapping"),
        )
    }
}

#[derive(Component)]
pub struct CountryFlag(pub CountryId);

pub fn flag(commands: &mut Commands, parent: Entity, country: CountryId, height: f32) -> Entity {
    let entity = commands
        .spawn((
            CountryFlag(country),
            ImageNode::default(),
            Node {
                width: px(height * 1.5),
                height: px(height),
                flex_shrink: 0.,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

pub fn load_flags(
    server: Res<AssetServer>,
    flags: Res<CountryFlags>,
    map: Option<Res<LoadedWorldMap>>,
    mut images: Query<(Ref<CountryFlag>, &mut ImageNode)>,
) {
    for (flag, mut image) in &mut images {
        if !flag.is_changed() && !map.as_ref().is_some_and(|map| map.is_changed()) {
            continue;
        }
        let code = map
            .as_ref()
            .and_then(|map| {
                map.0
                    .catalog
                    .countries
                    .iter()
                    .find(|country| country.id == flag.0)
            })
            .and_then(|country| flags.0.get(&country.key));
        image.image = match code {
            Some(code) => server.load(format!("flags/raster/{code}.png")),
            None => server.load("icons/heroicons/solid/building-library.png"),
        };
    }
}
