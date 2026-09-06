# RON map asset conversion

## Decisions and scope

The user authorized scripted conversion of the imported map text to RON and LFS configuration for large files. The user then explicitly prohibited committing until asked and requested undoing the recent asset commit. Local commit `d5c79c9` was undone with a mixed reset, retaining all working files; HEAD and the queried remote main are `1748a76`. No assets were pushed. `AGENTS.md` now records explicit-only commit/push authorization.

## Implementation

- `aggregate-asset-data` holds version 1 of an ordered structural RON import schema, strict document loading and serialization. It depends only on Serde/RON, not Jomini or Bevy.
- `xtask` commands: `convert-map-assets` (dry run), `convert-map-assets --apply`, `validate-map-assets`. Conversion uses the Jomini lexer, recognizes JSON by content, preserves repeated keys/order/operators/exact numeric spelling/headers and source comments, and updates known converted-file references.
- All 218 imported text definitions now live as `.ron` in the project assets tree. Old extensions were removed only after every output was validated and originals were backed up. Default-map topology now refers to `heightmap.ron`.
- Initial migration backup: `target/map-asset-conversion/1788676355204656500/originals/`. Audit report: `docs/asset-provenance/ron-conversion.json`. These hashes document the migration, not constraints on future edits.
- Binary assets plus three generated RON files larger than 10 MiB are configured for Git LFS. No files were staged, committed or pushed. LFS rules are configuration, not an upload.

## Boundaries

The new schema is a structural import representation, not final typed geography/renderer data. It retains engine-specific symbols and condition/effect data without interpreting them. Numeric lexemes remain strings inside `Number` to avoid conversion precision loss. Script text payloads retain original escape spelling. Native Rust simulation rules, JSON foundation scenario and JSON save format are unchanged. Map rendering/picking are not implemented.

## Validation

Workspace formatting, Clippy and 75 tests passed; one optional graphical test remained ignored. All 218 documents round-tripped before application and passed the separate on-disk validator after application. All 505 unconverted original assets still matched their SHA-256 inventory. LFS attributes were verified for all 497 selected files, with ordinary RON excluded from LFS. Git index remained empty after the requested reset. No remote CI or renderer integration was run.
