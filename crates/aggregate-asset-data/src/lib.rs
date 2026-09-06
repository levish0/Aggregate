//! Editable asset documents. This crate reads data; it does not execute imported conditions.

mod document;
mod loading;

pub use document::{ASSET_SCHEMA_VERSION, AssetDocument, AssetEntry, AssetOperator, AssetValue};
pub use loading::{parse_asset_document, serialize_asset_document};
