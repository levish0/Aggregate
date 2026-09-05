use aggregate_localization::{Language, Localization};
use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    MainMenu,
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
    pub fn text(&self, key: &str) -> String {
        self.localization.text(key).unwrap_or_else(|error| {
            error!("{error}");
            format!("[{key}]")
        })
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceAction {
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
