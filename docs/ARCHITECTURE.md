# Aggregate architecture

## Implemented boundaries

The workspace follows explicit responsibility boundaries. Project crates use the `aggregate-` prefix; internal types describe their actual role (`UiButton`, `TooltipContent`, `SimulationClock`). The standard developer executable remains `xtask`.

Current dependencies:

```text
aggregate-client -> aggregate-ui -> Bevy
                 -> aggregate-localization -> Fluent

aggregate-simulation-core -> bevy_ecs
                          -> aggregate-world
                          -> aggregate-scenario -> aggregate-world
                          -> aggregate-economy  -> aggregate-world

xtask -> Cargo commands
```

UI components receive localized strings from the client. The client is still a component preview and has no simulation dependency. The simulation owns one authoritative World and advances one logical day per explicit `step` call; it does not import rendering, windows, fonts or client code. A logical day is independent of real time and has no historical calendar mapping yet.

`aggregate-world` owns definitions, typed identifiers and serializable state. `aggregate-scenario` owns JSON loading and validation. `aggregate-economy` implements pure allocation and recipe calculations without mutating state. `aggregate-simulation-core` composes those calculations into ECS phases and owns command validation, committed state, reports, persistence and replay. No domain crate needs to depend on another domain's implementation to share the world contracts.

## UI approach

Use Bevy's native layout, text, input, asset and rendering facilities. Project components provide a cohesive game interface above those facilities. Do not introduce egui or reimplement a graphics engine. Victoria 3 is a reference for information density, panel hierarchy, map framing and visual ambition; existing game assets are not source material for distribution.

The first interface is a **component preview**, deliberately without fabricated treasury, population or simulation outcomes. It verifies the reusable presentation foundation before domain integration. It does not claim Victoria 3-level art completeness.

`aggregate-ui` owns theme tokens, bundled fonts, primitive construction, button state, focus and tooltip presentation. `aggregate-client/screens` owns page composition. `aggregate-client/interaction` maps UI activations to interface actions. Only screen/tab/language changes rebuild a screen; state labels and selection update in place. Scroll is confined to the pointed-at pane.

Fonts are loaded from compile-time bytes into native Bevy Font assets. This small fixed font set does not require a third-party embedding plugin. Future external scenario/mod assets must remain separate from the embedded interface baseline, with an explicit override policy.

Tooltips have an explicit waiting/visible/locked state stack. The defaults are a 250 ms display delay, an additional 850 ms dwell to lock, and 180 ms departure grace for crossing into the panel. A native segmented circular indicator displays locking progress. Click or keyboard activation locks immediately. Locked parents expose child explanation buttons; Escape removes the deepest level, and outside click or removal of the source clears the relevant stack. Timings are a `TooltipSettings` resource and use real time, independent of simulation ticks. Inline rich text links are not implemented; current nested links are separate native buttons. Placement uses the source rectangle and a viewport clamp with estimated height; very dense stacks and resizing while locked still need visual refinement.

Motion uses Bevy easing curves with per-instance state: interruptible hover transitions, a small elastic release on button labels, panel entrance and smooth scrolling. Hitboxes stay stable. Reduced motion finishes visual transitions immediately without disabling functional tooltip timers. The user's local osu!lazer reference was inspected for interaction timing and transition composition (RoundedButton, FormButton, SwitchButton and OsuButton); no reference code or assets were copied.

Large-list virtualization, text editing/IME acceptance, screen-reader acceptance and game data bindings are future work. Settings are session-only. The reference layout is 1280x800 with automatic fitting and a user scale multiplier.

## Localization

Fluent catalogs live in `locales/{ko-KR,en-US}/interface.ftl`. The localization crate has no rendering dependency. Missing selected-language messages fall back to English; missing keys and formatting errors are explicit errors. The client logs those errors and displays a diagnostic key rather than silently blank text.

Bundled catalogs currently contain argument-free UI messages. Tests check matching keys and successful formatting in both languages. Simulation events already store typed facts and domain IDs; their localized presentation is not connected yet. When parameterized news and terms are added, extend validation to variable contracts, plural/select variants and references. Font fallback beyond the bundled character coverage remains to be designed.

## Native simulation model

Mechanisms are Rust functions and systems. Serde maps JSON presets and saves to typed Rust data; it is not a rule language. Rhai, other scripting engines and a custom DSL are deferred. Content parameters can change without changing the algorithms, while a new mechanism currently requires Rust implementation and a build.

Definitions describe goods, per-level staffing, per-worker-day input/output recipes and construction costs. State describes countries, provinces, population groups, completed facilities and active construction projects. Goods are counted in integer scenario units; population, workers, stock and work quantities use checked `u64` arithmetic. Production operates in whole worker-day batches. The fixture values in `scenarios/foundation.json` are synthetic and have no empirical calibration.

`GoodId` and `FacilityDefinitionId` are typed string keys for authored content. `CountryId` and `ProvinceId` are also string keys in the current preset model. `FacilityId` and `PopulationGroupId` wrap UUIDs for instances. The caller assigns a new facility UUID before submitting a command, and replay reuses that recorded value. The core does not generate random identities during a day. Nil, duplicate and conflicting instance IDs are rejected; ECS entity IDs are never persisted or used as domain identities.

Scenario loading rejects unsupported versions, unknown fields and references, invalid quantities and unsupported aggregate ranges, with field paths and source paths when loaded from a file. Scenarios begin at day zero without pending construction; commands create projects. Saved snapshots can contain validated projects in progress.

## Commands and daily execution

`StartConstruction` runs between complete days. It checks province ownership, the definition, the requested workforce, instance identity and the proposed state's invariants before publication. Required goods are reserved up front. Rejected commands leave stockpiles, projects and command history unchanged. The successful command records its day, sequence and typed input, and returns construction-cost flows and a construction-started event.

Each day follows an explicit schedule:

1. **Begin day:** create proposed province stockpiles and an empty report.
2. **Labor allocation:** aggregate each province's workforce and share it across operating facilities and construction requests. Shortages use proportional allocation with largest remainders; UUID order breaks ties. No worker is assigned twice.
3. **Production:** process lower `production_priority` first, then facility UUID. Each facility consumes available inputs and produces only with its active workers. Outputs are available to later facilities and household consumption that day. Priority affects input order, not workforce allocation. Assigned workers blocked by missing inputs are not reassigned during that day.
4. **Construction:** spend allocated worker-days and report completions. Project priority is inherited by the completed facility and does not prioritize construction labor.
5. **Consumption:** satisfy household staple demand from the remaining stockpile and report unmet demand.
6. **Commit:** publish stockpiles, project progress, completed facilities and the new day only after all phases succeed. A newly completed facility first produces on the following day.

Before commit, stock and progress changes live in a temporary day plan. A checked arithmetic or planning failure discards those proposals without changing authoritative state or advancing time. Stable IDs and ordered maps determine results rather than ECS insertion order. No randomness is currently involved.

Reports contain typed production, construction and consumption facts, with goods-flow causes and referenced domain IDs. Food shortages do not automatically change population, mortality or migration. Population groups currently supply fixed population and workforce counts. Prices, wages, government budgets, trade and transport are not implemented.

## Saves and replay

`save_json` includes the original scenario and definitions, the current snapshot, the full accepted command log, a save schema version and a native ruleset version. `from_save_json` validates versions, state and command sequence/day ordering. Unsupported versions are rejected; migrations are not implemented. Loading validates the snapshot directly rather than replaying its entire history, so it does not prove that an edited snapshot was produced by the supplied log.

Replay starts from the initial scenario, advances the same daily schedule, and submits recorded commands through normal validation at their recorded days. UUIDs and consecutive command sequences remain stable. Replay is defined for the current ruleset; changing native rule semantics or execution order requires a ruleset-version change.

`state_hash` hashes the normalized snapshot with BLAKE3 after sorting state vectors by persistent ID and serializing ordered maps. It excludes definitions, rules, command history, reports and ECS layout. Compare hashes within the same scenario and ruleset; the hash is a state-comparison aid, not save authentication. The headless example verifies its final state against both save/load and command replay.

## Next integration

Connect a management screen to validated commands, snapshots and completed reports so changes can be inspected with their causes. UI state must not become a second simulation authority. Education, logistics, military, politics and diplomacy can then add concrete mechanisms against shared contracts; do not create empty domain crates or cyclic Cargo dependencies to represent reciprocal effects.

Any eventual rule language will need explicit scope, units, input time, permitted effects and diagnostics. Rust traits do not discover numeric dependencies, and feedback loops require deliberate temporal or solver semantics. Current code uses explicit typed mechanisms rather than a generic string-to-number store.

## Validation boundaries

Compilation, automated tests, native runtime inspection, performance measurement and remote CI are distinct evidence. Simulation tests cover resource accounting, deterministic allocation/order, command rejection, failure atomicity, construction timing and save/replay behavior. They do not establish economic realism, large-world performance, cross-platform determinism or playable simulation functionality. The existing UI preview has not yet exercised these domain flows.
