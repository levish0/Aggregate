use super::*;
use crate::{
    screens,
    state::{InterfaceAction, InterfaceState, Screen},
};
use aggregate_ui::{
    button::{ButtonActivated, UiButton},
    fonts::UiFonts,
};
use bevy::prelude::*;

fn app() -> App {
    let mut app = App::new();
    let session = ManagementSession::foundation();
    let view = session.initial_view();
    let state = InterfaceState {
        screen: Screen::Management,
        ..default()
    };
    app.add_plugins(MinimalPlugins)
        .add_message::<ButtonActivated>()
        .insert_resource(state)
        .insert_resource(session)
        .insert_resource(view)
        .insert_resource(UiFonts {
            regular: Handle::default(),
            semibold: Handle::default(),
        })
        .add_systems(
            Update,
            (
                apply_management_actions,
                advance_running_session,
                screens::rebuild,
                screens::management::update_management_lists,
                screens::management::update_management_labels,
            )
                .chain(),
        );
    app.update();
    app
}

fn activate(app: &mut App, action: ManagementAction) -> Entity {
    let entity = app
        .world_mut()
        .query::<(Entity, &ManagementAction)>()
        .iter(app.world())
        .find(|(_, candidate)| **candidate == action)
        .map(|(entity, _)| entity)
        .expect("visible action");
    app.world_mut().write_message(ButtonActivated(entity));
    app.update();
    entity
}

fn texts(app: &mut App) -> Vec<String> {
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .map(|text| text.0.clone())
        .collect()
}

#[test]
fn visible_build_and_time_buttons_drive_core_without_recreating_controls() {
    let mut app = app();
    let initial = app.world().resource::<ManagementSession>().snapshot.clone();
    let button = activate(
        &mut app,
        ManagementAction::StartConstruction("grain_farm".into()),
    );
    let session = app.world().resource::<ManagementSession>();
    assert_eq!(
        session.snapshot.provinces[0].stockpile[&"timber".into()],
        30
    );
    assert_eq!(session.snapshot.provinces[0].stockpile[&"tools".into()], 15);
    assert_eq!(session.snapshot.construction_projects.len(), 1);
    assert_eq!(
        session.snapshot.construction_projects[0]
            .facility_id
            .0
            .get_version_num(),
        7
    );
    assert_eq!(session.news.len(), 1);
    assert!(
        texts(&mut app)
            .iter()
            .any(|text| text.contains("건설 시작"))
    );
    activate(&mut app, ManagementAction::StepDay);
    let session = app.world().resource::<ManagementSession>();
    assert!(session.last_report.as_ref().unwrap().provinces[0].construction_workers > 0);
    assert_eq!(
        session.snapshot.population_groups,
        initial.population_groups
    );
    assert!(app.world().get::<UiButton>(button).is_some());
    for _ in 0..20 {
        if app
            .world()
            .resource::<ManagementSession>()
            .snapshot
            .construction_projects
            .is_empty()
        {
            break;
        }
        activate(&mut app, ManagementAction::StepDay);
    }
    assert!(
        app.world()
            .resource::<ManagementSession>()
            .snapshot
            .construction_projects
            .is_empty()
    );
    assert!(
        texts(&mut app)
            .iter()
            .any(|text| text.contains("건설 완료"))
    );
}

#[test]
fn province_selection_routes_construction_and_refreshes_visible_stocks() {
    let mut app = app();
    activate(
        &mut app,
        ManagementAction::SelectProvince("01a07577-e209-7938-8120-efb504849d04".parse().unwrap()),
    );
    activate(
        &mut app,
        ManagementAction::StartConstruction("grain_farm".into()),
    );
    let session = app.world().resource::<ManagementSession>();
    assert_eq!(
        session.snapshot.construction_projects[0]
            .province
            .to_string(),
        "01a07577-e209-7938-8120-efb504849d04"
    );
    assert_eq!(
        session.snapshot.provinces[0].stockpile[&"timber".into()],
        40
    );
    assert_eq!(
        session.snapshot.provinces[1].stockpile[&"timber".into()],
        30
    );
    assert!(texts(&mut app).iter().any(|text| text == "남부 능선"));
}

#[test]
fn rejected_construction_shows_error_without_spending_or_creating_news() {
    let mut app = app();
    for _ in 0..4 {
        activate(
            &mut app,
            ManagementAction::StartConstruction("grain_farm".into()),
        );
    }
    let before = app.world().resource::<ManagementSession>().snapshot.clone();
    let news_count = app.world().resource::<ManagementSession>().news.len();
    activate(
        &mut app,
        ManagementAction::StartConstruction("grain_farm".into()),
    );
    let session = app.world().resource::<ManagementSession>();
    assert_eq!(session.snapshot, before);
    assert_eq!(session.news.len(), news_count);
    assert!(matches!(session.feedback, SessionFeedback::Error(_)));
    assert!(
        texts(&mut app)
            .iter()
            .any(|text| text.contains("실행할 수 없습니다"))
    );
}

#[test]
fn language_and_screen_changes_preserve_simulation_and_pause_background_time() {
    let mut app = app();
    activate(
        &mut app,
        ManagementAction::StartConstruction("grain_farm".into()),
    );
    let before = app.world().resource::<ManagementSession>().snapshot.clone();
    app.world_mut()
        .resource_mut::<InterfaceState>()
        .localization
        .set_language(aggregate_localization::Language::English);
    app.update();
    assert!(
        texts(&mut app)
            .iter()
            .any(|text| text.contains("Construction started"))
    );
    activate(&mut app, ManagementAction::ToggleRunning);
    app.world_mut().resource_mut::<InterfaceState>().screen = Screen::MainMenu;
    app.update();
    assert!(!app.world().resource::<ManagementSession>().running);
    assert_eq!(app.world().resource::<ManagementSession>().snapshot, before);
    app.world_mut().resource_mut::<InterfaceState>().screen = Screen::Management;
    app.update();
    assert_eq!(app.world().resource::<ManagementSession>().snapshot, before);
    assert!(
        app.world_mut()
            .query::<&InterfaceAction>()
            .iter(app.world())
            .any(|action| *action == InterfaceAction::Back)
    );
}

#[test]
fn real_time_playback_steps_once_and_manual_step_pauses() {
    let mut app = app();
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(600),
    ));
    activate(&mut app, ManagementAction::ToggleRunning);
    app.update();
    assert_eq!(app.world().resource::<ManagementSession>().snapshot.day, 1);
    activate(&mut app, ManagementAction::StepDay);
    assert_eq!(app.world().resource::<ManagementSession>().snapshot.day, 2);
    assert!(!app.world().resource::<ManagementSession>().running);
    app.update();
    assert_eq!(app.world().resource::<ManagementSession>().snapshot.day, 2);
}
