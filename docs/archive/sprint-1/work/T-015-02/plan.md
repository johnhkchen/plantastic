# T-015-02 Plan: Mesh Generation & Export

## Step 1: Add dependencies to Cargo.toml

Add `delaunator`, `meshopt`, `image` (png feature only), and `serde_json` to `crates/pt-scan/Cargo.toml`.

**Verify**: `cargo check -p pt-scan` compiles.

## Step 2: Add error variants

Extend `ScanError` in `error.rs` with `MeshGeneration(String)` and `ExportError(String)`.

**Verify**: existing tests still pass.

## Step 3: Implement `mesh.rs` — triangulation

Create `crates/pt-scan/src/mesh.rs` with:
- `TerrainMesh` struct (positions, normals, colors, indices)
- `MeshConfig` struct (target_triangles with default 50,000)
- `triangulate(points: &[Point]) -> Result<TerrainMesh, ScanError>`:
  - Extract XY coords as `Vec<delaunator::Point>`
  - Call `delaunator::triangulate()`
  - Map triangle indices back to original positions + colors
  - Compute normals via `compute_normals()`
- `compute_normals()`: per-face cross product, accumulate at vertices, normalize

**Verify**: unit test — triangulate a known 4-point square → 2 triangles, normals point up.

## Step 4: Implement `mesh.rs` — decimation

Add `decimate(mesh: &TerrainMesh, target: usize) -> TerrainMesh`:
- Convert positions to flat `Vec<f32>` for meshopt
- Call `meshopt::simplify()` with target index count = target * 3
- Build new position/color/normal arrays from surviving vertices
- Recompute normals

**Verify**: unit test — triangulate 1000 points, decimate to 100 triangles, verify output has ≤ 100 triangles and valid indices.

## Step 5: Implement `export.rs` — glTF binary export

Create `crates/pt-scan/src/export.rs` with:
- `TerrainMetadata` struct (bbox, elevation_range, counts, processing_time_ms)
- `ExportConfig` struct (mesh config, pixels_per_foot, canopy_overlay)
- `to_glb(mesh: &TerrainMesh) -> Vec<u8>`:
  - Pack positions, normals, colors, indices into binary buffer
  - Build JSON with serde: asset, buffer, bufferViews (4), accessors (4), mesh, node, scene
  - Assemble GLB: header + JSON chunk + binary chunk with correct alignment

**Verify**: unit test — generate GLB from a simple mesh, verify magic bytes (0x46546C67), version (2), JSON is parseable, buffer sizes match.

## Step 6: Implement `export.rs` — PNG plan view

Add `to_plan_view_png()`:
- Compute image dimensions from bbox + pixels_per_foot
- Create `ImageBuffer<Rgb<u8>>`
- For each mesh triangle: project XY to pixel coords, rasterize with elevation-based coloring
- Optional: render obstacle points as darker overlay
- Encode to PNG in memory via `image::codecs::png::PngEncoder`

Add `rasterize_triangle()`:
- Bounding box clip, barycentric coordinate test for each pixel
- Interpolate color from vertex colors

**Verify**: unit test — render a known triangle, verify image dimensions and non-empty pixel content.

## Step 7: Implement `generate_terrain()` — pipeline orchestrator

Add the public entry point:
```rust
pub fn generate_terrain(cloud: &PointCloud, config: &ExportConfig) -> Result<TerrainOutput, ScanError>
```
- Start timer
- `triangulate(&cloud.ground)`
- `decimate(&mesh, config.mesh.target_triangles)`
- `to_glb(&decimated)`
- `to_plan_view_png(&decimated, &cloud.obstacles, &cloud.metadata.bbox, config)`
- Build `TerrainMetadata`
- Return `TerrainOutput`

**Verify**: integration test — synthetic PLY → process_scan → generate_terrain → verify all three outputs.

## Step 8: Wire up module exports in `lib.rs`

Add `pub mod mesh; pub mod export;` and re-exports.

**Verify**: `cargo check -p pt-scan` with new public API.

## Step 9: Integration tests

Add to `tests/integration.rs`:
1. `test_terrain_generation_pipeline`: full PLY → mesh → export, verify outputs are non-empty, metadata consistent
2. `test_glb_format_validity`: verify GLB structure (magic, version, chunks, JSON fields)
3. `test_plan_view_dimensions`: verify PNG dimensions match expected from bbox and config
4. `test_metadata_consistency`: verify metadata matches actual mesh properties

All tests use `timed()` wrapper. Expected values computed independently.

## Step 10: Update S.1.1 scenario test

In `tests/scenarios/src/suites/site_assessment.rs`:
- After existing `process_scan` validation, add `generate_terrain()` call
- Verify GLB magic bytes, PNG magic bytes, metadata sanity
- Keep at `OneStar`

## Step 11: Update milestone in progress.rs

Update the "pt-scan: PLY parsing + mesh generation" milestone:
- `delivered_by: Some("T-015-02")`
- Expand note to describe mesh gen + export capabilities

## Step 12: Quality gate

- `just fmt`
- `just lint`
- `just test`
- `just scenarios`

Verify S.1.1 still passes at OneStar. No regressions in other scenarios.

## Testing Strategy

| Test | Type | What it verifies |
|------|------|-----------------|
| `triangulate` unit test | Unit | 4-point square → 2 triangles, normals up |
| `decimate` unit test | Unit | 1000 points → ≤100 triangles |
| `to_glb` unit test | Unit | GLB magic, version, JSON structure |
| `plan_view` unit test | Unit | PNG dimensions, non-empty pixels |
| `test_terrain_generation_pipeline` | Integration | Full PLY → 3 output artifacts |
| `test_glb_format_validity` | Integration | GLB binary format correctness |
| `test_plan_view_dimensions` | Integration | PNG dimensions from bbox |
| `test_metadata_consistency` | Integration | Metadata matches mesh properties |
| S.1.1 scenario | Scenario | End-to-end with mesh + export |
