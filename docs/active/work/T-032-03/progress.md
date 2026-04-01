# T-032-03 Progress: scan-to-viewer-pipeline

## Completed

### Step 1: Coordinate fix in to_glb() ✓
- Modified `crates/pt-scan/src/export.rs`: positions `[x, y, z]` → `[x, z, -y]`, normals same transform
- Transforms min/max bounds to match
- All 25 existing unit tests pass unchanged (metadata uses raw coordinates, not GLB)

### Step 2: Update existing export tests ✓
- No changes needed — existing tests verify metadata (raw coords) and GLB structure (magic/version/JSON), not position values

### Step 3: Add integration tests ✓
- `test_terrain_glb_is_y_up`: verifies GLB position accessor has Y ≈ 0 for ground, X/Z span horizontal
- `test_terrain_glb_has_vertex_colors`: verifies COLOR_0 accessor is VEC4 UNSIGNED_BYTE normalized
- Both pass. Total: 10 integration tests (was 8)

### Step 4: Add justfile recipe ✓
- `just scan-to-viewer <path>` chains process-scan, then prints GLB path + instructions for serving and loading in viewer

### Step 5: Quality gate ✓
- `just check` passes (fmt + lint + test + scenarios)
- Scenario dashboard: 83.5 min (unchanged from baseline, no regressions)

### Incidental fixes
- Fixed pre-existing clippy warnings in `cluster.rs` exposed by `cargo fmt` reformatting
- Fixed formatting issues in `cluster.rs` (pre-existing, not introduced by this ticket)

## Deviations from Plan

None. All steps executed as planned.
