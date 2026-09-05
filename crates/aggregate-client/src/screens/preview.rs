use super::action_button;
use crate::state::{InterfaceAction, InterfaceState, PreviewTab};
use aggregate_ui::{
    button::UiButton,
    components as ui,
    fonts::UiFonts,
    theme,
    tooltip::{TooltipContent, TooltipLink},
};
use bevy::prelude::*;

pub fn build(commands: &mut Commands, root: Entity, fonts: &UiFonts, state: &InterfaceState) {
    let frame = ui::panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(28),
            right: px(28),
            top: px(25),
            bottom: px(65),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    commands.entity(frame).insert((
        aggregate_ui::motion::PanelEntrance::new(Vec2::new(0., 18.)),
        UiTransform::default(),
    ));
    let header = ui::node(
        commands,
        frame,
        Node {
            padding: UiRect::axes(px(26), px(18)),
            align_items: AlignItems::Center,
            column_gap: px(20),
            border: UiRect::bottom(px(1)),
            ..default()
        },
    );
    commands.entity(header).insert((
        BackgroundGradient::from(LinearGradient {
            angle: 1.57,
            stops: vec![theme::BURGUNDY.into(), theme::PANEL.into()],
            ..default()
        }),
        BorderColor::all(theme::GOLD),
    ));
    ui::seal(commands, header, 52.);
    let title = ui::node(
        commands,
        header,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            flex_grow: 1.,
            ..default()
        },
    );
    ui::text(
        commands,
        title,
        fonts,
        state.text("preview-subtitle"),
        12.,
        theme::GOLD,
        false,
    );
    ui::text(
        commands,
        title,
        fonts,
        state.text("preview-title"),
        27.,
        theme::TEXT,
        true,
    );
    let back_container = ui::node(
        commands,
        header,
        Node {
            width: px(125),
            ..default()
        },
    );
    action_button(
        commands,
        back_container,
        fonts,
        &state.text("menu-back"),
        UiButton::secondary(0),
        InterfaceAction::Back,
    );

    let body = ui::node(
        commands,
        frame,
        Node {
            flex_grow: 1.,
            min_height: px(0),
            ..default()
        },
    );
    let navigation = ui::node(
        commands,
        body,
        Node {
            width: px(195),
            flex_shrink: 0.,
            padding: UiRect::all(px(18)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            border: UiRect::right(px(1)),
            ..default()
        },
    );
    commands.entity(navigation).insert((
        BackgroundColor(theme::INK.with_alpha(0.55)),
        BorderColor::all(theme::BORDER),
    ));
    ui::text(
        commands,
        navigation,
        fonts,
        "AGGREGATE",
        12.,
        theme::GOLD,
        true,
    );
    ui::rule(commands, navigation);
    for (index, key, tab) in [
        (1, "nav-overview", PreviewTab::Overview),
        (2, "nav-components", PreviewTab::Components),
        (3, "nav-typography", PreviewTab::Typography),
    ] {
        action_button(
            commands,
            navigation,
            fonts,
            &state.text(key),
            UiButton {
                selected: state.tab == tab,
                ..UiButton::secondary(index)
            },
            InterfaceAction::SelectTab(tab),
        );
    }
    ui::rule(commands, navigation);
    action_button(
        commands,
        navigation,
        fonts,
        &state.text("menu-settings"),
        UiButton::secondary(4),
        InterfaceAction::OpenSettings,
    );
    action_button(
        commands,
        navigation,
        fonts,
        "KO / EN",
        UiButton::secondary(5),
        InterfaceAction::SwitchLanguage,
    );

    let main = ui::node(
        commands,
        body,
        Node {
            flex_grow: 1.,
            flex_basis: px(0),
            min_width: px(0),
            padding: UiRect::all(px(30)),
            flex_direction: FlexDirection::Column,
            row_gap: px(16),
            overflow: Overflow::scroll_y(),
            ..default()
        },
    );
    commands
        .entity(main)
        .insert(aggregate_ui::scroll::ScrollRegion::default());
    ui::text(
        commands,
        main,
        fonts,
        state.text("section-foundation"),
        30.,
        theme::TEXT,
        true,
    );
    ui::text(
        commands,
        main,
        fonts,
        state.text("section-description"),
        16.,
        theme::MUTED,
        false,
    );
    ui::rule(commands, main);
    match state.tab {
        PreviewTab::Overview => {
            panels(commands, main, fonts, state);
            controls(commands, main, fonts, state);
        }
        PreviewTab::Components => {
            controls(commands, main, fonts, state);
            panels(commands, main, fonts, state);
        }
        PreviewTab::Typography => typography(commands, main, fonts, state),
    }
    ui::rule(commands, main);
    let status_label = ui::text(
        commands,
        main,
        fonts,
        state.text(state.status_key),
        14.,
        theme::GOLD,
        false,
    );
    commands.entity(status_label).insert(super::StatusLabel);

    let aside = ui::node(
        commands,
        body,
        Node {
            width: px(245),
            flex_shrink: 0.,
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Column,
            row_gap: px(16),
            border: UiRect::left(px(1)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
    );
    commands.entity(aside).insert((
        BorderColor::all(theme::BORDER),
        BackgroundColor(theme::INK.with_alpha(0.3)),
        aggregate_ui::scroll::ScrollRegion::default(),
    ));
    ui::text(
        commands,
        aside,
        fonts,
        state.text("sidebar-title"),
        20.,
        theme::GOLD_BRIGHT,
        true,
    );
    for (number, title, description) in [
        ("01", "sidebar-one", "sidebar-one-body"),
        ("02", "sidebar-two", "sidebar-two-body"),
        ("03", "sidebar-three", "sidebar-three-body"),
    ] {
        ui::rule(commands, aside);
        ui::text(commands, aside, fonts, number, 25., theme::GOLD, false);
        ui::text(
            commands,
            aside,
            fonts,
            state.text(title),
            17.,
            theme::TEXT,
            true,
        );
        ui::text(
            commands,
            aside,
            fonts,
            state.text(description),
            15.,
            theme::MUTED,
            false,
        );
    }
}

fn panels(commands: &mut Commands, parent: Entity, fonts: &UiFonts, state: &InterfaceState) {
    ui::text(
        commands,
        parent,
        fonts,
        state.text("section-panels"),
        19.,
        theme::GOLD,
        true,
    );
    let row = ui::node(
        commands,
        parent,
        Node {
            column_gap: px(16),
            flex_shrink: 0.,
            ..default()
        },
    );
    for (title, body) in [
        ("panel-primary", "panel-primary-body"),
        ("panel-secondary", "panel-secondary-body"),
    ] {
        let card = ui::panel(
            commands,
            row,
            Node {
                flex_grow: 1.,
                flex_basis: px(0),
                min_width: px(0),
                padding: UiRect::all(px(20)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
        );
        ui::text(
            commands,
            card,
            fonts,
            state.text(title),
            21.,
            theme::TEXT,
            true,
        );
        ui::text(
            commands,
            card,
            fonts,
            state.text(body),
            16.,
            theme::MUTED,
            false,
        );
    }
}

fn controls(commands: &mut Commands, parent: Entity, fonts: &UiFonts, state: &InterfaceState) {
    ui::text(
        commands,
        parent,
        fonts,
        state.text("section-controls"),
        19.,
        theme::GOLD,
        true,
    );
    let row = ui::node(
        commands,
        parent,
        Node {
            column_gap: px(12),
            flex_shrink: 0.,
            ..default()
        },
    );
    action_button(
        commands,
        row,
        fonts,
        &state.text("control-primary"),
        UiButton {
            selected: state.primary_selected,
            ..UiButton::primary(10)
        },
        InterfaceAction::PrimaryExample,
    );
    action_button(
        commands,
        row,
        fonts,
        &state.text("control-secondary"),
        UiButton::secondary(11),
        InterfaceAction::SecondaryExample,
    );
    ui::button(
        commands,
        parent,
        fonts,
        &state.text("control-disabled"),
        UiButton {
            enabled: false,
            ..UiButton::secondary(12)
        },
    );
    let tip = ui::button(
        commands,
        parent,
        fonts,
        &state.text("tooltip-trigger"),
        UiButton::secondary(13),
    );
    commands.entity(tip).insert(TooltipContent {
        title: state.text("tooltip-title"),
        body: state.text("tooltip-body"),
        hint: state.text("tooltip-hint"),
        locking_label: state.text("tooltip-locking"),
        locked_label: state.text("tooltip-locked"),
        links: vec![TooltipLink {
            label: state.text("tooltip-child-link"),
            content: Box::new(TooltipContent {
                title: state.text("tooltip-child-title"),
                body: state.text("tooltip-child-body"),
                hint: state.text("tooltip-hint"),
                locking_label: state.text("tooltip-locking"),
                locked_label: state.text("tooltip-locked"),
                links: vec![],
            }),
        }],
    });
}

fn typography(commands: &mut Commands, parent: Entity, fonts: &UiFonts, state: &InterfaceState) {
    ui::text(
        commands,
        parent,
        fonts,
        state.text("section-type"),
        36.,
        theme::GOLD_BRIGHT,
        true,
    );
    ui::text(
        commands,
        parent,
        fonts,
        state.text("type-body"),
        18.,
        theme::TEXT,
        false,
    );
    ui::rule(commands, parent);
    for size in [28., 21., 16.] {
        ui::text(
            commands,
            parent,
            fonts,
            state.text("type-sample"),
            size,
            theme::TEXT,
            false,
        );
    }
    ui::text(
        commands,
        parent,
        fonts,
        "0123456789   86,400   +12.5%   −3.2%",
        25.,
        theme::GOLD,
        true,
    );
    ui::text(
        commands,
        parent,
        fonts,
        "가나다라마바사 · ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        16.,
        theme::MUTED,
        false,
    );
    ui::text(
        commands,
        parent,
        fonts,
        state.text("type-caption"),
        13.,
        theme::MUTED,
        false,
    );
}
