use bevy::prelude::*;

/// Presentation text is already localized by the caller.
#[derive(Component, Clone)]
pub struct TooltipContent {
    pub title: String,
    pub body: String,
    pub hint: String,
    pub locking_label: String,
    pub locked_label: String,
    pub links: Vec<TooltipLink>,
}

#[derive(Clone)]
pub struct TooltipLink {
    pub label: String,
    pub content: Box<TooltipContent>,
}
