use super::{SetupAction, SetupRoot, SetupSelect, WorldInitializationTask, WorldSetup};
use crate::state::{InterfaceState, Screen};
use aggregate_map_view::LoadedWorldMap;
use aggregate_ui::{button::UiButton, components as ui, fonts::UiFonts, layout, select, theme};
use bevy::prelude::*;

#[derive(PartialEq, Eq)]
pub struct WorldSetupLayoutState {
    visible: bool,
    revision: u64,
    map_loaded: bool,
    initializing: bool,
    language: aggregate_localization::Language,
}

pub fn rebuild(
    mut commands: Commands,
    mut setup: ResMut<WorldSetup>,
    interface: Res<InterfaceState>,
    fonts: Res<UiFonts>,
    loaded: Option<Res<LoadedWorldMap>>,
    pending: Option<Res<WorldInitializationTask>>,
    roots: Query<Entity, With<SetupRoot>>,
    mut previous: Local<Option<WorldSetupLayoutState>>,
) {
    let visible = setup.open && interface.screen == Screen::WorldMap;
    let key = WorldSetupLayoutState {
        visible,
        revision: setup.revision,
        map_loaded: loaded.is_some(),
        initializing: pending.is_some(),
        language: interface.localization.language(),
    };
    if previous.as_ref() == Some(&key) {
        return;
    }
    *previous = Some(key);
    for root in &roots {
        commands.entity(root).despawn();
    }
    if !visible {
        return;
    }
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.55)),
            GlobalZIndex(80),
            layout::UiPointerBlocker,
            SetupRoot,
        ))
        .id();
    let panel = ui::panel(
        &mut commands,
        root,
        Node {
            width: px(570),
            max_height: percent(92),
            padding: UiRect::all(px(22)),
            row_gap: px(12),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    ui::text(
        &mut commands,
        panel,
        &fonts,
        interface.text("setup-world-title"),
        25.,
        theme::TEXT,
        true,
    );
    ui::text(
        &mut commands,
        panel,
        &fonts,
        interface.text("setup-world-description"),
        13.,
        theme::MUTED,
        false,
    );
    let Some(loaded) = loaded else {
        ui::text(
            &mut commands,
            panel,
            &fonts,
            interface.text("map-loading"),
            16.,
            theme::TEXT,
            false,
        );
        return;
    };
    let playable: std::collections::BTreeSet<_> = loaded
        .0
        .catalog
        .provinces
        .iter()
        .filter(|p| !p.water)
        .filter_map(|p| p.owner.as_ref())
        .collect();
    if setup
        .country
        .as_ref()
        .is_none_or(|id| !playable.contains(id))
    {
        setup.country = loaded
            .0
            .catalog
            .countries
            .iter()
            .find(|country| country.key == "KOR" && playable.contains(&country.id))
            .or_else(|| {
                loaded
                    .0
                    .catalog
                    .countries
                    .iter()
                    .find(|country| playable.contains(&country.id))
            })
            .map(|country| country.id.clone());
    }
    if pending.is_some() {
        ui::text(
            &mut commands,
            panel,
            &fonts,
            interface.text("setup-world-building"),
            16.,
            theme::TEXT,
            false,
        );
        return;
    }
    let mut countries: Vec<_> = loaded
        .0
        .catalog
        .countries
        .iter()
        .filter(|country| playable.contains(&country.id))
        .map(|country| {
            let name = interface
                .localization
                .text(&format!("country-{}", country.key.to_lowercase()))
                .unwrap_or_else(|_| country.key.clone());
            (country.id.to_string(), name)
        })
        .collect();
    countries.sort_by(|a, b| a.1.cmp(&b.1));
    let selected = setup
        .country
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default();
    selector(
        &mut commands,
        panel,
        &fonts,
        &interface.text("setup-country"),
        SetupSelect::Country,
        &selected,
        countries,
        900,
    );
    selector(
        &mut commands,
        panel,
        &fonts,
        &interface.text("setup-population"),
        SetupSelect::Population,
        &setup.settings.population_per_province.to_string(),
        [200, 2000, 20000]
            .map(|n| (n.to_string(), n.to_string()))
            .to_vec(),
        1200,
    );
    selector(
        &mut commands,
        panel,
        &fonts,
        &interface.text("setup-workforce"),
        SetupSelect::Workforce,
        &setup.settings.workforce_percent.to_string(),
        [40, 50, 60]
            .map(|n| (n.to_string(), format!("{n}%")))
            .to_vec(),
        1210,
    );
    selector(
        &mut commands,
        panel,
        &fonts,
        &interface.text("setup-reserves"),
        SetupSelect::Reserves,
        &setup.settings.construction_goods_per_province.to_string(),
        [100, 1000, 10000]
            .map(|n| (n.to_string(), n.to_string()))
            .to_vec(),
        1220,
    );
    ui::rule(&mut commands, panel);
    let mut button = UiButton::secondary(1230);
    button.selected = setup.public_health;
    let toggle = ui::button(
        &mut commands,
        panel,
        &fonts,
        &interface.text(if setup.public_health {
            "setup-health-enabled"
        } else {
            "setup-health-disabled"
        }),
        button,
    );
    commands
        .entity(toggle)
        .insert(SetupAction::TogglePublicHealth);
    ui::text(
        &mut commands,
        panel,
        &fonts,
        interface.text("setup-health-description"),
        12.,
        theme::MUTED,
        false,
    );
    if let Some(error) = &setup.error {
        ui::text(&mut commands, panel, &fonts, error, 13., theme::TEXT, false);
    }
    let row = layout::row(&mut commands, panel, 10.);
    for (key, action, style) in [
        ("menu-back", SetupAction::Close, UiButton::secondary(1240)),
        ("setup-start", SetupAction::Start, UiButton::primary(1241)),
    ] {
        let slot = ui::node(
            &mut commands,
            row,
            Node {
                width: percent(48),
                ..default()
            },
        );
        let button = ui::button(&mut commands, slot, &fonts, &interface.text(key), style);
        commands.entity(button).insert(action);
    }
}

fn selector(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    label: &str,
    kind: SetupSelect,
    value: &str,
    options: Vec<(String, String)>,
    order: u32,
) {
    let row = layout::row(commands, parent, 12.);
    let title = ui::text(commands, row, fonts, label, 14., theme::TEXT, false);
    commands.entity(title).insert(Node {
        width: px(210),
        align_self: AlignSelf::Center,
        ..default()
    });
    let root = select::root(commands, row, value, 280.);
    commands.entity(root).insert(kind);
    let current = options
        .iter()
        .find(|(key, _)| key == value)
        .map(|(_, label)| label.as_str())
        .unwrap_or("—");
    select::trigger(commands, root, fonts, current, order);
    let content = select::content(commands, root);
    for (index, (value, label)) in options.into_iter().enumerate() {
        select::item(
            commands,
            content,
            root,
            fonts,
            &label,
            value,
            order + index as u32 + 1,
        );
    }
}
