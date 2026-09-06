# Aggregate

An extensible nation simulation with a native Bevy interface.

The first headless simulation connects a shared workforce, production, consumption and construction through explicit Rust rules. It loads a validated scenario, records commands, and supports save/resume and deterministic command replay. The bundled scenario is a small synthetic fixture, not a calibrated economic model or a playable nation simulation.

The native management screen runs that simulation: select a province, inspect stockpiles and workforce, construct buildings, and advance time. Construction progress and national dispatches reflect actual commands and completed days. The component preview remains available separately. Both use Korean/English localization, native controls and timed tooltips. The separate World Map route renders real province geography and terrain; the original menu backdrop remains decorative.

## Run

Rust 1.98 or newer is required. From the repository root:

```sh
cargo run -p aggregate-client
```

Choose **국가 관리 / Manage the country** from the main menu. The bundled scenario starts paused at day zero. **하루 진행 / Advance 1 day** runs one day and pauses; **진행 / Run** advances approximately one day per real second. Construction uses the selected province's materials and competes with existing production for workers. The sidebar tracks remaining work and the right panel records starts, completions and food shortages.

Leaving management pauses time. Returning, selecting another province, changing language or opening settings preserves the session. The client currently starts the bundled scenario only; preset selection and save/load controls are not yet available. Closing the application discards the in-memory session.

Pretendard and the initial Fluent catalogs are included in the executable. Fonts need no system installation. The map and textured UI load project assets from `assets/`.

- `Tab` / `Shift+Tab`: move between enabled buttons.
- `Enter` / `Space`: activate the focused button.
- Hover an explanation: wait for the circular indicator to fill and lock; click or press Enter to lock immediately. A locked explanation can open child explanations.
- `Escape`: close the deepest explanation, or return to the main menu when none is open. Clicking outside closes the explanation stack.
- `F12`: save a screenshot under `target/screenshots/` in the build workspace (development utility).
- Mouse wheel: smoothly scroll the frontmost content region under the pointer.

Language, interface scale and reduced motion are session settings. They are not persisted yet. The scale control offers 80–140%; smaller windows also fit the reference layout automatically.

## Headless simulation

Run the bundled scenario and construction command for ten logical days:

```sh
cargo run -p aggregate-simulation-core --example headless --locked
```

Provide scenario and command files, a final day, and an optional save destination:

```sh
cargo run -p aggregate-simulation-core --example headless --locked -- scenarios/foundation.json scenarios/foundation.commands.json 30 target/foundation-save.json
```

Replace the paths with your own files. [The scenario](scenarios/foundation.json) defines goods, facilities, population groups and province stockpiles. [The command file](scenarios/foundation.commands.json) starts construction with a caller-assigned facility UUID. Commands are ordered by day and consecutive sequence; use an empty JSON array for a run without commands. The example prints workforce and food-shortfall reports and compares its final state with a save/load and command replay.

Facilities and construction share each province's workforce. Construction reserves goods immediately, consumes worker-days, and begins production on the day after completion. Production and households draw from actual province stockpiles; food shortfalls are reported without inventing deaths or migration. There are no prices, wages, finance, trade or demographic transitions yet.

Rules are implemented directly in Rust. Serde handles preset and save data only; scripting and a DSL are deferred.

## Workspace

| Crate | Responsibility |
| --- | --- |
| `aggregate-client` | App startup, management session, command dispatch, localized screens and decorative background |
| `aggregate-ui` | Native Bevy UI theme, fonts, panels, buttons, focus and tooltips |
| `aggregate-localization` | Fluent catalogs and locale selection; no Bevy dependency |
| `aggregate-world` | Typed definitions, persistent identities and serializable world state; no ECS dependency |
| `aggregate-scenario` | JSON loading, reference checks and scenario/snapshot validation |
| `aggregate-economy` | Pure Rust workforce allocation, production and consumption calculations |
| `aggregate-simulation-core` | Authoritative ECS World, daily phases, commands, reports, saves and replay; no renderer |
| `xtask` | Development checks and the headless example |

The client submits core commands and reads snapshots/reports; UI components do not own simulation state. Education, military, politics, diplomacy and user-authored rule execution remain future mechanisms.

See [architecture](docs/ARCHITECTURE.md) for dependency boundaries and planned integration, and [memory](memory/README.md) for verified milestones.

## Validate

```sh
cargo run -p xtask -- check
cargo run -p xtask -- headless
```

On Windows, the opt-in graphical acceptance test opens the real client, activates its management controls, and saves screenshots under `target/screenshots/`:

```sh
cargo test -p aggregate-client native_management_capture --locked -- --ignored --nocapture
```

`just run`, `just check`, `just headless` and `just fmt` are equivalent conveniences when Just is installed. CI is configured for workspace-wide checks. Linux builds need the platform development packages listed in the workflows; see [Bevy's Linux dependencies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md).

## Assets

The interface uses native Bevy rendering; egui is not used. Pretendard 1.3.9 is bundled with its [SIL OFL license](crates/aggregate-ui/assets/fonts/LICENSE.txt). This local working tree contains imported Victoria 3 map and interface resources under `assets/`; see [asset provenance and runtime contracts](docs/MAP_ASSETS.md). They are not original Aggregate artwork. Do not commit or publish assets without an explicit user request.

Choose **World Map** for real terrain, province picking, political coloring, horizontal wrapping and a Tab destination preview. This geography is not yet connected to the small economic fixture. The intended modern preset is **2026**; the current ownership import is historical and has not been relabeled as present-day data.

```sh
cargo test -p aggregate-client native_map_capture --locked -- --ignored --nocapture
cargo run -p aggregate-geography --example measure_map_loading --locked
cargo run -p xtask --locked -- validate-map-assets
```

See [map interface](docs/MAP_INTERFACE.md) for implemented controls and remaining features.
