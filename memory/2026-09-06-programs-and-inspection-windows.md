# Programs, administration and inspection windows

## Scope and status

Supersedes the historical ownership and textured UI status in the earlier native-map milestone. No commit or push authorized. Broader documentation and full validation sweep remain deferred at the user's request.

## Architecture and decisions

- `aggregate-programs` hosts Rust `SimulationProgram` providers with versioned manifests, dependencies, program-owned save state, transactional daily plans and typed inspection sections. Save schema is 3, ruleset `aggregate-native-economy/2`; initial-world schema remains 2. No runtime mod selector or disease implementation yet.
- Geography schema 2 adds UUID-backed administrative states and `AdministrativeIndex`. The modern ownership baseline has 257 country/territory definitions, 885 states and 40,875 provinces. It projects Natural Earth onto existing province shapes; it is not a verified 2026 control/frontline survey. Review metadata lives in `assets/map_data/modern_ownership.json`. Austria/Australia UUID alias collision was corrected; native loading now succeeds.
- Geographic inspection and the synthetic economic session remain separate. Missing geographic economic statistics display unavailable. The header uses `ManagementSession.player_country`, currently Example Republic; inspecting France must not change the player country.
- UI uses monochrome translucent surfaces and white Heroicons, without imported Victoria UI textures. Shared `FloatingWindow` handles title dragging, all edges/corners, minimum sizes, focus raising and in-session geometry. Inspector files separate actions, layout, components and content. Shared scroll indicators are present.
- All 265 supplied flags match country-flag-icons 1.6.20. Original SVG, native PNG derivatives, source notice and MIT license are under `assets/flags/`, excluded from LFS. Country-key mappings are in `countries.ron`; unknown flags use a neutral icon.
- Terrain combines imported height/color maps, grass/rock/water textures and derived river coverage in WGSL. Province hover now reconstructs continuous coverage like administrative boundaries. Full material masks, normal mapping and vegetation remain unfinished. Map readiness waits for texture dependencies.
- JSONL diagnostics use the Bevy subscriber and rotating files; program/day/map failures are structured. Construction reuses unused production labor and guarantees progress against rounding starvation.

## Validation

Latest client test compilation passed. Two window geometry tests passed. Native map acceptance passed, including picking, camera controls, actual border resize/title drag, dropdowns, KO/EN and small-window rendering. Screenshots were inspected for white icons, flags and province coverage. Cached map data preparation was about 1.2 seconds; texture readiness adds time. Flag Git attributes verified as non-LFS. No full workspace test rerun after these final changes; no full Victoria visual parity or real-world economic acceptance claimed.
