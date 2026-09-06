# Aggregate assets

These are editable game resources. Modify, convert and replace them directly as Aggregate evolves.

- `map_data/`: province raster, regions, heightmaps, rivers and adjacency data.
- `gfx/map/`: terrain textures and map presentation resources.
- `gfx/interface/`: editable DDS interface surfaces, frames, buttons and icons.
- `shaders/`: native terrain and UI material shaders.
- `common/country_definitions/`: initial countries and colors.
- `common/history/states/`: initial territory ownership.

The initial hierarchy preserves relative references within the files. There is no vendor or game wrapper. Original import provenance is recorded in [the inventory](../docs/asset-provenance/initial-map-import.json); future edits need not match its initial hashes.

The imported text definitions are now `.ron` files. Images, binary map data and adjacency CSV remain in their existing formats. See [map assets](../docs/MAP_ASSETS.md) for schema boundaries, runtime caches and validation. The completed one-time converter has been removed. Interface import provenance is in `docs/asset-provenance/interface-import.json`.

Binary assets and generated RON files larger than 10 MiB are configured for Git LFS in `.gitattributes`. Nothing is committed or pushed without the user's explicit request.
