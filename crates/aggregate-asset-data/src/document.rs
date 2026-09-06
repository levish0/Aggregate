use serde::{Deserialize, Serialize};

pub const ASSET_SCHEMA_VERSION: u32 = 1;

/// A structural import schema, independent of the source parser and Bevy.
/// Domain-specific geography and renderer types can consume these documents later.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetDocument {
    pub schema_version: u32,
    pub entries: Vec<AssetEntry>,
}

/// Ordered entries preserve repeated definitions, effects and mixed lists/objects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AssetEntry {
    Property {
        key: String,
        operator: AssetOperator,
        value: AssetValue,
    },
    Item(AssetValue),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetOperator {
    Assign,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Exists,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AssetValue {
    /// Strings and symbols remain distinct; no guessing about engine-specific expressions.
    Text(String),
    Symbol(String),
    /// Exact numeric spelling avoids loss through float conversion during import.
    Number(String),
    Boolean(bool),
    Null,
    /// JSON arrays are distinct from empty objects.
    Array(Vec<AssetValue>),
    Block(Vec<AssetEntry>),
    /// Typed source headers, for example HSV colors, remain explicit data.
    Tagged {
        tag: String,
        value: Box<AssetValue>,
    },
}
