mod state;
use crate::{management::{ManagementSession, presentation}, state::{InterfaceState, Screen}};
use aggregate_ui::{button::{ButtonActivated, UiButton}, components as ui, fonts::UiFonts, icon::Icon, layout::UiPointerBlocker, theme};
use bevy::prelude::*;
use state::NotificationState;

#[derive(Component)]
pub struct NotificationStack;
#[derive(Component)]
pub struct NotificationToast(u64);
#[derive(Component)]
pub struct DismissNotification(u64);

pub fn build(commands: &mut Commands, parent: Entity) {
    let root = ui::node(commands, parent, Node {
        position_type: PositionType::Absolute, right: px(14), bottom: px(70), width: px(300),
        flex_direction: FlexDirection::Column, row_gap: px(8), ..default()
    });
    commands.entity(root).insert((NotificationStack, GlobalZIndex(40), bevy::ui::FocusPolicy::Pass));
}

pub fn refresh(
    mut commands: Commands,
    session: Res<ManagementSession>,
    interface: Res<InterfaceState>,
    fonts: Res<UiFonts>,
    time: Res<Time<Real>>,
    window: Single<&Window>,
    roots: Query<Entity, With<NotificationStack>>,
    cards: Query<(Entity, &NotificationToast, &ComputedNode, &UiGlobalTransform)>,
    mut clicks: MessageReader<ButtonActivated>,
    close_buttons: Query<&DismissNotification>,
    mut state: Local<NotificationState>,
    mut rendered: Local<Option<(Entity, uuid::Uuid, aggregate_localization::Language, Vec<u64>)>>,
) {
    if interface.screen != Screen::WorldMap { clicks.clear(); return; }
    let Ok(root) = roots.single() else { return; };
    state.advance(time.delta_secs(), |sequence| {
        window.physical_cursor_position().is_some_and(|cursor| cards.iter().any(|(_, toast, node, transform)| toast.0 == sequence && Rect::from_center_size(transform.translation, node.size()).contains(cursor)))
    });
    state.receive(session.session_id, &session.notifications);
    for click in clicks.read() {
        if let Ok(close) = close_buttons.get(click.0) { state.active.retain(|toast| toast.entry.sequence != close.0); }
    }
    let key = (root, session.session_id, interface.localization.language(), state.active.iter().map(|toast| toast.entry.sequence).collect());
    if rendered.as_ref() == Some(&key) { return; }
    *rendered = Some(key);
    for (entity, _, _, _) in &cards { commands.entity(entity).despawn(); }
    for (order, toast) in state.active.iter().enumerate() {
        let card = ui::panel(&mut commands, root, Node { width: percent(100), padding: UiRect::all(px(12)), column_gap: px(10), align_items: AlignItems::Center, ..default() });
        commands.entity(card).insert((NotificationToast(toast.entry.sequence), UiPointerBlocker, aggregate_ui::motion::PanelEntrance::new(Vec2::new(16., 0.)), UiTransform::default()));
        aggregate_ui::icon::icon(&mut commands, card, Icon::Buildings, 20., theme::TEXT);
        let text = ui::text(&mut commands, card, &fonts, presentation::event_text(&session, &interface, &toast.entry.event), 13., theme::TEXT, false);
        commands.entity(text).insert(Node { flex_grow: 1., flex_basis: px(0), ..default() });
        let close = super::controls::icon_button(&mut commands, card, &fonts, &interface, Icon::Close, "menu-back", UiButton::secondary(1500 + order as u32));
        commands.entity(close).insert(DismissNotification(toast.entry.sequence));
    }
}
