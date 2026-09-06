use super::{InspectionAction, InspectionTab};
use aggregate_ui::{button::UiButton, components as ui, fonts::UiFonts, theme};
use bevy::prelude::*;

pub(super) fn action(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    label: &str,
    action: InspectionAction,
    selected: bool,
    order: u32,
) {
    let mut style = UiButton::secondary(order);
    style.selected = selected;
    let button = ui::button(
        commands,
        parent,
        fonts,
        if matches!(action, InspectionAction::Close | InspectionAction::Tab(_)) {
            ""
        } else {
            label
        },
        style,
    );
    use aggregate_ui::icon::Icon;
    let icon = match action {
        InspectionAction::Tab(InspectionTab::Overview) => Some(Icon::Chart),
        InspectionAction::Tab(InspectionTab::Buildings) => Some(Icon::Buildings),
        InspectionAction::Tab(InspectionTab::Construction) | InspectionAction::Construction => {
            Some(Icon::Queue)
        }
        InspectionAction::Tab(InspectionTab::Population) => Some(Icon::Population),
        InspectionAction::Tab(InspectionTab::Territory) => Some(Icon::Map),
        InspectionAction::Tab(InspectionTab::Programs) => Some(Icon::Programs),
        InspectionAction::Country => Some(Icon::Government),
        InspectionAction::Close => Some(Icon::Close),
        _ => None,
    };
    if let Some(icon) = icon {
        aggregate_ui::icon::icon(commands, button, icon, 16., theme::TEXT);
    }
    if matches!(action, InspectionAction::Tab(_)) {
        commands.entity(button).insert((
            Name::new(label.to_string()),
            aggregate_ui::tooltip::TooltipContent {
                title: label.into(),
                body: String::new(),
                hint: String::new(),
                locking_label: String::new(),
                locked_label: String::new(),
                links: vec![],
            },
        ));
    }
    let compact = matches!(
        action,
        InspectionAction::Tab(_) | InspectionAction::Country | InspectionAction::Close
    );
    commands.entity(button).insert(Node {
        width: if compact { Val::Auto } else { percent(100) },
        min_height: px(34),
        flex_shrink: 0.,
        padding: UiRect::axes(px(9), px(5)),
        column_gap: px(6),
        border: UiRect::all(px(1)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    });
    commands.entity(button).insert(action);
}

pub(super) fn metric(
    commands: &mut Commands,
    parent: Entity,
    fonts: &UiFonts,
    label: &str,
    value: String,
) {
    let row = ui::node(
        commands,
        parent,
        Node {
            width: percent(100),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: px(12),
            padding: UiRect::vertical(px(5)),
            ..default()
        },
    );
    ui::text(commands, row, fonts, label, 13., theme::MUTED, false);
    ui::text(commands, row, fonts, value, 16., theme::TEXT, true);
}
