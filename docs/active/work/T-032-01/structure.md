# T-032-01 Structure: scan-cli-example

## Files Created

### `crates/pt-scan/examples/process_sample.rs` (NEW, ~150 lines)

Single-file example. No module structure needed.

**Sections:**
1. **Imports** — pt_scan sub-modules, std::time, std::fs, std::path, std::io
2. **`main()`** — entry point, orchestrates pipeline
3. **`format_bytes()`** — helper to format byte sizes (B/KB/MB)
4. **`format_count()`** — helper to format numbers with commas

**Flow in `main()`:**
```
parse_args() → resolve PLY path
    ↓
open file → BufReader
    ↓
Stage 1: parser::parse_ply(reader) → raw_points
Stage 2: filter::voxel_downsample(&raw_points, 0.05) → downsampled
Stage 3: filter::remove_outliers(&downsampled, 20, 2.0) → filtered
Stage 4: ransac::fit_ground_plane(&filtered, 1000, 0.05) → classification
    ↓
Assemble PointCloud from classification results
    ↓
Stage 5: generate_terrain(&cloud, &ExportConfig::default()) → terrain
    ↓
Write terrain.mesh_glb → {stem}-terrain.glb
Write terrain.plan_view_png → {stem}-planview.png
    ↓
Print summary metadata
```

**Public API used from pt_scan:**
- `pt_scan::parser::parse_ply` (pub fn)
- `pt_scan::filter::voxel_downsample` (pub fn)
- `pt_scan::filter::remove_outliers` (pub fn)
- `pt_scan::ransac::fit_ground_plane` (pub fn)
- `pt_scan::generate_terrain` (pub fn, re-exported)
- `pt_scan::types::*` (Point, PointCloud, ScanMetadata, BoundingBox, Plane)
- `pt_scan::export::ExportConfig` (re-exported)

### `justfile` (MODIFIED, +4 lines)

Add recipe after the `scenarios` section:

```just
# Process a PLY scan through the full pipeline (release mode)
process-scan path="assets/scans/samples/Scan at 09.23.ply":
    cargo run -p pt-scan --example process_sample --release -- "{{path}}"
```

## Files NOT Modified

- `crates/pt-scan/Cargo.toml` — No new dependencies needed. Example only uses pt-scan's existing deps.
- `crates/pt-scan/src/lib.rs` — No changes to library code.
- `.gitignore` — Need to verify coverage for .glb/.png in assets/scans/samples/. May need a line.

## Gitignore Check

Current `.gitignore` likely covers `*.ply` in assets/scans/samples/. Need to verify it also covers `*.glb` and `*.png` output files in the same directory. If not, add a rule.

## Module Boundary

The example is a consumer of pt-scan's public API. It does not modify any library code. All sub-module functions it calls are already `pub`.

Functions used from sub-modules:
- `parser::parse_ply` — declared `pub fn parse_ply(reader: impl Read) -> Result<Vec<Point>, ScanError>`
- `filter::voxel_downsample` — declared `pub fn voxel_downsample(points: &[Point], voxel_size: f32) -> Vec<Point>`
- `filter::remove_outliers` — declared `pub fn remove_outliers(points: &[Point], k: usize, threshold: f32) -> Vec<Point>`
- `ransac::fit_ground_plane` — declared `pub fn fit_ground_plane(points: &[Point], iterations: usize, threshold: f32) -> Result<GroundClassification, ScanError>`

## No Test Changes

This is an example binary, not library code. The existing test suite covers all library functions used. The example itself is validated by:
1. `cargo build -p pt-scan --examples` (compiles)
2. Running it against the sample PLY (manual, not CI — file is gitignored)
3. `just check` still passes (no library changes)
