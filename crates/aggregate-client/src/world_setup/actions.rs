use super::{SetupAction, SetupSelect, WorldInitializationTask, WorldSetup};
use crate::{
    management::{ManagementSession, ManagementViewState},
    state::{InterfaceState, Screen},
};
use aggregate_map_view::{LoadedWorldMap, MapCameraController, MapViewState};
use aggregate_ui::{button::ButtonActivated, select::SelectChanged};
use aggregate_world::CountryId;
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, block_on, futures_lite::future},
};

pub fn apply_actions(
    mut commands: Commands,
    mut setup: ResMut<WorldSetup>,
    mut interface: ResMut<InterfaceState>,
    map: Option<Res<LoadedWorldMap>>,
    task: Option<Res<WorldInitializationTask>>,
    mut changes: MessageReader<SelectChanged>,
    selects: Query<&SetupSelect>,
    mut clicks: MessageReader<ButtonActivated>,
    actions: Query<&SetupAction>,
) {
    if !setup.open || interface.screen != Screen::WorldMap || task.is_some() {
        return;
    }
    for event in changes.read() {
        let Ok(select) = selects.get(event.root) else {
            continue;
        };
        match select {
            SetupSelect::Country => {
                if let Ok(id) = uuid::Uuid::parse_str(&event.value) {
                    setup.country = Some(CountryId(id));
                }
            }
            SetupSelect::Population => {
                if let Ok(value) = event.value.parse() {
                    setup.settings.population_per_province = value;
                }
            }
            SetupSelect::Workforce => {
                if let Ok(value) = event.value.parse() {
                    setup.settings.workforce_percent = value;
                }
            }
            SetupSelect::Reserves => {
                if let Ok(value) = event.value.parse() {
                    setup.settings.construction_goods_per_province = value;
                }
            }
        }
        setup.revision += 1;
    }
    for event in clicks.read() {
        let Ok(action) = actions.get(event.0) else {
            continue;
        };
        match action {
            SetupAction::Close => {
                setup.open = false;
                interface.screen = Screen::MainMenu;
            }
            SetupAction::Start => {
                let (Some(map), Some(country)) = (map.as_ref(), setup.country.clone()) else {
                    continue;
                };
                let map = map.0.clone();
                let settings = setup.settings.clone();
                setup.error = None;
                info!(country = %country, population_per_province = settings.population_per_province, "World initialization requested");
                commands.insert_resource(WorldInitializationTask(
                    AsyncComputeTaskPool::get().spawn(async move {
                        let programs: Vec<
                            std::sync::Arc<dyn aggregate_programs::SimulationProgram>,
                        > = vec![std::sync::Arc::new(aggregate_economy::EconomyProgram)];
                        let definitions = aggregate_programs::ProgramRuntime::collect_definitions(
                            programs.clone(),
                        )?;
                        let scenario = aggregate_world_generation::initialize_world(
                            &map.catalog,
                            &settings,
                            definitions,
                        )?;
                        ManagementSession::geographic(scenario, country, programs)
                            .map_err(|error| error.to_string())
                    }),
                ));
                break;
            }
        }
    }
}

pub fn finish_initialization(
    mut commands: Commands,
    pending: Option<ResMut<WorldInitializationTask>>,
    mut setup: ResMut<WorldSetup>,
    mut session: ResMut<ManagementSession>,
    mut view: ResMut<ManagementViewState>,
    loaded: Option<Res<LoadedWorldMap>>,
    mut map: ResMut<MapViewState>,
    mut camera: ResMut<MapCameraController>,
) {
    let Some(mut pending) = pending else {
        return;
    };
    let Some(result) = block_on(future::poll_once(&mut pending.0)) else {
        return;
    };
    commands.remove_resource::<WorldInitializationTask>();
    match result {
        Ok(created) => {
            *view = created.initial_view();
            if let Some(loaded) = loaded
                && let Some((index, _)) = loaded
                    .0
                    .catalog
                    .provinces
                    .iter()
                    .enumerate()
                    .find(|(_, province)| province.id == view.selected_province)
            {
                map.selected_index = index as u32 + 1;
                map.inspect_country = true;
                let uv = loaded.0.provinces.centroids[index + 1];
                camera.target = Vec3::new(
                    uv.x * loaded.0.terrain.size.x,
                    0.,
                    uv.y * loaded.0.terrain.size.y,
                );
                camera.desired_distance = 160.;
            }
            info!(country = %created.player_country, provinces = created.snapshot.provinces.len(), "Geographic simulation ready");
            *session = created;
            setup.open = false;
        }
        Err(error) => {
            error!(%error, "World initialization failed; previous session preserved");
            setup.error = Some(error);
            setup.revision += 1;
        }
    }
}
