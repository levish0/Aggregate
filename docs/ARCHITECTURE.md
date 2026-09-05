# Aggregate architecture

## Implemented foundation

The workspace follows explicit responsibility boundaries. Project crates use the `aggregate-` prefix; internal types describe their actual role (`UiButton`, `TooltipContent`, `SimulationClock`). The standard developer executable remains `xtask`.

Current dependencies:

```text
aggregate-client -> aggregate-ui -> Bevy
                 -> aggregate-localization -> Fluent

aggregate-simulation-core -> bevy_ecs

xtask -> Cargo commands
```

UI components receive localized strings from the client. No UI component can mutate simulation state. The simulation owns a separate World and advances only through explicit calls; it does not import rendering, windows, fonts or client code. The current schedule only advances a logical clock. Domain time resolution is undecided.

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

Bundled catalogs currently contain argument-free UI messages. Tests check matching keys and successful formatting in both languages. When parameterized news and terms are added, extend validation to variable contracts, plural/select variants and references. Simulation events must ultimately store typed facts and domain IDs, not rendered localized sentences. Font fallback beyond the bundled character coverage remains to be designed.

## Planned simulation integration (not implemented)

Economy, education, population, logistics, military, politics and diplomacy will operate on one logical authoritative simulation World. Components, tables and graphs have explicit source owners. The client submits domain commands and reads completed query results; UI state is not a second editable authority.

Add domain crates when concrete mechanisms and contracts are implemented. Do not create empty crates for every future feature. Do not create cyclic Cargo dependencies to represent reciprocal social/economic effects. Exchange typed requests/results through domain-owned contracts and compose schedules above them.

Distinguish:

- Definition parameters and policies, which may be declared as data.
- Persisted state, changed by its owning mechanism.
- Derived metrics, calculated from authoritative inputs.
- Competing-resource allocation and graph/equilibrium solvers, implemented as explicit Rust algorithms.

Any future declarative rule format must define units, scope, input phase/time, allowed operations, diagnostics and provenance. Rust traits alone do not reveal numeric dependencies. Temporal feedback and same-step algebraic cycles require explicit modeling choices. A generic string-to-number store or unrestricted mutation DSL is not the foundation.

The user operates through construction, policies and automation with optional detailed intervention. Scenario editing may expose finer state but must validate and record changes. Shared action validation must serve player UI, AI and automation.

## Next integration milestone

Define stable IDs, a validated scenario schema, source ownership and command/query contracts before wiring a management screen. Select an actual model and test assumptions before implementing university investment or mobilization. Verify conservation, invalid command rejection, save/resume and replay at that stage. Avoid labeling fixture values as simulated results.

## Validation boundaries

Compilation, automated tests, native runtime inspection, performance measurement and remote CI are distinct evidence. Current tests establish the minimal clock and UI/localization behavior only. They do not establish economic realism, large-world performance, cross-platform determinism or playable simulation functionality.
