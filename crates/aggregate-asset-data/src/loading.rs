use crate::{ASSET_SCHEMA_VERSION, AssetDocument};

pub fn parse_asset_document(source: &str) -> Result<AssetDocument, String> {
    let document: AssetDocument = ron::from_str(source).map_err(|error| error.to_string())?;
    if document.schema_version != ASSET_SCHEMA_VERSION {
        return Err(format!(
            "unsupported asset schema version: {}",
            document.schema_version
        ));
    }
    Ok(document)
}

pub fn serialize_asset_document(document: &AssetDocument) -> Result<String, ron::Error> {
    ron::ser::to_string_pretty(
        document,
        ron::ser::PrettyConfig::new()
            .new_line("\n")
            .depth_limit(6)
            .compact_structs(true),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_versions_fields_and_trailing_documents() {
        assert!(parse_asset_document("(schema_version: 2, entries: [])").is_err());
        assert!(parse_asset_document("(schema_version: 1, entries: [], extra: 3)").is_err());
        assert!(parse_asset_document("(schema_version: 1, entries: []) ()").is_err());
    }
}
