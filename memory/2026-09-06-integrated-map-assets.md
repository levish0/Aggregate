# Editable Aggregate map assets

## Decision and changes

The user clarified that imported files are to become Aggregate's editable assets, adapted and improved in place. This supersedes the previous local-pack and runtime-pack placement decisions.

Moved all assets directly into `assets/map_data/`, `assets/gfx/map/` and `assets/common/`. Removed the empty `victoria3/game` wrapper. The original import inventory now lives at `docs/asset-provenance/initial-map-import.json`; its hashes record provenance, not a constraint on subsequent edits.

The user explicitly deferred Git work. A local Git LFS initialization had already occurred before that instruction arrived; no LFS attributes, staging or commits were made in this milestone. Do not resume Git work without user direction.

## Validation and limitations

All 723 imported assets matched the inventory sizes and SHA-256 hashes after relocation. No asset format conversion or Rust code change occurred. Rust tests were not rerun. The client does not yet load or render these map resources.
