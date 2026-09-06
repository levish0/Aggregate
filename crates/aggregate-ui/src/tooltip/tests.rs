use super::{
    TooltipContent, TooltipLink, TooltipState,
    panel::{TooltipPanel, TooltipParent},
    state::TooltipPhase,
};
use crate::{AggregateUiPlugin, button::UiButton};
use bevy::{input::mouse::MouseWheel, prelude::*};
use std::time::Duration;

fn content() -> TooltipContent {
    TooltipContent {
        title: "Explanation".into(),
        body: "Body".into(),
        hint: "Escape".into(),
        locking_label: "Locking".into(),
        locked_label: "Locked".into(),
        links: vec![],
    }
}

fn advance(app: &mut App, seconds: f32) {
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(Duration::from_secs_f32(seconds));
    app.update();
}

fn interface_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        bevy::asset::AssetPlugin::default(),
        bevy::image::ImagePlugin::default(),
    ))
    .init_asset::<bevy::shader::Shader>()
    .init_resource::<Assets<Font>>()
    .init_resource::<Time<Real>>()
    .init_resource::<ButtonInput<KeyCode>>()
    .init_resource::<ButtonInput<MouseButton>>()
    .init_resource::<UiScale>()
    .add_message::<MouseWheel>()
    .add_plugins(AggregateUiPlugin);
    app.world_mut().spawn(Window::default());
    app
}

#[test]
fn native_systems_create_lock_nest_and_dismiss_tooltip_panels() {
    let mut app = interface_app();
    let mut explanation = content();
    explanation.links.push(TooltipLink {
        label: "Child".into(),
        content: Box::new(content()),
    });
    let source = app
        .world_mut()
        .spawn((
            Button,
            UiButton::secondary(0),
            explanation,
            Interaction::Hovered,
        ))
        .id();
    advance(&mut app, 0.3);
    assert_eq!(
        app.world().resource::<TooltipState>().entries[0].phase,
        TooltipPhase::Visible
    );
    let child = app
        .world_mut()
        .query_filtered::<Entity, With<TooltipParent>>()
        .single(app.world())
        .unwrap();
    assert!(!app.world().get::<UiButton>(child).unwrap().enabled);
    advance(&mut app, 0.9);
    assert!(app.world().get::<UiButton>(child).unwrap().enabled);
    *app.world_mut().get_mut::<Interaction>(source).unwrap() = Interaction::None;
    *app.world_mut().get_mut::<Interaction>(child).unwrap() = Interaction::Hovered;
    advance(&mut app, 1.2);
    assert_eq!(app.world().resource::<TooltipState>().entries.len(), 2);
    assert_eq!(
        app.world_mut()
            .query::<&TooltipPanel>()
            .iter(app.world())
            .count(),
        2
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    advance(&mut app, 0.01);
    assert_eq!(app.world().resource::<TooltipState>().entries.len(), 1);
    assert!(app.world().resource::<TooltipState>().dismissed_this_frame);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    advance(&mut app, 1.2);
    assert_eq!(
        app.world().resource::<TooltipState>().entries.len(),
        1,
        "hover must not reopen a dismissed child"
    );
    app.world_mut().despawn(source);
    advance(&mut app, 0.01);
    assert!(app.world().resource::<TooltipState>().entries.is_empty());
    assert_eq!(
        app.world_mut()
            .query::<&TooltipPanel>()
            .iter(app.world())
            .count(),
        0
    );
}

#[test]
fn keyboard_activation_locks_and_outside_click_does_not_reopen_from_stale_focus() {
    let mut app = interface_app();
    app.world_mut()
        .spawn((Button, UiButton::secondary(0), content()));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Tab);
    advance(&mut app, 0.01);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);
    advance(&mut app, 0.01);
    assert_eq!(
        app.world().resource::<TooltipState>().entries[0].phase,
        TooltipPhase::Locked
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    advance(&mut app, 0.01);
    assert!(app.world().resource::<TooltipState>().entries.is_empty());
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    advance(&mut app, 2.);
    assert!(app.world().resource::<TooltipState>().entries.is_empty());
}
