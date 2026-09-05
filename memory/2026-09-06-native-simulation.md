# Native simulation foundation

## Scope and status

The first headless domain simulation now connects construction, production and household consumption to one province workforce and stockpile. This supersedes the clock-only core described in the interface-foundation milestone. The client remains a separate UI component preview with no simulation dependency. The bundled country and rates are synthetic fixtures, not empirical calibration or playable nation simulation.

## Architecture and public contracts

- `aggregate-world` owns typed definitions, identifiers and serializable state without ECS. `aggregate-scenario` loads JSON and validates definitions, references, UUIDs, quantities and aggregate ranges. `aggregate-economy` implements pure Rust allocation, production and consumption. `aggregate-simulation-core` owns the authoritative ECS World, scheduling, commands, reports, persistence and replay.
- Rules are native Rust functions/systems. The user explicitly deferred Rhai and a custom DSL. Serde is used only for preset/save data. Do not introduce a scripting adapter or generic numeric registry as presumed future infrastructure.
- `GoodId` and `FacilityDefinitionId` wrap strings for authored content kinds. `CountryId` and `ProvinceId` are stable string keys in current presets. `FacilityId` and `PopulationGroupId` wrap UUIDs for instances. Type wrappers prevent confusing different ID domains. The caller supplies a facility UUID before construction; completion retains it and replay reuses the recorded UUID. Nil and duplicate/conflicting UUIDs are rejected. ECS entity IDs never cross persistence boundaries.
- `Simulation::from_scenario` requires a validated scenario; there is no empty `Default` simulation. `step` returns `DayReport` and advances one full logical day. `SimulationClock::day()` replaces `tick()`; no calendar mapping exists. `execute` returns `CommandOutcome`. `snapshot` returns a portable, ID-sorted state.
- Explicit daily phases are BeginDay, LaborAllocation, Production, Construction, Consumption and Commit. Only stockpile/progress proposals are staged each day. Any planning/arithmetic failure leaves authoritative state and the day unchanged. Commands execute serially between complete days.
- Workers are shared proportionally between completed facilities and construction in each province. Largest remainders allocate whole workers; persistent UUID order breaks ties. Production processes lower `production_priority` first, then UUID, to reserve inputs. This priority does not affect workforce allocation. Later facilities may use earlier facilities' outputs in the same day. Workers idle due to insufficient inputs are not reassigned within the day.
- Construction consumes its entire goods cost when accepted and then consumes allocated worker-days. Completion produces a level-one facility which first operates the following day. Commands validate proposed state before publishing costs/projects, including capacity reserved for all pending facilities and mixed completion labor demands. Currently this command preflight clones a snapshot and runs shared validation; large-world command performance has not been measured.
- Quantities are checked `u64`; proportional products use `u128`. Goods have abstract scenario units and production uses whole worker-day batches. Population and workforce counts remain fixed. Shortages produce typed facts, not invented population effects. No prices, wages, treasury, trade, transport, migration, fertility, education or military model exists yet.
- `DayReport` contains province/facility results, goods-flow amounts with typed causes, and construction/shortage events. Event facts carry domain IDs rather than localized prose. News presentation is not wired to the UI.

## Persistence and example data

- Scenario and save schema versions are both 1. Native ruleset version is `aggregate-native-economy/1`; bump it when native rules/order change. There are no migrations yet.
- Saves embed the original scenario/definitions, current snapshot and full accepted command log. Loading validates versions, state and log sequence/day order without replaying all history. An edited snapshot is not authenticated against its log. Explicit replay uses the original scenario and the regular command path.
- BLAKE3 state hashes cover ID-sorted snapshots and ordered maps, excluding definitions, rules, command history and reports. Use them within the same scenario/ruleset; they are not save authentication or a cross-platform determinism guarantee.
- `scenarios/foundation.json` contains one country, two provinces, 100 residents/50 workers, grain/timber/tools, and farms/logging/tool production. `foundation.commands.json` starts a farm at day zero. Fixture IDs are fixed UUIDs so runs can be reproduced.
- `cargo run -p xtask -- headless` runs the embedded fixture for ten days. The direct example accepts `[scenario.json] [commands.json] [days=10] [save.json]`; see README. Supplying only a custom scenario runs without commands. An empty commands array can be used to skip construction. The example compares its final state against save/load and command replay and optionally writes a save file.

## Validation

- `cargo run -p xtask -- check` passed: workspace formatting, Clippy across all targets with warnings denied, and 62 tests (17 economy, 20 scenario, 14 core integration, 2 localization and 9 UI).
- Core integration tests verify goods-flow accounting, shared workforce, construction costs/timing, rejection without mutation, capacity reservations, insertion-order independence, failed-day atomicity, shortage facts and save/resume/replay agreement. Scenario tests include future and mixed construction-completion capacity validation.
- `cargo build --workspace --locked` passed on Windows.
- `cargo run -p xtask -- headless` passed for ten days. In the north province, day-one work split into 13 production and 7 construction workers; the farm completed on day four and first produced on day five. Final save/load and replay hashes matched.
- The README's explicit file-input example passed for 30 days and wrote `target/foundation-save.json`; save/load and replay hashes matched. The file is a local generated artifact, not versioned content.
- No new native UI session, release build, remote CI, Linux/macOS build or performance benchmark was run. The simulation is not yet integrated into the preview UI. No push or deployment was performed.

## Remaining work

Connect a management screen to actual snapshots, validated construction commands and completed reports. Add localized result explanations and a news feed using typed event facts. Add subsequent domains only with concrete shared resource contracts and explicit timing. Large-world performance, cross-platform/release validation, remote CI, and new native UI acceptance have not been performed for this milestone. Existing interface limitations remain documented in the previous milestone.

## References

- [Architecture](../docs/ARCHITECTURE.md)
- [Run and validation commands](../README.md)
- [Previous interface milestone](2026-09-06-interface-foundation.md)
