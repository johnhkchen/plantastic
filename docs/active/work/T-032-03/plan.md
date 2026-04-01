# T-032-03 Plan: scan-to-viewer-pipeline

## Step 1: Coordinate fix in to_glb()

**File**: `crates/pt-scan/src/export.rs`

In `to_glb()`, transform positions and normals from Z-up to Y-up:
- Position `[x, y, z]` → `[x, z, -y]`
- Normal `[nx, ny, nz]` → `[nx, nz, -ny]`
- Update min/max bounds computation to use transformed coordinates

**Verification**: existing tests still pass after updating expected values.

## Step 2: Update existing export tests

**File**: `crates/pt-scan/src/export.rs` (tests module)

- `test_metadata_consistency`: elevation range assertions change (now Y-axis)
- `test_glb_json_chunk_parseable`: still valid (structure unchanged)
- `test_glb_magic_and_version`: still valid (header unchanged)

Run: `cargo test -p pt-scan`

## Step 3: Add integration tests

**File**: `crates/pt-scan/tests/integration.rs`

Test 1: `terrain_glb_is_y_up`
- Create synthetic point cloud (ground at z≈0, obstacles at z>0.3)
- Call `generate_terrain()`
- Parse GLB JSON chunk
- Verify position accessor: max Y > 0 (vertical), min/max X and Z span the horizontal plane
- Verify node name is "terrain"

Test 2: `terrain_glb_has_vertex_colors`
- Same synthetic cloud with colors
- Generate terrain GLB
- Parse JSON chunk
- Verify accessor for COLOR_0 exists with componentType=5121 (UNSIGNED_BYTE) and type="VEC4"

Run: `cargo test -p pt-scan`

## Step 4: Add justfile recipe

**File**: `justfile`

```
# Process a PLY scan and prepare for 3D viewer
scan-to-viewer path="assets/scans/samples/Scan at 09.23.ply":
    just process-scan "{{path}}"
    @echo ""
    @echo "── Viewer Instructions ──"
    @echo "1. Serve the output directory:"
    @echo "   python3 -m http.server 8080 -d $(dirname '{{path}}')"
    @echo ""
    @echo "2. In the viewer, send loadScene with the terrain GLB URL:"
    @echo "   { \"type\": \"loadScene\", \"url\": \"http://localhost:8080/$(basename '{{path}}' .ply)-terrain.glb\" }"
```

## Step 5: Run quality gate

Run `just check` (fmt + lint + test + scenarios).

Verify:
- All tests pass
- No clippy warnings
- Scenario dashboard: no regressions (83.5 min baseline)
- S.1.1 still passes

## Step 6: Manual verification (informational)

With the Powell & Market scan:
1. `just scan-to-viewer`
2. Serve GLB locally
3. Load in Bevy viewer
4. Confirm: terrain mesh renders right-side-up, brick paths visible, orbit camera works

This step is for the review.md report, not automated.
