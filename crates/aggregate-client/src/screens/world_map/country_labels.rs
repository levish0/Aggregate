pub mod material;
mod placement;
mod typography;
use crate::state::{InterfaceState, Screen};
use aggregate_map_view::{LoadedWorldMap, MapCameraController};
use aggregate_ui::fonts::UiFonts;
use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

#[derive(Component)]
pub struct CountryLabelLayer;
#[derive(Component)]
pub struct CountryLabel {
    anchor_x: f32,
}
#[derive(Default)]
pub struct LabelPreparation {
    source: Option<(usize, aggregate_localization::Language)>,
    root: Option<Entity>,
    task: Option<Task<Vec<placement::PreparedLabel>>>,
}

/// Map lettering has a fixed world orientation and scale, independent of the camera.
pub fn update(
    mut commands: Commands,
    interface: Res<InterfaceState>,
    fonts: Res<UiFonts>,
    font_assets: Res<Assets<Font>>,
    map: Option<Res<LoadedWorldMap>>,
    controller: Res<MapCameraController>,
    mut roots: Query<&mut Visibility, With<CountryLabelLayer>>,
    mut labels: Query<(&CountryLabel, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<material::CountryLetteringMaterial>>,
    mut preparation: Local<LabelPreparation>,
) {
    let visible = interface.screen == Screen::WorldMap;
    for mut visibility in &mut roots {
        let value = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != value {
            *visibility = value;
        }
    }
    let source = map.as_ref().map(|map| {
        (
            std::sync::Arc::as_ptr(&map.0) as usize,
            interface.localization.language(),
        )
    });
    if source != preparation.source {
        if let Some(root) = preparation.root.take() {
            commands.entity(root).despawn();
        }
        preparation.task = None;
        preparation.source = source;
        if source.is_some()
            && let (Some(map), Some(font)) = (&map, font_assets.get(&fonts.semibold))
        {
            let names = map
                .0
                .catalog
                .countries
                .iter()
                .map(|country| {
                    let name = interface
                        .localization
                        .text(&format!("country-{}", country.key.to_lowercase()))
                        .unwrap_or_else(|_| country.key.clone());
                    (country.id.clone(), name)
                })
                .collect();
            let map = map.0.clone();
            let bytes = font.data.as_ref().to_vec();
            preparation.task = Some(AsyncComputeTaskPool::get().spawn(async move {
                let started = std::time::Instant::now();
                let labels = placement::prepare(&map, &names, &bytes);
                tracing::info!(
                    count = labels.len(),
                    elapsed_ms = started.elapsed().as_millis(),
                    "Country lettering prepared"
                );
                labels
            }));
        } else if source.is_some() {
            preparation.source = None;
        }
    }
    if let Some(task) = &mut preparation.task
        && let Some(prepared) = block_on(future::poll_once(task))
    {
        preparation.task = None;
        let root = commands
            .spawn((
                Name::new("Country map lettering"),
                CountryLabelLayer,
                Transform::default(),
                if visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ))
            .id();
        for label in prepared {
            let material = materials.add(material::CountryLetteringMaterial {
                lettering: images.add(label.image),
            });
            commands.spawn((
                Name::new(label.name),
                CountryLabel {
                    anchor_x: label.anchor_x,
                },
                Mesh3d(meshes.add(label.mesh)),
                MeshMaterial3d(material),
                Transform::default(),
                RenderLayers::layer(1),
                Pickable::IGNORE,
                bevy::light::NotShadowCaster,
                ChildOf(root),
            ));
        }
        preparation.root = Some(root);
    }
    if let Some(map) = map {
        let width = map.0.terrain.size.x;
        for (label, mut transform) in &mut labels {
            let offset = ((controller.target.x - label.anchor_x) / width).round() * width;
            if transform.translation.x != offset {
                transform.translation.x = offset;
            }
        }
    }
}
