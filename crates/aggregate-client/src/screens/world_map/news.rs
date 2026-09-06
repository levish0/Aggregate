use crate::{management::{ManagementSession,presentation},state::{InterfaceState,Screen}};
use aggregate_ui::{components as ui,fonts::UiFonts,theme};
use bevy::prelude::*;

#[derive(Component)]
pub struct NewsFeed;
#[derive(Component)]
pub struct NewsFeedContent;

pub fn refresh(mut commands: Commands,session: Res<ManagementSession>,interface: Res<InterfaceState>,fonts: Res<UiFonts>,roots: Query<Entity,With<NewsFeed>>,old: Query<Entity,With<NewsFeedContent>>,mut previous: Local<Option<(Entity,uuid::Uuid,u64,aggregate_localization::Language)>>) {
    if interface.screen != Screen::WorldMap { return; }
    let Ok(root) = roots.single() else { return; };
    let key = (root,session.session_id,session.revision,interface.localization.language());
    if previous.as_ref() == Some(&key) { return; }
    *previous = Some(key);
    for old in &old { commands.entity(old).despawn(); }
    let column = ui::node(&mut commands,root,Node { flex_direction: FlexDirection::Column, row_gap: px(8), width: percent(100), ..default() });
    commands.entity(column).insert(NewsFeedContent);
    if session.news.is_empty() { ui::text(&mut commands,column,&fonts,interface.text("map-news-empty"),13.,theme::MUTED,false); }
    for entry in session.news.iter().rev().take(4) {
        ui::text(&mut commands,column,&fonts,interface.format("management-day",&[("day",entry.day.to_string())]),11.,theme::MUTED,false);
        ui::text(&mut commands,column,&fonts,presentation::event_text(&session,&interface,&entry.event),13.,theme::TEXT,false);
        ui::rule(&mut commands,column);
    }
}
