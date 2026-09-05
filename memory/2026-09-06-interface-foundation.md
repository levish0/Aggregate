# Workspace and native interface foundation

## Scope and status

The initial executable has become a Rust 2024 workspace (minimum Rust 1.98). The default executable is `aggregate-client`. This milestone establishes the workspace and a reusable native interface preview. No nation simulation gameplay or domain models are implemented. The user specifically corrected an earlier proposal to wire university commands before the crate foundation existed; do not invent domain integration in the preview.

## Architecture and public contracts

- Project crates use `aggregate-*`: client, ui, localization and simulation-core. `xtask` is the developer command runner. Add domain crates only when actual contracts and mechanisms exist.
- `aggregate-client` depends on `aggregate-ui` and `aggregate-localization`. It currently has no simulation dependency. Screen composition and interface actions live in the client; fonts, controls, tooltip behavior and motion live in UI.
- `aggregate-simulation-core` depends on `bevy_ecs`, owns an isolated World and Schedule, and exposes `Simulation::step()` and `SimulationClock::tick()`. Ticks have no agreed calendar duration. Overflow is rejected before mutation. This is infrastructure, not a realistic model or a determinism guarantee.
- `aggregate-localization` uses concurrent Fluent bundles without Bevy. Catalogs are embedded from `locales/ko-KR/interface.ftl` and `locales/en-US/interface.ftl`. The caller localizes text before passing it to UI. Missing selected-language messages fall back to English; missing messages or formatting errors remain explicit errors.
- Bevy 0.19.1 uses `default-features = false`, features `2d` and `ui`. No egui dependency. Pretendard 1.3.9 Regular/SemiBold and its OFL license are bundled in `aggregate-ui/assets/fonts`; fonts load directly from embedded bytes. No asset-embedding plugin is needed for these two fixed files.
- `UiButton`, `ButtonActivated`, `TooltipContent`, `TooltipLink`, `TooltipSettings`, `TooltipState`, `MotionValue`, `MotionPreferences`, `PanelEntrance` and `ScrollRegion` describe actual presentation responsibilities. Avoid the leftover `GameButton`/`game-ui` names.
- Buttons support enabled/selected state and Tab/Shift+Tab/Enter/Space navigation using native input focus integration. Screen/tab/language changes rebuild pages. Selection, status, scale and motion preference changes update in place.
- Tooltips use waiting/visible/locked states with a parent stack. Defaults: 250 ms show delay, another 850 ms to lock, 180 ms departure grace. The circular indicator is native UI segments. Click/Enter locks immediately. Children are separate explanation buttons enabled after parent lock. Escape removes the deepest level; outside click closes the stack. Removing a source removes its explanations. Real time is independent of simulation pause/speed.
- Bevy easing curves drive per-instance hover, label press/release, panel entry and smooth scrolling. Retargeting starts at the current displayed value. Reduced motion snaps visual transitions, preserving functional tooltip timing. Wheel input goes to the frontmost scroll region under the pointer.

## Product direction and constraints

- Victoria 3 is the information hierarchy, density and visual-quality reference. The user's local osu!lazer reference informs responsive transition composition. Read-only inspection included RoundedButton/FormButton/SwitchButton/OsuButton and the installed Victoria 3 tooltip timer/lock declarations. No game source or artwork was copied into the product.
- The current atlas background is an original procedural decorative image, not a real province map. The UI honestly presents component examples rather than fabricated treasury/population outcomes.
- Future domains must share authoritative resources/population/budgets and explicit source ownership. Separate parameters, persisted state, derived metrics and allocation/graph solvers. A restricted declarative rules layer may later expose units, dependencies and provenance; do not start with an unrestricted DSL or string-to-number state bag.
- Management should use buildings, policy and automation with optional fine control. Scenario presets/editor, save/replay, simulation command/query contracts, AI, military fronts, news and economic models remain future work.
- Preserve the user's local `reference/` material. It is excluded by their .gitignore change and is not a product dependency. Additional workspace dependencies declared by the user were retained; declaration alone does not mean a crate uses them.

## Validation

- `cargo run -p xtask -- check` passed: workspace formatting, Clippy with warnings denied, and 13 tests across UI/localization/simulation-core.
- UI tests exercise the real Bevy systems without a renderer, including timed creation/locking, nested panels, deepest-first Escape, source cleanup, keyboard activation and outside-click dismissal without stale-focus reopening. Motion tests cover interruption continuity, frame-rate agreement and reduced-motion completion.
- `cargo build --workspace --locked` passed on Windows.
- `cargo run -p xtask -- headless` passed and printed `Headless simulation clock: 100 logical ticks`.
- An earlier native build was launched and its Korean settings screen visually inspected. The user stopped Computer Use with physical Escape. Subsequent motion/nested-tooltip changes were compiled and tested, but were **not** visually exercised in the final executable. Do not treat state tests as visual acceptance.
- Linux workflows now install Bevy platform dependencies and run workspace-wide locked commands. Remote CI, Linux/macOS builds, release builds and performance benchmarks were not run in this session. No push or deployment was requested.

## Remaining work

Visually tune motion, focus and nested tooltip placement at multiple scales. Current placement uses source bounds with an estimated-height viewport clamp; dense stacks and resizing while locked need refinement. Inline rich-text links, table virtualization, text editing/IME, accessibility acceptance and persistent settings are not implemented. Define validated scenario/ID/command contracts before introducing domain simulation screens.

## References

- [Architecture](../docs/ARCHITECTURE.md)
- [Run and validation commands](../README.md)
- [Bevy Linux platform dependencies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md)
