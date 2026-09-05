use aggregate_localization::{Language, Localization};
use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    MainMenu,
    Management,
    Preview,
    Settings,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PreviewTab {
    #[default]
    Overview,
    Components,
    Typography,
}

#[derive(Resource)]
pub struct InterfaceState {
    pub screen: Screen,
    pub tab: PreviewTab,
    pub localization: Localization,
    pub scale: f32,
    pub status_key: &'static str,
    pub primary_selected: bool,
    pub reduced_motion: bool,
}

impl Default for InterfaceState {
    fn default() -> Self {
        Self {
            screen: Screen::MainMenu,
            tab: PreviewTab::Overview,
            localization: Localization::new(Language::Korean)
                .expect("validated bundled translations"),
            scale: 1.,
            status_key: "status-ready",
            primary_selected: false,
            reduced_motion: false,
        }
    }
}

impl InterfaceState {
    pub fn format(&self, key: &str, values: &[(&str, String)]) -> String {
        let mut arguments = aggregate_localization::FluentArgs::new();
        for (name, value) in values {
            arguments.set(*name, value.as_str());
        }
        self.localization
            .format(key, Some(&arguments))
            .unwrap_or_else(|error| {
                error!("{error}");
                format!("[{key}]")
            })
    }

    pub fn text(&self, key: &str) -> String {
        self.localization.text(key).unwrap_or_else(|error| {
            error!("{error}");
            format!("[{key}]")
        })
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceAction {
    OpenManagement,
    OpenPreview,
    OpenSettings,
    Back,
    Exit,
    SwitchLanguage,
    SelectTab(PreviewTab),
    ScaleDown,
    ScaleUp,
    ResetScale,
    ToggleMotion,
    PrimaryExample,
    SecondaryExample,
}
