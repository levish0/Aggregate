//! Opt-in graphical acceptance test using the real client and its button messages.
use crate::{
    management::{ManagementAction, ManagementSession},
    state::InterfaceAction,
};
use aggregate_ui::button::ButtonActivated;
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};

#[derive(Resource, Default)]
struct CaptureRun {
    frame: u32,
    captures: u32,
}

#[test]
#[ignore = "opens a native window and requires a graphics adapter"]
fn native_management_capture() {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screenshots");
    std::fs::create_dir_all(directory).unwrap();
    let mut app = crate::create_app();
    app.insert_resource(bevy::winit::WinitSettings::continuous())
        .init_resource::<CaptureRun>()
        .add_systems(
            Update,
            drive_capture.after(crate::screens::management::update_management_labels),
        );
    assert_eq!(app.run(), AppExit::Success);
}

fn drive_capture(
    mut commands: Commands,
    mut run: ResMut<CaptureRun>,
    interface_buttons: Query<(Entity, &InterfaceAction)>,
    management_buttons: Query<(Entity, &ManagementAction)>,
    mut activated: MessageWriter<ButtonActivated>,
    mut window: Single<&mut Window>,
    session: Res<ManagementSession>,
    mut exit: MessageWriter<AppExit>,
) {
    run.frame += 1;
    let interface_action = match run.frame {
        35 => Some(InterfaceAction::OpenManagement),
        185 => Some(InterfaceAction::SwitchLanguage),
        _ => None,
    };
    if let Some(action) = interface_action {
        let entity = interface_buttons
            .iter()
            .find(|(_, candidate)| **candidate == action)
            .unwrap()
            .0;
        activated.write(ButtonActivated(entity));
    }
    let action = match run.frame {
        55 => Some(ManagementAction::StartConstruction("grain_farm".into())),
        60 | 120 | 125 | 130 | 135 => Some(ManagementAction::StepDay),
        205 => Some(ManagementAction::SelectProvince("south_ridge".into())),
        _ => None,
    };
    if let Some(action) = action {
        let entity = management_buttons
            .iter()
            .find(|(_, candidate)| **candidate == action)
            .unwrap()
            .0;
        activated.write(ButtonActivated(entity));
    }
    if run.frame == 205 {
        window.resolution.set(960., 640.);
    }
    let name = match run.frame {
        25 => Some("management-menu.png"),
        95 => Some("management-construction-ko.png"),
        165 => Some("management-completed-ko.png"),
        250 => Some("management-small-en.png"),
        _ => None,
    };
    if let Some(name) = name {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/screenshots")
            .join(name);
        commands.spawn(Screenshot::primary_window()).observe(
            move |capture: On<ScreenshotCaptured>, mut run: ResMut<CaptureRun>| {
                capture
                    .image
                    .clone()
                    .try_into_dynamic()
                    .unwrap()
                    .to_rgb8()
                    .save(&path)
                    .unwrap();
                run.captures += 1;
            },
        );
    }
    if run.frame >= 300 && run.captures == 4 {
        assert_eq!(session.snapshot.day, 5);
        assert!(session.snapshot.construction_projects.is_empty());
        assert_eq!(session.news.len(), 2);
        exit.write(AppExit::Success);
    }
    assert!(
        run.frame < 1800,
        "native screenshot capture did not complete"
    );
}
