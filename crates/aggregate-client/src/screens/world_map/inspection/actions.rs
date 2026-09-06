use super::{InspectionAction, InspectionTab, InspectionView};
use aggregate_map_view::{LoadedWorldMap, MapCameraController, MapViewState};
use aggregate_ui::button::ButtonActivated;
use bevy::prelude::*;

pub fn apply_actions(
    mut activated: MessageReader<ButtonActivated>,
    actions: Query<&InspectionAction>,
    mut view: ResMut<InspectionView>,
    mut map: ResMut<MapViewState>,
    loaded: Option<Res<LoadedWorldMap>>,
    mut camera: ResMut<MapCameraController>,
    session: Res<crate::management::ManagementSession>,
) {
    for event in activated.read() {
        let Ok(action) = actions.get(event.0) else {
            continue;
        };
        match action {
            InspectionAction::Construction => {
                if let Some(loaded) = &loaded
                    && let Some(index) = loaded.0.catalog.provinces.iter().position(|province| {
                        !province.water && province.owner.as_ref() == Some(&session.player_country)
                    })
                {
                    map.selected_index = index as u32 + 1;
                    map.inspect_country = true;
                    view.tab = InspectionTab::Construction;
                    view.page = 0;
                }
            }
            InspectionAction::Tab(tab) => {
                view.tab = *tab;
                view.page = 0;
            }
            InspectionAction::Page(page) => view.page = *page,
            InspectionAction::Country => {
                view.page = 0;
                map.inspect_country = true;
                view.tab = InspectionTab::Overview;
            }
            InspectionAction::State => {
                view.page = 0;
                map.inspect_country = false;
                view.tab = InspectionTab::Overview;
            }
            InspectionAction::Close => map.selected_index = 0,
            InspectionAction::Province(index) => {
                view.page = 0;
                map.selected_index = *index;
                map.inspect_country = false;
                if let Some(loaded) = &loaded
                    && let Some(uv) = loaded.0.provinces.centroids.get(*index as usize)
                {
                    camera.target = Vec3::new(
                        uv.x * loaded.0.terrain.size.x,
                        0.,
                        uv.y * loaded.0.terrain.size.y,
                    );
                }
            }
        }
    }
}
