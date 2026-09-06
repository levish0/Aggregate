# Local Victoria 3 map assets

## Decision and scope

The user explicitly chose copying the installed Victoria 3 files into the workspace and using that copy for map development. Do not spend the next milestone implementing a Steam-installation importer instead. The user wants province geography and a terrain map brought forward in the roadmap; 3D character/building models remain separable later work.

## Local inputs

- Source: `D:/SteamLibrary/steamapps/common/Victoria 3/game/`.
- Destination: `reference/victoria3/game/` under the Aggregate workspace, preserving source hierarchy.
- Copied `map_data/`, `gfx/map/`, `common/country_definitions/` and `common/history/states/`.
- Local inventory: `reference/victoria3/asset-manifest.json`, containing relative paths, byte counts, SHA-256 hashes and copy provenance.
- The existing `.gitignore` entry `reference/` excludes the copy and inventory. No original game assets were staged or committed. Ordinary builds do not require them yet.

## Validation and limitations

723 files totaling 1,828,520,823 bytes were copied and individually verified against source SHA-256 hashes. Git exclusion was verified for the province raster, a terrain texture and the manifest; no files beneath `reference/victoria3` were tracked. No Rust code changed, so Rust tests were not rerun for this asset-copy milestone.

The client still renders its decorative background. No province geometry, adjacency extraction, heightfield renderer or map picking was implemented in this copy step. Some copied graphics definitions reference assets elsewhere in the game installation; only the listed subtrees were copied. The source snapshot remains stable if Steam later updates the installation.

## Next work

Use these local inputs to implement geographic province/state contracts, map rendering and selection into the existing management screen. Distinguish map provinces from economic/administrative aggregation; current `ProvinceState` only has country/name/stockpile fields. The previous proposal to prioritize save/load should not obscure the user's requested map priority.

## References

- [Map inputs](../docs/MAP_ASSETS.md)
- [Architecture](../docs/ARCHITECTURE.md)
