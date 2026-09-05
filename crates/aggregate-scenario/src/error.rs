use std::{error::Error, fmt, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioError {
    pub source_path: Option<PathBuf>,
    pub field_path: String,
    pub message: String,
}

impl ScenarioError {
    pub(crate) fn new(field_path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            source_path: None,
            field_path: field_path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.source_path {
            write!(formatter, "{}: ", path.display())?;
        }
        write!(formatter, "{}: {}", self.field_path, self.message)
    }
}

impl Error for ScenarioError {}
