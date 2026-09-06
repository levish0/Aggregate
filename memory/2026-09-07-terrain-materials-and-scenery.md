# Terrain materials and scenery — 2026-09-07

## Decisions and implementation

- Offline map outputs now live in `assets/gfx/map/derived`, replacing `compiled`. These are processed texture/geometry inputs, not precompiled GPU shader pipelines. Runtime WGSL pipelines still compile through Bevy/wgpu.
- `tools/build_terrain_materials.py` produces 46-layer, 1024-square BC3 diffuse/normal/material-property arrays with complete mip chains, preserving authored placement indices/weights. `tools/build_terrain_relief.py` derives 4096 x 1808 world normals and local horizon ambient visibility from the source 16-bit heightmap and terrain settings.
- Terrain rendering uses height-weighted material blending, packed source normal decoding, world relief, authored colormap soft-light blending, and material roughness/specular channels. This is not complete source-engine PBR or dynamic shadow parity.
- Height scale was reduced from the old 40 to 8 during calibration, then increased to **10** on the user's final request for slightly taller mountains (25 percent above the accepted 8). Relief was regenerated at 10. Runtime scenery samples the terrain triangles, so trees and roads follow the changed elevation. Terrain settings participate in the geography cache key.
- Zoomed terrain retains a faint owner-colored border rim around a thin boundary, while selected/hovered states retain white highlighting. Country fill remains minimal close up and grows at political overview distances.
- Water uses imported ambient normals and flowmap with animated wind/current UV phases, view-dependent reflection and sun highlights. Visual animation uses Bevy elapsed time, independent of paused simulation days. Rivers reuse animated water appearance; this is not hydrological simulation.
- Clouds use imported density/normal textures on a low plane at height 3.5. Shared `map_cloud_density.wgsl` combines two rotated/scaled drifting fields with weather coverage noise for cloud color and projected terrain shadows. Opacity is zero at camera distances <=35 and >=180, full at 80–110, smoothly faded between. Close zoom clears the clouds. These are transparent layered surfaces, not volumetric clouds.
- `tools/build_map_scenery.py` reads the imported spline network and forest placement transforms. Road paths receive two Chaikin smoothing passes. Derived binaries have magic/version/count headers and 16-byte records. Async map preparation builds 64-unit spatial chunks, sharing meshes across wrapped copies instead of creating an entity per tree. The local data prepared 438,709 land trees and 135,343 land road segments; roads follow subdivided terrain samples and use imported road texture.

## Provenance and constraints

- Terrain, water, cloud and road textures, heightmap, forest placements and road spline source come from the locally imported Victoria assets. Derived files are transformations of those sources. Rust/WGSL rendering and the simplified canopy geometry are Aggregate implementations. Renaming or processing does not change source provenance.
- Original tree models, city models and complete source-engine lighting are not connected. Imported historical road corridors are visual scenery, not a verified 2026 transport network or simulation infrastructure authority. Full Victoria visual parity is not claimed.
- User accepted the overall presentation and requested only modest mountain-height adjustment at the end. Preserve the narrowed cloud zoom range and subtle national border coloring.

## Validation

- Focused client/map-view tests passed (11 regular tests); map-view all-target Clippy with warnings denied passed. Broader client Clippy encountered unrelated existing aggregate-ui lints in scrollbar.rs (collapsible_if) and window.rs (type_complexity); those files were not changed.
- Native map acceptance passed after the final height change: 1 test, 12 captures, 50.92 seconds. This covers map selection, UI blocking, camera controls, wrap/overview, label invariance and terrain captures. Latest log: `target/terrain-native-height.log`. The final mountain/border capture was visually inspected at `target/screenshots/world-map-country-border-close.png`.
- Earlier same-camera captures verified changing water pixels across 30 ready frames; cloud/water shaders load and render. DDS headers/mips and scenery headers/counts were checked, and malformed spline tokens/truncation were rejected by focused parser checks.
- Client build passed before the final data-only height/relief update. No Rust source changed in that final adjustment; the native test loaded the updated assets. No new remote CI, long-run simulation benchmark or full graphics performance budget was established.
- No commit or push was performed by the agent.
