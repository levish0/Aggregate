//! Locale selection and Fluent formatting, independent of Bevy and simulation state.
mod catalog;
mod language;
pub use catalog::Localization;
pub use fluent_bundle::FluentArgs;
pub use language::Language;
