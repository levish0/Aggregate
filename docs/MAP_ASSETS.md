# Editable map and interface assets

Runtime inputs live directly under `assets/`: `map_data/` owns province/height rasters and typed geography/settings; `gfx/map/` contains imported map presentation inputs; `gfx/interface/` contains UI textures; `common/` contains structural country/ownership RON; `shaders/` contains Aggregate's Bevy shaders.

The user's local Victoria 3 installation supplied the assets. The initial map import had 723 files (1,828,520,823 bytes): 218 text files became structural RON and 505 others kept their bytes. [Original inventory](asset-provenance/initial-map-import.json), [conversion report](asset-provenance/ron-conversion.json), and [UI inventory](asset-provenance/interface-import.json) record provenance, not a restriction on subsequent edits.

## Runtime contracts

`aggregate-geography` owns validation, persistent IDs, dense province lookup, sampled elevation, ray queries and disposable caches. `geography.ron` contains 40,875 provinces, 781 regions and 730 historical/unused country definitions, not 730 modern sovereign states. UUID v7 values were assigned once and persisted. Pixel colors and dense render indices are lookup addresses. Nil/duplicate UUIDs, unknown references, unknown colors and catalog provinces absent from the raster fail validation. [51 ambiguous imported ownership colors](asset-provenance/ambiguous-ownership.txt) remain unassigned.

`aggregate-map-view` renders terrain chunks shared across five horizontal copies. Picking intersects those same terrain triangles. The source province raster is 8192 x 3616; elevation is 16384 x 7232, 16-bit. `terrain_settings.ron` controls grid density, world width and visual elevation calibration; these are not verified physical meters. The renderer does not yet reproduce the complete Jomini texture, vegetation, river, city and model pipeline.

`aggregate-asset-data` remains the structural RON reader. Imported scripts stay data; native Rust owns mechanisms. The one-time converter and catalog initializer have been removed, and Jomini is no longer a dependency. Original conversion backups remain under `target/map-asset-conversion/1788676355204656500/originals/`. Foundation presets and saves remain JSON.

```sh
cargo run -p xtask --locked -- validate-map-assets
cargo run -p aggregate-geography --example measure_map_loading --locked
```

The validator reads all RON contracts. The loading example also validates PNG/catalog correspondence.

## Derived cache

The client uses `%LOCALAPPDATA%/Aggregate/cache/geography/`, with a temporary-directory fallback. Content hashes of geography, settings and both PNGs plus cache version identify derived lookup/elevation planes. Corrupt data rebuilds from source; edits produce a new key. Cache headers, dimensions, native endianness, checksums, indices and finite values are checked. Writes use a temporary file and rename. Cached indices cannot substitute for persistent UUIDs. Old content-key files are disposable and may be removed separately when reclaiming disk space.

Measured in this Windows development build: source data preparation 20.63 s (catalog 0.68 s, province raster 7.66 s, heightfield 12.29 s); populated cache read 1.35 s, including hashing and validation. The example writes to `target/map-cache/`. These numbers exclude application startup and GPU upload. First preparation still decodes PNGs. Mesh preparation and texture packing run in the asynchronous loading task.

## Epoch and publication boundaries

The requested modern baseline is 2026. Current map names and ownership remain historical. Modern borders, claims/control and demographic/economic data require a separately sourced scenario; a year label must not imply that historical data is current. Real geography is not connected to the synthetic economy yet.

Binary image/map files and three placement RON files above 10 MiB use LFS rules. These local changes and imported assets must not be staged, committed, pushed or published without an explicit user request.
