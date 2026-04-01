# T-032-03 Structure: scan-to-viewer-pipeline

## Files Modified

### crates/pt-scan/src/export.rs
- **to_glb()**: Swap coordinate system from Z-up to Y-up in position and normal arrays
  - Positions: `[x, y, z]` → `[x, z, -y]` (Y becomes vertical, negate old Y for right-handedness)
  - Normals: same transform `[nx, ny, nz]` → `[nx, nz, -ny]`
  - Min/max bounds: same transform for accessor min/max
- Update existing tests to expect Y-up coordinates

### justfile
- Add `scan-to-viewer` recipe:
  - Calls `just process-scan <path>` to produce GLB
  - Prints the GLB file path
  - Prints instructions for serving locally and loading in viewer

### crates/pt-scan/tests/integration.rs
- Add integration test: `terrain_glb_is_y_up` — synthetic cloud → generate_terrain() → verify Y-up in GLB positions
- Add integration test: `terrain_glb_has_vertex_colors` — verify COLOR_0 accessor exists in JSON chunk

## Files NOT Modified

### apps/viewer/ (no changes)
- Already loads arbitrary glTF via loadScene postMessage
- Already handles vertex colors (Bevy's StandardMaterial)
- Camera, lighting, picking all work with any valid glTF

### crates/pt-scene/ (no changes)
- Zone scenes remain separate from terrain scenes
- Compositing terrain + zones is T-033+ scope

### crates/pt-scan/examples/process_sample.rs (no changes)
- Already produces `{stem}-terrain.glb` — coordinate fix in to_glb() propagates automatically

## Module Boundaries

```
pt-scan::export::to_glb()    ← coordinate fix here (internal, no API change)
     ↓
  TerrainOutput.mesh_glb     ← GLB bytes are now Y-up (consumer-transparent)
     ↓
  process_sample example      ← writes GLB to disk (unchanged)
     ↓
  justfile scan-to-viewer     ← orchestrates + prints instructions (new)
     ↓
  Bevy viewer                 ← loads GLB via URL (unchanged)
```

## Change Ordering

1. Coordinate fix in `to_glb()` (export.rs) — standalone, testable
2. Update existing tests in export.rs to match new coordinate system
3. Add new integration tests (integration.rs) — verify Y-up + COLOR_0
4. Add justfile recipe — depends on coordinate fix being correct
5. Manual verification with real scan data

## Public Interface Changes

None. `generate_terrain()` returns the same `TerrainOutput` type. The GLB bytes are now glTF-spec-compliant (Y-up instead of Z-up). This is a bug fix, not an API change.
