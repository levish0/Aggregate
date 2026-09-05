# Native management screen integration

## Scope and status

The main menu now opens a working management screen backed by the native simulation. This supersedes the previous milestone's statement that the client has no simulation dependency. The separate interface/component preview remains available. The client currently runs only the bundled development scenario; it is not a complete nation game.

## Architecture and public contracts

- `aggregate-client` now depends on `aggregate-simulation-core`, `aggregate-scenario`, `aggregate-world` and UUID. `aggregate-ui` still has no domain dependency. Project source roles are explicit: `management/session.rs` owns core execution and read caches, `management/actions.rs` routes actions/playback, `management/presentation.rs` localizes domain facts, and `screens/management/{layout,bindings,lists}.rs` compose/update the native screen.
- `ManagementSession` wraps one authoritative `Simulation`, current snapshot, definitions, last completed report, typed news and feedback. UI code never writes the snapshot into the core. `ManagementViewState` separately tracks the selected province. The first bundled country is player-controlled.
- `ManagementAction` uses existing `ButtonActivated` messages for province selection, construction, single-day stepping and run/pause. The menu's `InterfaceAction::OpenManagement` opens the new screen.
- Construction selects the current province, supplies a fresh UUID, requests the recipe's maximum builders and uses the lowest existing production priority for that building kind, or 100 if no instance exists. All acceptance and resource spending go through the core. There is no client-side cost deduction or fabricated result.
- `SimulationError::InsufficientConstructionGoods { province, good, required, available }` is a new typed public error for localized material-shortage feedback. Other command/invariant failures retain their diagnostic error text. Successful rule semantics, save schema and ruleset version did not change.
- A session starts paused at day zero. Single-day stepping also pauses. Run mode advances approximately one day per real second, at most one day per rendered frame without catch-up bursts. Leaving management pauses time and clears elapsed playback time. A core failure pauses automatically. Returning, switching province or changing language preserves the same simulation.

## Presentation and localization

- Country totals/day are in the header. Left navigation selects a province and shows construction progress; the center shows province labor/food reports, current stock and building recipes; the right column shows typed construction/shortage news newest first. Feedback is above scrollable content so rejected commands remain visible.
- Day-zero reports explicitly say no day has run. Stock deltas describe the last completed day's production minus input and household consumption, excluding between-day construction costs. Tooltips explain those boundaries and shared labor. Completion participates in production on the following day, as in the core.
- Existing buttons and scrolling containers survive simulation ticks. Labels and progress update in place when session/view data or newly created bindings change. Construction row membership and news event count determine list rebuilds. News is not truncated; large-list virtualization remains future work.
- Fluent now formats parameterized management/news/error text. `FluentArgs` is re-exported by `aggregate-localization`. Tests exercise all keys with the declared variable set, matching catalogs and required event variables. Bundled country/province/good/facility names have KO/EN keys; authored names are the fallback for missing content translations. News stores facts and is reformatted when language changes.
- Pretendard, native Bevy UI, motion and timed tooltip infrastructure remain in use. No egui, 3D assets or copied game art was introduced.

## Validation

- `cargo run -p xtask -- check` passed: formatting, workspace/all-target Clippy with warnings denied, and 68 tests. The Windows graphical test is ignored in this ordinary check.
- Five new client tests drive visible button messages through the core and back into text: shared labor/construction completion without control recreation, selected-province stock changes, rejection without spending/news, session preservation across language/screens, and real-time playback/manual pause.
- One additional localization test verifies required shortage-event variables in both languages. Existing core save/replay/accounting tests remain passing after the typed error change.
- `cargo test -p aggregate-client native_management_capture --locked -- --ignored --nocapture` passed separately. It opens the real client, navigates from the menu, starts construction, steps to day five, switches to English and selects the other province. It captures menu, active construction, completion and a 960x640 logical window. The initial logical size is 1440x900; Windows DPI scaling produced 2160x1350 and 1440x960 pixel captures. The Korean construction and English small-window renders were visually inspected. The graphical test uses actual action messages, not synthesized physical mouse/keyboard input.
- `cargo build --workspace --locked` passed on Windows. Generated captures are in `target/screenshots/management-*.png` and are not committed.
- Remote CI, non-Windows/release builds, performance profiling and physical input acceptance were not performed. No push or deployment was requested.

## Remaining work

Add client preset selection, save/load controls and durable sessions before claiming restart persistence. Closing the app currently discards the session; existing headless save/replay APIs remain available. Add richer province/facility inspection, scalable lists and later domain mechanisms against the existing shared contracts. The map remains decorative; geography/province interaction is not implemented.

## References

- [Architecture](../docs/ARCHITECTURE.md)
- [Run instructions](../README.md)
- [Previous native simulation milestone](2026-09-06-native-simulation.md)
