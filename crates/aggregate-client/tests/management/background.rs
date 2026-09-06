use super::{ManagementSession, poll_simulation};
use aggregate_programs::{ProgramContext, ProgramManifest, ProgramPlan, SimulationProgram};
use aggregate_world::WorldSnapshot;
use bevy::prelude::*;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex, mpsc};

struct PausedCalculation {
    started: mpsc::Sender<String>,
    resume: Mutex<mpsc::Receiver<()>>,
}

impl SimulationProgram for PausedCalculation {
    fn manifest(&self) -> ProgramManifest {
        ProgramManifest {
            id: "test.paused_calculation".into(),
            version: aggregate_programs::Version::new(1, 0, 0),
            api_version: 1,
            state_schema_version: 1,
            dependencies: vec![],
            conflicts: vec![],
        }
    }
    fn initialize(&self, _: &WorldSnapshot) -> Result<Value, String> {
        Ok(json!({}))
    }
    fn validate_state(&self, _: &WorldSnapshot, _: &Value) -> Result<(), String> {
        Ok(())
    }
    fn plan_day(&self, _: &ProgramContext<'_>, state: &Value) -> Result<ProgramPlan, String> {
        self.started
            .send(
                std::thread::current()
                    .name()
                    .unwrap_or("unnamed")
                    .to_owned(),
            )
            .unwrap();
        self.resume
            .lock()
            .unwrap()
            .recv()
            .map_err(|error| error.to_string())?;
        Ok(ProgramPlan {
            next_state: state.clone(),
            workforce_limits: Default::default(),
        })
    }
}

#[derive(Resource, Default)]
struct RenderUpdates(usize);

#[test]
fn repeated_clicks_create_immediate_targets_and_failed_commands_remove_only_their_target() {
    let (started, notification) = mpsc::channel();
    let (resume, gate) = mpsc::channel();
    let mut scenario = aggregate_scenario::parse_scenario(include_str!("../../../../scenarios/foundation.json")).unwrap();
    scenario.initial_state.facilities.clear();
    let province = scenario.initial_state.provinces[0].id.clone();
    let country = scenario.initial_state.provinces[0].country.clone();
    scenario.initial_state.provinces[0].stockpile.insert("timber".into(), 25);
    scenario.initial_state.provinces[0].stockpile.insert("tools".into(), 15);
    let mut session = ManagementSession::geographic(scenario, country, vec![Arc::new(aggregate_economy::EconomyProgram), Arc::new(PausedCalculation { started, resume: Mutex::new(gate) })]).unwrap();
    session.step();
    notification.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    for _ in 0..3 { session.start_construction(&province, &"grain_farm".into()); }
    assert_eq!(session.pending_construction.len(), 3);
    assert!(session.snapshot.construction_projects.is_empty());
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).insert_resource(session).add_systems(Update, poll_simulation);
    resume.send(()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while app.world().resource::<ManagementSession>().is_busy() {
        app.update();
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    let session = app.world().resource::<ManagementSession>();
    assert!(session.pending_construction.is_empty());
    assert_eq!(session.snapshot.construction_projects.len(), 2);
    assert!(matches!(session.feedback, super::SessionFeedback::Error(_)));
}

#[test]
fn frames_continue_and_committed_state_stays_stable_while_worker_waits() {
    let (started, notification) = mpsc::channel();
    let (resume, gate) = mpsc::channel();
    let scenario =
        aggregate_scenario::parse_scenario(include_str!("../../../../scenarios/foundation.json"))
            .unwrap();
    let country = scenario.initial_state.countries[0].id.clone();
    let mut session = ManagementSession::geographic(
        scenario,
        country,
        vec![Arc::new(PausedCalculation {
            started,
            resume: Mutex::new(gate),
        })],
    )
    .unwrap();
    let before = session.snapshot.clone();
    session.step();
    assert!(session.is_busy());
    assert_eq!(
        notification
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap(),
        "aggregate-simulation"
    );
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(session)
        .init_resource::<RenderUpdates>()
        .add_systems(
            Update,
            (poll_simulation, |mut updates: ResMut<RenderUpdates>| {
                updates.0 += 1
            }),
        );
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(app.world().resource::<RenderUpdates>().0, 5);
    assert_eq!(app.world().resource::<ManagementSession>().snapshot, before);
    resume.send(()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while app.world().resource::<ManagementSession>().is_busy() {
        app.update();
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(app.world().resource::<ManagementSession>().snapshot.day, 1);
}
