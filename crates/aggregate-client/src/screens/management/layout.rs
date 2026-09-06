use super::{ManagementLabel, ManagementList};
use crate::{
    management::{ManagementAction, ManagementSession, presentation},
    screens::action_button,
    state::{InterfaceAction, InterfaceState},
};
use aggregate_ui::{
    button::UiButton, components as ui, fonts::UiFonts, theme, tooltip::TooltipContent,
};
use bevy::prelude::*;

pub fn build(
    commands: &mut Commands,
    root: Entity,
    fonts: &UiFonts,
    state: &InterfaceState,
    session: &ManagementSession,
) {
    let frame = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            right: px(24),
            top: px(20),
            bottom: px(58),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    commands.entity(frame).insert((
        aggregate_ui::motion::PanelEntrance::new(Vec2::new(0., 12.)),
        UiTransform::default(),
    ));
    let header = ui::node(
        commands,
        frame,
        Node {
            padding: UiRect::axes(px(22), px(14)),
            align_items: AlignItems::Center,
            column_gap: px(20),
            border: UiRect::bottom(px(1)),
            flex_shrink: 0.,
            ..default()
        },
    );
    commands.entity(header).insert((
        BackgroundColor(theme::TITLE_BAR),
        BorderColor::all(theme::ACCENT),
    ));
    ui::seal(commands, header, 44.);
    let title = column(commands, header, 4.);
    commands.entity(title).insert(Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(4),
        flex_grow: 1.,
        ..default()
    });
    ui::text(
        commands,
        title,
        fonts,
        state.text("management-title"),
        12.,
        theme::ACCENT,
        true,
    );
    label(
        commands,
        title,
        fonts,
        ManagementLabel::Country,
        24.,
        theme::TEXT,
    );
    for (key, binding) in [
        ("management-population", ManagementLabel::Population),
        ("management-workforce", ManagementLabel::Workforce),
    ] {
        let stat = column(commands, header, 3.);
        ui::text(
            commands,
            stat,
            fonts,
            state.text(key),
            12.,
            theme::MUTED,
            false,
        );
        label(commands, stat, fonts, binding, 24., theme::ACCENT_BRIGHT);
    }
    let back = ui::node(
        commands,
        header,
        Node {
            width: px(106),
            ..default()
        },
    );
    action_button(
        commands,
        back,
        fonts,
        &state.text("menu-back"),
        UiButton::secondary(0),
        InterfaceAction::Back,
    );
    let toolbar = ui::node(
        commands,
        frame,
        Node {
            padding: UiRect::axes(px(22), px(10)),
            align_items: AlignItems::Center,
            column_gap: px(12),
            flex_shrink: 0.,
            ..default()
        },
    );
    label(
        commands,
        toolbar,
        fonts,
        ManagementLabel::Day,
        23.,
        theme::TEXT,
    );
    let help = label(
        commands,
        toolbar,
        fonts,
        ManagementLabel::ReportDay,
        12.,
        theme::MUTED,
    );
    commands.entity(help).insert(Node {
        flex_grow: 1.,
        ..default()
    });
    button_slot(
        commands,
        toolbar,
        fonts,
        &state.text("management-play"),
        ManagementAction::ToggleRunning,
        1,
        112.,
    );
    button_slot(
        commands,
        toolbar,
        fonts,
        &state.text("management-step"),
        ManagementAction::StepDay,
        2,
        142.,
    );
    let lang = ui::node(
        commands,
        toolbar,
        Node {
            width: px(96),
            ..default()
        },
    );
    action_button(
        commands,
        lang,
        fonts,
        "KO / EN",
        UiButton::secondary(3),
        InterfaceAction::SwitchLanguage,
    );
    let feedback = ui::node(
        commands,
        frame,
        Node {
            padding: UiRect::axes(px(22), px(8)),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.,
            ..default()
        },
    );
    label(
        commands,
        feedback,
        fonts,
        ManagementLabel::Feedback,
        13.,
        theme::ACCENT_BRIGHT,
    );
    let body = ui::node(
        commands,
        frame,
        Node {
            flex_grow: 1.,
            min_height: px(0),
            border: UiRect::top(px(1)),
            ..default()
        },
    );
    commands
        .entity(body)
        .insert(BorderColor::all(theme::BORDER));
    let navigation = scroll_column(commands, body, Some(184.), 16.);
    ui::text(
        commands,
        navigation,
        fonts,
        state.text("management-provinces"),
        17.,
        theme::ACCENT_BRIGHT,
        true,
    );
    ui::rule(commands, navigation);
    for (index, province) in session
        .snapshot
        .provinces
        .iter()
        .filter(|province| province.country == session.player_country)
        .enumerate()
    {
        management_button(
            commands,
            navigation,
            fonts,
            &presentation::province_name(session, state, &province.id),
            ManagementAction::SelectProvince(province.id.clone()),
            10 + index as u32,
        );
    }
    ui::rule(commands, navigation);
    ui::text(
        commands,
        navigation,
        fonts,
        state.text("management-scenario-note"),
        13.,
        theme::MUTED,
        false,
    );

    section(
        commands,
        navigation,
        fonts,
        state,
        "management-construction",
    );
    let projects = column(commands, navigation, 10.);
    commands.entity(projects).insert(ManagementList::Projects);

    let main = scroll_column(commands, body, None, 22.);
    label(
        commands,
        main,
        fonts,
        ManagementLabel::Province,
        28.,
        theme::TEXT,
    );
    let allocation = label(
        commands,
        main,
        fonts,
        ManagementLabel::Allocation,
        14.,
        theme::ACCENT,
    );
    commands.entity(allocation).insert(tooltip(
        state,
        "management-labor-title",
        "management-labor-help",
    ));
    label(
        commands,
        main,
        fonts,
        ManagementLabel::FoodShortfall,
        13.,
        theme::MUTED,
    );
    section(commands, main, fonts, state, "management-stockpiles");
    let stock_row = ui::node(
        commands,
        main,
        Node {
            column_gap: px(10),
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(10),
            flex_shrink: 0.,
            ..default()
        },
    );
    for good in &session.definitions.goods {
        let card = ui::panel(
            commands,
            stock_row,
            Node {
                min_width: px(128),
                flex_grow: 1.,
                padding: UiRect::all(px(13)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
        );
        ui::text(
            commands,
            card,
            fonts,
            presentation::good_name(session, state, &good.id),
            14.,
            theme::MUTED,
            false,
        );
        label(
            commands,
            card,
            fonts,
            ManagementLabel::Stock(good.id.clone()),
            28.,
            theme::ACCENT_BRIGHT,
        );
        label(
            commands,
            card,
            fonts,
            ManagementLabel::StockChange(good.id.clone()),
            13.,
            theme::MUTED,
        );
        commands.entity(card).insert(tooltip(
            state,
            "management-stockpiles",
            "management-stock-help",
        ));
    }
    section(commands, main, fonts, state, "management-production");
    for (index, definition) in session.definitions.facilities.iter().enumerate() {
        let card = ui::panel(
            commands,
            main,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                padding: UiRect::all(px(15)),
                flex_shrink: 0.,
                ..default()
            },
        );
        let heading = ui::node(
            commands,
            card,
            Node {
                column_gap: px(12),
                align_items: AlignItems::Center,
                ..default()
            },
        );
        let title = ui::text(
            commands,
            heading,
            fonts,
            presentation::facility_name(session, state, &definition.id),
            19.,
            theme::TEXT,
            true,
        );
        commands.entity(title).insert(Node {
            flex_grow: 1.,
            ..default()
        });
        button_slot(
            commands,
            heading,
            fonts,
            &state.text("management-build"),
            ManagementAction::StartConstruction(definition.id.clone()),
            100 + index as u32,
            110.,
        );
        label(
            commands,
            card,
            fonts,
            ManagementLabel::FacilityStaffing(definition.id.clone()),
            13.,
            theme::ACCENT,
        );
        ui::text(
            commands,
            card,
            fonts,
            presentation::recipe_description(session, state, definition),
            13.,
            theme::MUTED,
            false,
        );
    }
    let news = scroll_column(commands, body, Some(252.), 18.);
    commands
        .entity(news)
        .insert(BackgroundColor(theme::INK.with_alpha(0.6)));
    ui::text(
        commands,
        news,
        fonts,
        state.text("management-news"),
        21.,
        theme::ACCENT_BRIGHT,
        true,
    );
    ui::text(
        commands,
        news,
        fonts,
        state.text("management-news-description"),
        12.,
        theme::MUTED,
        false,
    );
    let news_list = column(commands, news, 12.);
    commands.entity(news_list).insert(ManagementList::News);
}

pub(super) fn column(commands: &mut Commands, parent: Entity, gap: f32) -> Entity {
    ui::node(
        commands,
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(gap),
            flex_shrink: 0.,
            ..default()
        },
    )
}

fn scroll_column(
    commands: &mut Commands,
    parent: Entity,
    width: Option<f32>,
    padding: f32,
) -> Entity {
    let entity = ui::node(
        commands,
        parent,
        Node {
            width: width.map(px).unwrap_or(Val::Auto),
            flex_grow: if width.is_none() { 1. } else { 0. },
            flex_basis: if width.is_none() { px(0) } else { Val::Auto },
            flex_shrink: 0.,
            min_width: px(0),
            min_height: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            padding: UiRect::all(px(padding)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
    );
    commands
        .entity(entity)
        .insert(aggregate_ui::scroll::ScrollRegion::default());
    entity
}

pub(super) fn label(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    binding: ManagementLabel,
    size: f32,
    color: Color,
) -> Entity {
    let entity = ui::text(commands, parent, fonts, "", size, color, size >= 19.);
    commands.entity(entity).insert(binding);
    entity
}

fn section(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    state: &InterfaceState,
    key: &str,
) {
    ui::rule(commands, parent);
    ui::text(
        commands,
        parent,
        fonts,
        state.text(key),
        18.,
        theme::ACCENT_BRIGHT,
        true,
    );
}

fn management_button(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    text: &str,
    action: ManagementAction,
    order: u32,
) -> Entity {
    let button = ui::button(commands, parent, fonts, text, UiButton::secondary(order));
    commands.entity(button).insert(action);
    button
}

fn button_slot(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    text: &str,
    action: ManagementAction,
    order: u32,
    width: f32,
) -> Entity {
    let slot = ui::node(
        commands,
        parent,
        Node {
            width: px(width),
            flex_shrink: 0.,
            ..default()
        },
    );
    management_button(commands, slot, fonts, text, action, order)
}

fn tooltip(state: &InterfaceState, title: &str, body: &str) -> TooltipContent {
    TooltipContent {
        title: state.text(title),
        body: state.text(body),
        hint: state.text("tooltip-hint"),
        locking_label: state.text("tooltip-locking"),
        locked_label: state.text("tooltip-locked"),
        links: Vec::new(),
    }
}
