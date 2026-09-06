use crate::state::{InterfaceState, Screen};
use aggregate_map_view::{LoadedWorldMap, MapCameraController, MapViewState};
use aggregate_ui::{
    button::{ButtonActivated, UiButton},
    components as ui,
    fonts::UiFonts,
    theme,
};
use aggregate_world::{CountryId, RegionId};
use bevy::prelude::*;
use std::collections::BTreeSet;

#[derive(Component)]
pub struct MapOutliner;
#[derive(Component)]
pub struct MapOutlinerContent;
#[derive(Component)]
pub struct JumpToRegion {
    region: RegionId,
    country: CountryId,
}

pub fn rebuild(
    mut commands: Commands,
    interface: Res<InterfaceState>,
    state: Res<MapViewState>,
    map: Option<Res<LoadedWorldMap>>,
    fonts: Res<UiFonts>,
    roots: Query<Entity, With<MapOutliner>>,
    old: Query<Entity, With<MapOutlinerContent>>,
    mut previous: Local<Option<(Entity, Option<CountryId>, aggregate_localization::Language)>>,
) {
    if interface.screen != Screen::WorldMap {
        return;
    }
    let Ok(root) = roots.single() else {
        return;
    };
    let country = map
        .as_ref()
        .and_then(|map| {
            state
                .selected_index
                .checked_sub(1)
                .and_then(|i| map.0.catalog.provinces.get(i as usize))
        })
        .and_then(|province| province.owner.clone());
    let key = (root, country.clone(), interface.localization.language());
    if previous.as_ref() == Some(&key) {
        return;
    }
    *previous = Some(key);
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let column = ui::node(
        &mut commands,
        root,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            width: percent(100),
            ..default()
        },
    );
    commands.entity(column).insert(MapOutlinerContent);
    let (Some(country), Some(map)) = (country, map) else {
        ui::text(
            &mut commands,
            column,
            &fonts,
            interface.text("map-select-hint"),
            14.,
            theme::TEXT,
            false,
        );
        return;
    };
    let ids: BTreeSet<_> = map
        .0
        .catalog
        .provinces
        .iter()
        .filter(|province| province.owner.as_ref() == Some(&country))
        .filter_map(|province| province.region.as_ref())
        .collect();
    let mut regions: Vec<_> = map
        .0
        .catalog
        .regions
        .iter()
        .filter(|region| ids.contains(&region.id))
        .collect();
    regions.sort_by(|a, b| a.name.cmp(&b.name));
    for (index, region) in regions.into_iter().enumerate() {
        let button = ui::button(
            &mut commands,
            column,
            &fonts,
            &region.name,
            UiButton::secondary(100 + index as u32),
        );
        commands.entity(button).insert(Node {
            width: percent(100),
            min_height: px(28),
            padding: UiRect::axes(px(8), px(4)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            flex_shrink: 0.,
            ..default()
        });
        commands.entity(button).insert(JumpToRegion {
            region: region.id.clone(),
            country: country.clone(),
        });
    }
}

pub fn apply_jumps(
    mut events: MessageReader<ButtonActivated>,
    actions: Query<&JumpToRegion>,
    map: Option<Res<LoadedWorldMap>>,
    mut state: ResMut<MapViewState>,
    mut camera: ResMut<MapCameraController>,
) {
    let Some(map) = map else {
        return;
    };
    for event in events.read() {
        if let Ok(action) = actions.get(event.0)
            && let Some((index, _)) =
                map.0
                    .catalog
                    .provinces
                    .iter()
                    .enumerate()
                    .find(|(_, province)| {
                        province.region.as_ref() == Some(&action.region)
                            && province.owner.as_ref() == Some(&action.country)
                    })
        {
            let index = index + 1;
            state.selected_index = index as u32;
            state.inspect_country = false;
            let uv = map.0.provinces.centroids[index];
            camera.target = Vec3::new(uv.x * map.0.terrain.size.x, 0., uv.y * map.0.terrain.size.y);
        }
    }
}
