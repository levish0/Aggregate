pub mod inspection;
mod layout;
pub mod outliner;
use crate::{
    backdrop::CartographicBackdrop,
    state::{InterfaceState, Screen},
};
use aggregate_map_view::{LoadedWorldMap, MapViewState};
use aggregate_ui::{layout::UiPointerBlocker, select, theme};
use bevy::prelude::*;
pub use layout::build;

#[derive(Component)]
pub struct MapModeSelect;
#[derive(Component)]
pub struct MapInspectionPanel;
#[derive(Component)]
pub struct MapOutlinerPanel;
#[derive(Component)]
pub enum MapLabel {
    Status,
    Hover,
}

pub fn configure_view(
    interface: Res<InterfaceState>,
    mut map: ResMut<MapViewState>,
    window: Single<&Window>,
    blockers: Query<(&ComputedNode, &UiGlobalTransform), With<UiPointerBlocker>>,
    select_state: Res<select::SelectInteractionState>,
    mut changed: MessageReader<select::SelectChanged>,
    mut modes: Query<(Entity, &mut select::Select), With<MapModeSelect>>,
    mut backdrop: Query<&mut Visibility, With<CartographicBackdrop>>,
    mut camera: Single<&mut Camera, With<Camera2d>>,
    policy: Res<aggregate_ui::button::UiKeyboardPolicy>,
    mut panels: Query<&mut Node, With<MapInspectionPanel>>,
) {
    map.enabled = interface.screen == Screen::WorldMap;
    map.reduced_motion = interface.reduced_motion;
    map.overview_allowed = policy.reserve_plain_tab;
    for mut panel in &mut panels {
        panel.display = if map.selected_index == 0 {
            Display::None
        } else {
            Display::Flex
        };
    }
    map.pointer_blocked = select_state.any_open
        || window.physical_cursor_position().is_some_and(|cursor| {
            blockers.iter().any(|(node, transform)| {
                node.size().min_element() > 0.
                    && Rect::from_center_size(transform.translation, node.size()).contains(cursor)
            })
        });
    for event in changed.read() {
        if modes.contains(event.root) {
            map.political = event.value == "political";
        }
    }
    for (_, mut select) in &mut modes {
        let value = if map.political {
            "political"
        } else {
            "terrain"
        };
        if select.value != value {
            select.value = value.into();
        }
    }
    camera.clear_color = if map.enabled {
        ClearColorConfig::None
    } else {
        ClearColorConfig::Custom(theme::INK)
    };
    for mut visibility in &mut backdrop {
        *visibility = if map.enabled {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

pub fn configure_keyboard_policy(
    interface: Res<InterfaceState>,
    select: Res<select::SelectInteractionState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut policy: ResMut<aggregate_ui::button::UiKeyboardPolicy>,
    mut map: ResMut<MapViewState>,
) {
    policy.reserve_plain_tab = interface.screen == Screen::WorldMap
        && !select.any_open
        && !keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if interface.screen != Screen::WorldMap {
        return;
    }
    if keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) && keys.just_pressed(KeyCode::Digit2)
    {
        map.political = true;
    }
}

pub fn update_labels(
    interface: Res<InterfaceState>,
    state: Res<MapViewState>,
    map: Option<Res<LoadedWorldMap>>,
    mut labels: Query<(&MapLabel, &mut Text)>,
) {
    if interface.screen != Screen::WorldMap {
        return;
    }
    for (label, mut text) in &mut labels {
        let region_name = |index: u32| {
            map.as_ref().and_then(|map| {
                index
                    .checked_sub(1)
                    .and_then(|i| map.0.catalog.provinces.get(i as usize))
                    .map(|province| {
                        province
                            .region
                            .as_ref()
                            .and_then(|id| {
                                map.0.catalog.regions.iter().find(|region| &region.id == id)
                            })
                            .map(|region| region.name.clone())
                            .unwrap_or_else(|| {
                                interface.text(if province.water {
                                    "map-water"
                                } else {
                                    "map-unassigned"
                                })
                            })
                    })
            })
        };
        let value = match label {
            MapLabel::Status => {
                if let Some(error) = &state.error {
                    format!("{}\n{error}", interface.text("map-load-failed"))
                } else if let Some(map) = &map {
                    interface.format(
                        "map-counts",
                        &[
                            ("provinces", map.0.catalog.provinces.len().to_string()),
                            ("regions", map.0.catalog.regions.len().to_string()),
                        ],
                    )
                } else {
                    interface.text("map-loading")
                }
            }
            MapLabel::Hover if state.overview_active => interface.text("map-overview-hint"),
            MapLabel::Hover => {
                region_name(state.hovered_index).unwrap_or_else(|| interface.text("map-hover-hint"))
            }
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}
