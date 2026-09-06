# Runtime map asset location

## Decision and scope

The user corrected the classification of the copied Victoria 3 files: they are game map resources and must not live in `reference/`. This entry supersedes the destination and exclusion described in `2026-09-06-local-map-assets.md`.

## Changes

- Moved the complete pack from `reference/victoria3/` to `assets/victoria3/`, including the inventory. Map inputs now reside at `assets/victoria3/game/` with their original hierarchy intact.
- The precise `/assets/victoria3/` exclusion preserves the local pack's existing untracked status. Other runtime assets can be versioned normally.
- Updated map and architecture documentation and added `assets/README.md` to distinguish runtime resources from reference material.

## Validation and limitations

Verified all 723 asset files against the manifest's sizes and SHA-256 hashes after the move; no files remain at the former pack location. Rust code and renderer behavior are unchanged, so Rust tests were not rerun. Map loading, geography and rendering remain the next implementation work.
