use super::*;

fn app() -> (App, Entity, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<KeyboardFocus>()
        .init_resource::<InputFocus>()
        .init_resource::<SelectInteractionState>()
        .add_message::<ButtonActivated>()
        .add_message::<SelectChanged>()
        .add_systems(Update, interact);
    app.world_mut().spawn(Window::default());
    let root = app
        .world_mut()
        .spawn((
            Node::default(),
            Select {
                value: "terrain".into(),
                open: false,
            },
        ))
        .id();
    let trigger = app.world_mut().spawn(SelectTrigger { root }).id();
    let content = app
        .world_mut()
        .spawn((Node::default(), SelectContent { root }))
        .id();
    let _first = app
        .world_mut()
        .spawn((
            SelectItem {
                root,
                value: "terrain".into(),
                label: "Terrain".into(),
            },
            UiButton::secondary(10),
        ))
        .id();
    let _second = app
        .world_mut()
        .spawn((
            SelectItem {
                root,
                value: "political".into(),
                label: "Political".into(),
            },
            UiButton::secondary(11),
        ))
        .id();
    app.update();
    (app, root, trigger, content)
}

#[test]
fn selecting_an_option_emits_value_and_restores_trigger_focus() {
    let (mut app, root, trigger, content) = app();
    let option = app
        .world_mut()
        .query::<(Entity, &SelectItem)>()
        .iter(app.world())
        .find(|(_, item)| item.value == "political")
        .unwrap()
        .0;
    assert_eq!(
        app.world().get::<Node>(content).unwrap().display,
        Display::None
    );
    assert!(!app.world().get::<UiButton>(option).unwrap().enabled);
    app.world_mut().write_message(ButtonActivated(trigger));
    app.update();
    assert!(app.world().get::<Select>(root).unwrap().open);
    assert_eq!(
        app.world().get::<Node>(content).unwrap().display,
        Display::Flex
    );
    assert!(app.world().get::<UiButton>(option).unwrap().enabled);
    let mut cursor = app
        .world()
        .resource::<Messages<SelectChanged>>()
        .get_cursor_current();
    app.world_mut().write_message(ButtonActivated(option));
    app.update();
    let events: Vec<_> = cursor
        .read(app.world().resource::<Messages<SelectChanged>>())
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].value, "political");
    assert_eq!(app.world().get::<Select>(root).unwrap().value, "political");
    assert!(!app.world().get::<Select>(root).unwrap().open);
    assert_eq!(app.world().resource::<KeyboardFocus>().0, Some(trigger));
    assert!(!app.world().get::<UiButton>(option).unwrap().enabled);
    // A stale activation cannot change a closed select.
    app.world_mut().write_message(ButtonActivated(option));
    app.update();
    assert_eq!(
        cursor
            .read(app.world().resource::<Messages<SelectChanged>>())
            .count(),
        0
    );
}

#[test]
fn arrows_follow_tab_order_and_escape_closes_without_changing_selection() {
    let (mut app, root, trigger, _) = app();
    app.world_mut().write_message(ButtonActivated(trigger));
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowDown);
    app.update();
    let focused = app.world().resource::<KeyboardFocus>().0.unwrap();
    assert_eq!(
        app.world().get::<SelectItem>(focused).unwrap().value,
        "political"
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    assert_eq!(app.world().get::<Select>(root).unwrap().value, "terrain");
    assert!(!app.world().resource::<SelectInteractionState>().any_open);
    assert!(
        app.world()
            .resource::<SelectInteractionState>()
            .dismissed_this_frame
    );
    assert_eq!(app.world().resource::<KeyboardFocus>().0, Some(trigger));
}
