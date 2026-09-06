use super::MapInspectionPanel;
use crate::{management::ManagementSession, state::{InterfaceState, Screen}};
use aggregate_map_view::{LoadedAdministration, LoadedWorldMap, MapCameraController, MapViewState};
use aggregate_programs::{InspectionScope, MetricValue};
use aggregate_ui::{button::{ButtonActivated, UiButton}, components as ui, fonts::UiFonts, layout::{self, UiPointerBlocker}, theme};
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectionTab { #[default] Overview, Buildings, Population, Territory, Programs }

#[derive(Resource, Default)]
pub struct InspectionView { pub tab: InspectionTab }

#[derive(Component)]
pub struct InspectionRoot;
#[derive(Component)]
pub struct InspectionContent;
#[derive(Component, Clone)]
pub enum InspectionAction { Tab(InspectionTab), Country, State, Province(u32), Close }

pub fn build(commands: &mut Commands, root: Entity) {
    let panel = ui::panel(commands, root, Node {
        position_type: PositionType::Absolute, left: px(92), top: px(94), bottom: px(140), width: px(390),
        flex_direction: FlexDirection::Column, ..default()
    });
    commands.entity(panel).insert((MapInspectionPanel, UiPointerBlocker, InspectionRoot, aggregate_ui::window::FloatingWindow { key: "territory-inspection".into(), minimum_size: Vec2::new(340., 300.) }));
    let resize = ui::node(commands, panel, Node { position_type: PositionType::Absolute, right: px(0), bottom: px(0), width: px(16), height: px(16), ..default() });
    commands.entity(resize).insert((aggregate_ui::window::WindowResizeHandle(panel), BackgroundColor(theme::ACCENT.with_alpha(0.35)), ZIndex(2)));
}

pub fn apply_actions(
    mut activated: MessageReader<ButtonActivated>, actions: Query<&InspectionAction>,
    mut view: ResMut<InspectionView>, mut map: ResMut<MapViewState>, loaded: Option<Res<LoadedWorldMap>>,
    mut camera: ResMut<MapCameraController>,
) {
    for event in activated.read() {
        let Ok(action) = actions.get(event.0) else { continue; };
        match action {
            InspectionAction::Tab(tab) => view.tab = *tab,
            InspectionAction::Country => { map.inspect_country = true; view.tab = InspectionTab::Overview; }
            InspectionAction::State => { map.inspect_country = false; view.tab = InspectionTab::Overview; }
            InspectionAction::Close => map.selected_index = 0,
            InspectionAction::Province(index) => {
                map.selected_index = *index;
                map.inspect_country = false;
                if let Some(loaded) = &loaded && let Some(uv) = loaded.0.provinces.centroids.get(*index as usize) {
                    camera.target = Vec3::new(uv.x * loaded.0.terrain.size.x, 0., uv.y * loaded.0.terrain.size.y);
                }
            }
        }
    }
}

fn action(commands: &mut Commands, parent: Entity, fonts: &UiFonts, label: &str, action: InspectionAction, selected: bool, order: u32) {
    let mut style = UiButton::secondary(order);
    style.selected = selected;
    let button = ui::button(commands, parent, fonts, if matches!(action, InspectionAction::Close) { "" } else { label }, style);
    use aggregate_ui::icon::Icon;
    let icon = match action {
        InspectionAction::Tab(InspectionTab::Overview) => Some(Icon::Chart),
        InspectionAction::Tab(InspectionTab::Buildings) => Some(Icon::Buildings),
        InspectionAction::Tab(InspectionTab::Population) => Some(Icon::Population),
        InspectionAction::Tab(InspectionTab::Territory) => Some(Icon::Map),
        InspectionAction::Tab(InspectionTab::Programs) => Some(Icon::Programs),
        InspectionAction::Country => Some(Icon::Government),
        InspectionAction::Close => Some(Icon::Close),
        _ => None,
    };
    if let Some(icon) = icon { aggregate_ui::icon::icon(commands, button, icon, 16., theme::TEXT); }
    let compact = matches!(action, InspectionAction::Tab(_) | InspectionAction::Country | InspectionAction::Close);
    commands.entity(button).insert(Node {
        width: if compact { Val::Auto } else { percent(100) },
        min_height: px(34), flex_shrink: 0., padding: UiRect::axes(px(9), px(5)),
        column_gap: px(6),
        border: UiRect::all(px(1)), justify_content: JustifyContent::Center, align_items: AlignItems::Center,
        ..default()
    });
    commands.entity(button).insert(action);
}

fn metric(commands: &mut Commands, parent: Entity, fonts: &UiFonts, label: &str, value: String) {
    let row = ui::node(commands, parent, Node { width: percent(100), justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: px(12), padding: UiRect::vertical(px(5)), ..default() });
    ui::text(commands, row, fonts, label, 13., theme::MUTED, false);
    ui::text(commands, row, fonts, value, 16., theme::TEXT, true);
}

type InspectionRefreshKey = (Entity, u32, bool, InspectionTab, aggregate_localization::Language, u64, usize, usize);

pub fn refresh(
    mut commands: Commands, interface: Res<InterfaceState>, map: Res<MapViewState>, loaded: Option<Res<LoadedWorldMap>>,
    administration: Option<Res<LoadedAdministration>>, view: Res<InspectionView>, fonts: Res<UiFonts>,
    mut session: ResMut<ManagementSession>, roots: Query<Entity, With<InspectionRoot>>, old: Query<Entity, With<InspectionContent>>,
    mut previous: Local<Option<InspectionRefreshKey>>,
) {
    if interface.screen != Screen::WorldMap || map.selected_index == 0 { return; }
    let (Ok(root), Some(loaded), Some(administration)) = (roots.single(), loaded, administration) else { return; };
    let key = (root, map.selected_index, map.inspect_country, view.tab, interface.localization.language(), session.snapshot.day, session.snapshot.facilities.len(), session.snapshot.construction_projects.len());
    if previous.as_ref() == Some(&key) { return; }
    *previous = Some(key);
    let catalog = &loaded.0.catalog;
    let Some(province) = catalog.provinces.get(map.selected_index as usize - 1) else { return; };
    let state_id = administration.0.state_for_province(map.selected_index);
    let state = state_id.and_then(|id| administration.0.states.get(id)).map(|entry| &catalog.states[entry.catalog_index]);
    let country = province.owner.as_ref().and_then(|id| catalog.countries.iter().find(|country| &country.id == id));
    let region = province.region.as_ref().and_then(|id| catalog.regions.iter().find(|region| &region.id == id));
    let state_ids: Vec<_> = if map.inspect_country {
        country.and_then(|country| administration.0.countries.get(&country.id)).map(|ids| ids.iter().collect()).unwrap_or_default()
    } else { state_id.into_iter().collect() };
    let mut indices: Vec<u32> = state_ids.iter().flat_map(|id| administration.0.states[*id].province_indices.iter().copied()).collect();
    if indices.is_empty() { indices.push(map.selected_index); }
    let province_ids: BTreeSet<_> = indices.iter().map(|index| catalog.provinces[*index as usize - 1].id.clone()).collect();
    let simulated: BTreeSet<_> = session.snapshot.provinces.iter().map(|province| &province.id).collect();
    let has_statistics = province_ids.iter().all(|id| simulated.contains(id));
    let groups: Vec<_> = session.snapshot.population_groups.iter().filter(|group| province_ids.contains(&group.province)).collect();
    let population: u128 = groups.iter().map(|group| u128::from(group.population)).sum();
    let workforce: u128 = groups.iter().map(|group| u128::from(group.workforce)).sum();
    let facilities: Vec<_> = session.snapshot.facilities.iter().filter(|facility| province_ids.contains(&facility.province)).collect();
    let projects: Vec<_> = session.snapshot.construction_projects.iter().filter(|project| province_ids.contains(&project.province)).collect();
    for entity in &old { commands.entity(entity).despawn(); }
    let content = ui::node(&mut commands, root, Node { width: percent(100), height: percent(100), flex_direction: FlexDirection::Column, ..default() });
    commands.entity(content).insert(InspectionContent);
    let heading = ui::node(&mut commands, content, Node { width: percent(100), flex_direction: FlexDirection::Column, row_gap: px(7), padding: UiRect::all(px(16)), ..default() });
    commands.entity(heading).insert(BackgroundColor(theme::TITLE_BAR));
    commands.entity(heading).insert(aggregate_ui::window::WindowDragHandle(root));
    let breadcrumb = layout::row(&mut commands, heading, 8.);
    if let Some(country) = country {
        let swatch = ui::node(&mut commands, breadcrumb, Node { width: px(8), height: px(30), ..default() });
        commands.entity(swatch).insert(BackgroundColor(Color::srgb(country.color[0], country.color[1], country.color[2])));
        let country_name = interface.localization.text(&format!("country-{}", country.key.to_lowercase())).unwrap_or_else(|_| country.key.clone());
        action(&mut commands, breadcrumb, &fonts, &country_name, InspectionAction::Country, map.inspect_country, 30);
    }
    action(&mut commands, breadcrumb, &fonts, "×", InspectionAction::Close, false, 31);
    let title = if map.inspect_country { country.map(|country| interface.localization.text(&format!("country-{}", country.key.to_lowercase())).unwrap_or_else(|_| country.key.clone())) } else { region.map(|region| region.name.clone()) }.unwrap_or_else(|| interface.text(if province.water { "map-water" } else { "map-unassigned" }));
    ui::text(&mut commands, heading, &fonts, title, 25., theme::TEXT, true);
    ui::text(&mut commands, heading, &fonts, interface.text(if map.inspect_country { "inspection-country" } else { "inspection-state" }), 12., theme::ACCENT_BRIGHT, false);
    let tabs = ui::node(&mut commands, content, Node { width: percent(100), display: Display::Flex, flex_wrap: FlexWrap::Wrap, column_gap: px(3), row_gap: px(3), padding: UiRect::all(px(8)), ..default() });
    for (index, (tab, label)) in [(InspectionTab::Overview, "inspection-overview"), (InspectionTab::Buildings, "inspection-buildings"), (InspectionTab::Population, "inspection-population"), (InspectionTab::Territory, "inspection-territory"), (InspectionTab::Programs, "inspection-programs")].into_iter().enumerate() {
        action(&mut commands, tabs, &fonts, &interface.text(label), InspectionAction::Tab(tab), view.tab == tab, 32 + index as u32);
    }
    let body = layout::scroll_area(&mut commands, content, Node { width: percent(100), flex_grow: 1., min_height: px(0), padding: UiRect::all(px(16)), flex_direction: FlexDirection::Column, row_gap: px(8), ..default() });
    match view.tab {
        InspectionTab::Overview => {
            metric(&mut commands, body, &fonts, &interface.text("inspection-provinces"), indices.len().to_string());
            if map.inspect_country { metric(&mut commands, body, &fonts, &interface.text("inspection-states"), state_ids.len().to_string()); }
            metric(&mut commands, body, &fonts, &interface.text("inspection-population"), if has_statistics { population.to_string() } else { "—".into() });
            metric(&mut commands, body, &fonts, &interface.text("inspection-workforce"), if has_statistics { workforce.to_string() } else { "—".into() });
            metric(&mut commands, body, &fonts, &interface.text("inspection-buildings"), if has_statistics { facilities.iter().map(|facility| u128::from(facility.level)).sum::<u128>().to_string() } else { "—".into() });
            metric(&mut commands, body, &fonts, &interface.text("inspection-construction"), if has_statistics { projects.len().to_string() } else { "—".into() });
            ui::rule(&mut commands, body);
            if map.inspect_country {
                for (index, id) in state_ids.iter().enumerate() {
                    let entry = &administration.0.states[*id];
                    let state = &catalog.states[entry.catalog_index];
                    let name = catalog.regions.iter().find(|region| region.id == state.region).map(|region| region.name.as_str()).unwrap_or("—");
                    action(&mut commands, body, &fonts, name, InspectionAction::Province(entry.province_indices[0]), false, 200 + index as u32);
                }
            } else if let Some(region) = region {
                let portions = catalog.states.iter().filter(|state| state.region == region.id).count();
                metric(&mut commands, body, &fonts, &interface.text("inspection-region-owners"), portions.to_string());
            }
        }
        InspectionTab::Buildings => {
            for facility in &facilities {
                let definition = session.definitions.facilities.iter().find(|definition| definition.id == facility.definition);
                let name = definition.map(|definition| definition.name.clone()).unwrap_or_else(|| facility.definition.to_string());
                metric(&mut commands, body, &fonts, &name, facility.level.to_string());
            }
            for project in &projects { metric(&mut commands, body, &fonts, &project.definition.to_string(), format!("{} · {}", interface.text("inspection-remaining-work"), project.remaining_worker_days)); }
        }
        InspectionTab::Population => {
            metric(&mut commands, body, &fonts, &interface.text("inspection-population"), if has_statistics { population.to_string() } else { "—".into() });
            metric(&mut commands, body, &fonts, &interface.text("inspection-workforce"), if has_statistics { workforce.to_string() } else { "—".into() });
            for group in &groups { metric(&mut commands, body, &fonts, &group.id.to_string(), group.population.to_string()); }
        }
        InspectionTab::Territory => {
            let mut terrains = BTreeMap::<&str, usize>::new();
            for index in &indices { *terrains.entry(catalog.provinces[*index as usize - 1].terrain.as_str()).or_default() += 1; }
            for (terrain, count) in terrains { let label = interface.localization.text(&format!("terrain-{terrain}")).unwrap_or_else(|_| terrain.into()); metric(&mut commands, body, &fonts, &label, count.to_string()); }
            ui::rule(&mut commands, body);
            for (order, index) in indices.iter().enumerate() {
                let province = &catalog.provinces[*index as usize - 1];
                let terrain = interface.localization.text(&format!("terrain-{}", province.terrain)).unwrap_or_else(|_| province.terrain.clone());
                let label = format!("#{:06X} · {terrain}", province.raster_color);
                action(&mut commands, body, &fonts, &label, InspectionAction::Province(*index), *index == map.selected_index, 500 + order as u32);
            }
        }
        InspectionTab::Programs => {
            let scope = if map.inspect_country { country.map(|country| InspectionScope::Country(country.id.clone())) } else { state.map(|state| InspectionScope::State { id: state.id.clone(), country: state.country.clone(), provinces: province_ids.iter().cloned().collect() }) };
            if let Some(scope) = scope {
                match session.inspect_programs(&scope) {
                    Ok(sections) => {
                        if sections.is_empty() { ui::text(&mut commands, body, &fonts, interface.text("inspection-no-program-data"), 14., theme::MUTED, false); }
                        for section in sections {
                            ui::text(&mut commands, body, &fonts, interface.text(&section.title_key), 16., theme::ACCENT_BRIGHT, true);
                            for item in section.metrics {
                                let value = match item.value { MetricValue::Quantity(value) => value.to_string(), MetricValue::Ratio { numerator, denominator } if denominator != 0 => format!("{:.1}%", numerator as f64 / denominator as f64 * 100.), MetricValue::Localized(key) => interface.text(&key), _ => "—".into() };
                                metric(&mut commands, body, &fonts, &interface.text(&item.label_key), value);
                            }
                        }
                    }
                    Err(error) => { error!(%error, "Program inspection failed"); ui::text(&mut commands, body, &fonts, interface.text("inspection-program-error"), 14., theme::MUTED, false); }
                }
            }
        }
    }
    if !has_statistics && matches!(view.tab, InspectionTab::Overview | InspectionTab::Buildings | InspectionTab::Population) {
        ui::text(&mut commands, body, &fonts, interface.text("inspection-statistics-unavailable"), 13., theme::MUTED, false);
    }
    if map.inspect_country { action(&mut commands, body, &fonts, &interface.text("inspection-return-state"), InspectionAction::State, false, 39); }
}
