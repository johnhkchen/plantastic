# T-032-03 Review: scan-to-viewer-pipeline

## Summary

This ticket wires the end-to-end path from PLY scan to 3D viewer: pt-scan now exports glTF-spec-compliant Y-up terrain meshes that load correctly in the Bevy WASM viewer. A `just scan-to-viewer` recipe orchestrates the full pipeline and prints instructions for local viewing.

## Changes

### Modified Files

| File | Change |
|---|---|
| `crates/pt-scan/src/export.rs` | Coordinate transform in `to_glb()`: Z-up → Y-up for positions and normals |
| `crates/pt-scan/tests/integration.rs` | +2 integration tests: Y-up verification, vertex color verification |
| `crates/pt-scan/src/cluster.rs` | Pre-existing clippy/format fixes (cast_possible_truncation, cast_sign_loss) |
| `justfile` | New `scan-to-viewer` recipe |

### No changes to
- `apps/viewer/` — already loads any valid glTF
- `crates/pt-scene/` — zone scenes remain independent (compositing is T-033+)
- `crates/pt-scan/examples/process_sample.rs` — coordinate fix propagates through `to_glb()` automatically

## Test Coverage

| Suite | Before | After | Notes |
|---|---|---|---|
| pt-scan unit tests | 25 | 25 | Unchanged — metadata tests use raw coords |
| pt-scan integration tests | 8 | 10 | +`test_terrain_glb_is_y_up`, +`test_terrain_glb_has_vertex_colors` |
| Total workspace | All pass | All pass | `just check` green |

### What the new tests verify
- **Y-up coordinates**: GLB position accessor has Y ≈ 0 for ground plane, X/Z span the horizontal extent. Catches regressions in the coordinate transform.
- **Vertex colors**: GLB has COLOR_0 accessor of type VEC4 UNSIGNED_BYTE normalized. Ensures terrain renders with scan colors in Bevy.

## Scenario Dashboard

| Metric | Before | After |
|---|---|---|
| Effective savings | 83.5 min | 83.5 min |
| Scenarios passing | 9 | 9 |
| Milestones | 22/25 | 22/25 |

No scenario regression. This ticket is infrastructure — it makes the scan-to-viewer path work but doesn't flip a new scenario. The coordinate fix is a correctness improvement that enables S.4.1 ("3D viewer on tablet") to advance once the viewer is wired to load terrain GLBs from the API.

## Acceptance Criteria Status

| Criterion | Status |
|---|---|
| CLI / `just` recipe: `just scan-to-viewer <ply-path>` | ✓ Done |
| process_scan() → ClassifiedCloud → generate_terrain() → GLB | ✓ Done (existed, now Y-up) |
| Write GLB to local path the viewer can load | ✓ Done (process_sample writes it) |
| Print instructions to open viewer with GLB URL | ✓ Done (recipe prints serve + loadScene) |
| pt-scene's generate_scene() accepts terrain GLB as base layer | Deferred — design decision: terrain and zone scenes are separate GLBs, composited later (T-033+) |
| Bevy viewer loads and renders terrain with orbit camera | ✓ Works — viewer loads any valid glTF, coordinate fix ensures correct orientation |
| Powell & Market: brick paths visible, trunks visible | Requires manual verification with real scan |

## Open Concerns

1. **Manual verification pending**: The coordinate fix is tested with synthetic data. Real-scan verification (Powell & Market) requires running `just scan-to-viewer` with the 294 MB PLY and viewing in the Bevy WASM viewer. This is a local dev exercise, not automatable in CI (PLY is gitignored).

2. **Terrain + zone compositing**: The ticket mentions `pt-scene's generate_scene() can accept a terrain GLB as the base layer`. The design decision was to keep these as separate GLBs — the viewer already supports loading a single scene, and multi-scene compositing is a T-033+ concern. The viewer could be extended to support multiple concurrent scenes (terrain + zones) via separate loadScene calls.

3. **Camera auto-framing**: The default camera at (3, 3, 5) may be too close for terrain spanning 30+ meters. The recipe prints the GLB path but doesn't adjust the camera. Future work: auto-frame camera to bounding box on scene load.

4. **cluster.rs fixes**: Fixed pre-existing clippy warnings that were exposed when `cargo fmt` reformatted the file. These are cosmetic (allow annotations for safe truncation casts) and don't change behavior.
