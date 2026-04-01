# T-015-02 Progress: Mesh Generation & Export

## Completed

### Step 1: Dependencies
- Added `delaunator = "1"`, `meshopt = "0.4"`, `image` (png), `serde_json` to pt-scan Cargo.toml.
- `cargo check` passes.

### Step 2: Error variants
- Added `MeshGeneration(String)` and `ExportError(String)` to `ScanError`.
- Existing tests unaffected.

### Step 3: Triangulation (mesh.rs)
- `TerrainMesh` struct: positions, normals, colors, indices.
- `MeshConfig` with `target_triangles` (default 50K).
- `triangulate()`: XY projection → delaunator → 3D positions + normals + colors.
- `compute_normals()`: area-weighted face normals averaged at vertices.
- 3 unit tests: square (2 triangles, normals up), insufficient points, default color.

### Step 4: Decimation (mesh.rs)
- `decimate()`: meshopt `simplify_decoder` with QEM, vertex remapping preserves colors.
- Passthrough when mesh is already below target.
- 2 unit tests: passthrough and reduction (30×30 grid → ≤200 triangles).

### Step 5: glTF binary export (export.rs)
- `to_glb()`: manual GLB construction with POSITION, NORMAL, COLOR_0, indices.
- Correct 4-byte alignment for JSON (space padding) and binary (zero padding).
- RGBA vertex colors (glTF requires VEC4 for COLOR_0).
- 2 unit tests: magic+version validation, JSON chunk parseability.

### Step 6: PNG plan view (export.rs)
- `to_plan_view_png()`: orthographic projection with elevation shading.
- Barycentric triangle rasterizer with color interpolation.
- Optional canopy overlay (3×3 dark dots for obstacle points).
- Image capped at 4096×4096 to prevent memory explosion.
- 1 unit test: PNG magic bytes.

### Step 7: Pipeline orchestrator (export.rs)
- `generate_terrain()`: triangulate → decimate → glb + png + metadata.
- `TerrainOutput`: mesh_glb, plan_view_png, metadata.
- `TerrainMetadata`: bbox, elevation_range, counts, processing_time_ms.
- `ExportConfig`: mesh config + pixels_per_meter + canopy_overlay.
- 2 unit tests: outputs non-empty, metadata consistency.

### Step 8: Module wiring (lib.rs)
- Added `pub mod mesh; pub mod export;` and re-exports.
- Public API: `generate_terrain`, `ExportConfig`, `TerrainMetadata`, `TerrainOutput`, `MeshConfig`, `TerrainMesh`.

### Step 9: Integration tests
- `test_terrain_generation_pipeline`: full PLY → 3 artifacts, metadata checks.
- `test_glb_structure`: GLB header, chunk types, JSON fields, accessor count.
- `test_plan_view_png_valid`: PNG signature, image decodable with positive dimensions.
- 6 integration tests total (3 existing + 3 new).

### Step 10: S.1.1 scenario update
- Added `generate_terrain()` call after existing PointCloud checks.
- Validates GLB magic, PNG signature, metadata triangle/vertex counts.
- Remains at OneStar (pure computation, no API integration).

### Step 11: Milestone update
- "pt-scan: PLY parsing + mesh generation" now delivered_by T-015-02.
- Note expanded to cover full pipeline: parsing + filtering + mesh gen + export.

### Step 12: Quality gate
- `just check` passes: fmt, lint (clippy strict), test (31 tests), scenarios (7 pass, 0 fail).
- No regressions. S.1.1 passes at ★☆☆☆☆.

## Deviations from plan

- `delaunator` version bumped from 0.5 to 1 by project hook (API compatible).
- Used `simplify_decoder` instead of `simplify` for meshopt (0.4 API uses `DecodePosition` trait, `[f32; 3]` implements it natively).
- PNG resolution uses `pixels_per_meter` instead of `pixels_per_foot` to stay consistent with metric coordinates in the scan pipeline.
