use crate::state::{InterfaceState, Screen};
use aggregate_map_view::{LoadedWorldMap, MapCamera, MapCameraController};
use aggregate_ui::{components as ui, fonts::UiFonts};
use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Component)]
pub struct CountryLabelLayer;
#[derive(Component)]
pub struct CountryLabel { anchor: Vec2, width: f32, priority: usize, characters: usize }

pub fn build(commands: &mut Commands, root: Entity) {
    let layer = ui::node(commands, root, Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default() });
    commands.entity(layer).insert((CountryLabelLayer, GlobalZIndex(-1), Pickable::IGNORE, bevy::ui::FocusPolicy::Pass));
}

pub fn update(
    mut commands: Commands, interface: Res<InterfaceState>, fonts: Res<UiFonts>,
    map: Option<Res<LoadedWorldMap>>, controller: Res<MapCameraController>,
    camera: Single<(&Camera, &Transform), With<MapCamera>>, window: Single<&Window>, scale: Res<UiScale>,
    layers: Query<Entity, With<CountryLabelLayer>>,
    mut labels: Query<(&CountryLabel, &mut Node, &mut TextFont, &mut TextColor)>,
    mut previous: Local<Option<Entity>>,
) {
    if interface.screen != Screen::WorldMap { return; }
    let (Some(map), Ok(layer)) = (map, layers.single()) else { return; };
    let size = map.0.terrain.size;
    if *previous != Some(layer) {
        *previous = Some(layer);
        let mut countries = BTreeMap::<_, Vec<Vec2>>::new();
        for (index, province) in map.0.catalog.provinces.iter().enumerate() {
            if !province.water && let Some(country) = &province.owner { countries.entry(country).or_default().push(map.0.provinces.centroids[index + 1]); }
        }
        for country in &map.0.catalog.countries {
            let Some(points) = countries.get(&country.id) else { continue; };
            let angle = points.iter().map(|point| Vec2::from_angle(point.x * std::f32::consts::TAU)).sum::<Vec2>().to_angle();
            let center = Vec2::new(angle.rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU, points.iter().map(|point| point.y).sum::<f32>() / points.len() as f32);
            let offset = |point: Vec2| Vec2::new((point.x - center.x + 0.5).rem_euclid(1.) - 0.5, point.y - center.y);
            // Keep the anchor on owned land instead of placing island countries over the sea.
            let anchor = points.iter().min_by(|left, right| offset(**left).length_squared().total_cmp(&offset(**right).length_squared())).copied().unwrap();
            let spread = (points.iter().map(|point| offset(*point).x.powi(2)).sum::<f32>() / points.len() as f32).sqrt();
            let name = interface.localization.text(&format!("country-{}", country.key.to_lowercase())).unwrap_or_else(|_| country.key.clone());
            let entity = ui::text(&mut commands, layer, &fonts, &name, 18., Color::srgba(0.035,0.04,0.045,0.85), true);
            commands.entity(entity).insert((CountryLabel { anchor: anchor * size, width: (spread * size.x * 2.).max(8.), priority: points.len(), characters: name.chars().count() }, Pickable::IGNORE, bevy::ui::FocusPolicy::Pass));
        }
    }
    let transform = GlobalTransform::from(*camera.1);
    let mut occupied = Vec::<Rect>::new();
    let mut ordered: Vec<_> = labels.iter_mut().collect();
    ordered.sort_by_key(|(label, _, _, _)| std::cmp::Reverse(label.priority));
    for (label, mut node, mut font, mut color) in ordered {
        let x = label.anchor.x + ((controller.target.x - label.anchor.x) / size.x).round() * size.x;
        let point = Vec3::new(x, 4., label.anchor.y);
        let (Ok(center), Ok(edge)) = (camera.0.world_to_viewport(&transform, point), camera.0.world_to_viewport(&transform, point + Vec3::X * label.width)) else { node.display = Display::None; continue; };
        let projected_width = center.distance(edge).min(320.);
        let font_size = (projected_width / (label.characters.max(1) as f32 * 0.85)).min(36.);
        let rect = Rect::from_center_size(center, Vec2::new(font_size * label.characters as f32 * 0.9 + 10., font_size + 8.));
        if font_size < 11. || center.x < 0. || center.x > window.width() || center.y < 76. || center.y > window.height() - 45. || occupied.iter().any(|other| other.intersect(rect).size().min_element() > 0.) { node.display = Display::None; continue; }
        occupied.push(rect);
        let ui_scale = scale.0;
        node.display = Display::Flex;
        node.position_type = PositionType::Absolute;
        node.left = px(rect.min.x / ui_scale);
        node.top = px(rect.min.y / ui_scale);
        node.width = px(rect.width() / ui_scale);
        node.justify_content = JustifyContent::Center;
        font.font_size = FontSize::Px(font_size / ui_scale);
        color.0 = Color::srgba(0.025, 0.03, 0.035, 0.65 + ((controller.distance - 130.) / 400.).clamp(0.,1.) * 0.25);
    }
}
