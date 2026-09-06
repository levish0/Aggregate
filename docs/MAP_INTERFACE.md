# Map interface

Reference: [Victoria 3 Keyboard shortcuts](https://vic3.paradoxwikis.com/Keyboard_shortcuts), read in the browser on 2026-09-06. The web fetch returned 401, but the browser exposed the actual page. Direct game observation and user screenshots supplement the table: the wiki names Tab overview without describing its full preview behavior.

| Input | Current Aggregate behavior |
| --- | --- |
| WASD / arrows | Camera-relative movement |
| Wheel | Smooth zoom |
| Middle drag | Pan along the map |
| Right drag | Change pitch and yaw; release preserves pitch and smoothly restores the pre-drag heading |
| Hold Tab | World overview; open inspection panels remain |
| Move pointer during overview | White projected footprint of the prospective zoomed viewport |
| Click during overview | Commit destination without selecting a province |
| Release Tab | Restore previous zoom/orientation at committed destination; original destination if nothing clicked |
| Normal left click | Ray-pick geographic province |
| Alt+2 | Political coloring |
| Esc | Dismiss tooltip/select, leave overview, close inspection, then return to menu |
| Ctrl+Tab | Focus navigation while plain Tab belongs to the overview |

The outline uses the perspective return camera, producing a trapezoid where appropriate. Longitude wraps across shared render copies; north/south edges meet a procedural tabletop and paper margin. The full original table and paper artwork is not implemented.

`Q` is Ledger in the reference, not map modes. F1-F11 panels, ledger, location finder, capital centering, complete lenses and speed controls await their actual domain features. They are not bound to unrelated placeholders. Right click was observed opening a country/state/strategic-region menu; that menu and context-sensitive country/state selection remain pending. The user verified middle-button panning and right-button rotation in the installed game on 2026-09-06. Pitch persists; heading returns after release. Drag capture begins over the map and remains active across UI panels until release. Reduced-motion mode restores heading immediately. The wiki and installed 1.13 profile differ: Shift+F1 opens Companies in the installed profile, so Aggregate does not assign it to the outliner.

## Native composition

`aggregate-ui` owns flex composition, scroll regions, select root/trigger/content/items, timed nested tooltips and retargetable motion. Client screens own layout and actions. `UiButton` owns interaction; `PanelSkin`, `TextureFrame` and `SurfaceMaterial` own rendering. DDS images load directly through Bevy. Layered surfaces combine base, grain, shading and pattern; sliced images preserve decorative borders. Primary buttons use imported gold frames. Hover/press reuse existing motion, including reduced-motion mode. Decorative layers ignore pointer picking. Pretendard and Fluent remain font/translation boundaries.

This ports a subset of original GUI overlay, masking, atlas and density behavior. It is not yet a pixel-identical recreation of every screen. Full headers, icon rails, scrollbar thumbs and domain panel content remain. Real geography and the synthetic economic fixture are separate routes.

## Verification

Native captures exercise Bevy input in a real rendered window, not OS automation:

```sh
cargo test -p aggregate-client native_map_capture --locked -- --ignored --nocapture
cargo test -p aggregate-client native_management_capture --locked -- --ignored --nocapture
```

Screenshots are written under `target/screenshots/` for terrain, political view, select menu, overview and a smaller English window.
