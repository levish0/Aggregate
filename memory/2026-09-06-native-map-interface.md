# Native geography, camera and textured interface

## Scope and status

World Map is a native terrain viewer, separate from the synthetic economic management session. Historical source ownership has not been converted to 2026. The user chose 2026 as the eventual modern baseline; source modern borders/control, demographics and economics separately instead of relabeling historical data.

## Architecture and public contracts

- `aggregate-geography` owns validated catalogs, dense province rasters, heightfields, terrain ray picking and derived caches. `aggregate-map-view` owns asynchronous preparation, shared terrain meshes, map materials, camera and overview. `aggregate-asset-data` remains the structural RON reader; the spent converter and catalog initializer were removed.
- CountryId, ProvinceId and RegionId now wrap UUIDs alongside facility/population IDs. Generate UUIDv7 once at creation and persist/replay it. Render indices and raster colors are addresses, not identities. Scenario/save schemas are version 2; native ruleset is version 1. Foundation fixture and recorded commands migrated together.
- `assets/map_data/geography.ron` holds 40,875 provinces, 781 regions and 730 historical/unused country definitions. 51 ambiguous imported ownership colors remain unassigned. Shared terrain meshes appear across five horizontal copies; ray picking uses the actual sampled triangle surface. North/south margins use procedural tabletop artwork.
- Content-addressed BLAKE3 + zstd caches at `%LOCALAPPDATA%/Aggregate/cache/geography/` retain derived raster/elevation planes. Checksums, dimensions, endianness and indices are validated. Corrupt cache rebuilds from authoritative RON/PNG. The cache example uses `target/map-cache/`. Measured development data preparation: 20.63 s from source, 1.35 s populated cache read; excludes GPU upload and full application startup. First load still decodes sources. Mesh preparation and texture packing run off the main thread.
- Imported 323 editable DDS interface files under `assets/gfx/interface/`; provenance is `docs/asset-provenance/interface-import.json`. `aggregate-ui::skin` owns SurfaceMaterial, PanelSkin and TextureFrame; native shader layers textures and sliced frames. No egui. This is a partial visual port; original headers, icon rails and all domain panels are not finished. Pretendard and Fluent remain font/localization boundaries.

## Verified input and reference decisions

- The user physically verified in Victoria 3: middle-button drag pans; right-button drag changes pitch/yaw. Pitch persists after release, heading returns smoothly to the pre-drag direction. Aggregate implements this; reduced motion snaps heading back. Gestures start over the map and remain captured until release.
- Tab holds world overview with a white projected viewport footprint. Pointer motion previews, left click commits a destination, release restores the prior zoom/orientation at that destination. No click restores the original destination. Overview clicks do not change province selection. Normal clicks ray-pick provinces; Esc closes inspection before leaving the map.
- The actual wiki was read through browser after web fetch returned 401. It lists Q as Ledger and Tab as overview. Installed 1.13 `input_profile/default.profile` differs from wiki: Shift+F1 opens Companies. Do not bind Shift+F1 to outliner or Q to map modes based on the older table. Alt+2 currently enables political coloring.
- Country/state/strategic-region right-click menus were observed in the game but are still pending in Aggregate. A future context-menu implementation must distinguish right click from camera drag. Do not claim exact Victoria control/UI parity.

## Validation

- Workspace all-target tests passed after initializing AssetPlugin/ImagePlugin/Shader assets in the headless tooltip harness; the new material plugin requires these resources.
- Workspace Clippy with warnings denied and formatting passed. AnimatedButton uses a named mutable QueryData following existing UI conventions.
- Native map acceptance passed after mouse correction: middle pan without rotation, right rotation without translation, pitch preservation and heading return, overview preview/commit, horizontal wrapping, actual province picking, dropdown/UI blocking, Escape, KO/EN rendering.
- Native management acceptance passed with textured UI. Captures are in `target/screenshots/`; map and management output was visually inspected.
- Geographic tests cover cache corruption/rebuild/source invalidation, persistent UUID lookup across catalog reorder and wrapped ray hits. All 220 RON assets passed `xtask validate-map-assets`. Final focused UI/localization/headless runs are recorded below when complete.
- No commit, staging, push, remote CI, release or modern-scenario acceptance. Imported assets remain local; commit/push only on explicit user request.

## Remaining work

Modern 2026 scenario sourcing; connect geography and management through explicit domain IDs; contextual country/state selection and click menus; full map texture/river/vegetation/detail pipeline; original-quality header/icon/scrollbar composition; measure large-world GPU/frame performance. The nation simulation is not complete.
