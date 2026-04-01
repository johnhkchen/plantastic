# T-032-01 Research: scan-cli-example

## Objective

Create a CLI example (`crates/pt-scan/examples/process_sample.rs`) that runs a real 20.5M-point PLY through the full pt-scan pipeline, prints timing/metadata, and writes GLB + PNG outputs.

## Crate API Surface

### `process_scan(reader: impl Read, config: &ScanConfig) -> Result<PointCloud, ScanError>`

Full pipeline: parse PLY → voxel downsample → outlier removal → RANSAC ground fitting.

Returns `PointCloud` with `.ground`, `.obstacles`, `.metadata` (counts, bbox, ground plane).

### `generate_terrain(cloud: &PointCloud, config: &ExportConfig) -> Result<TerrainOutput, ScanError>`

Terrain export: triangulate → decimate → GLB + PNG + metadata.

Returns `TerrainOutput` with `.mesh_glb`, `.plan_view_png`, `.metadata`.

### Key Config Types

**ScanConfig** (defaults: 2cm voxels, k=30, sigma=2.0, 1000 RANSAC iters, 2cm threshold):
- Ticket specifies outdoor urban config: 5cm voxels, k=20, 2.0 sigma, 1000 iters, 5cm threshold
- Larger voxels = more aggressive downsampling = faster for 20M points

**ExportConfig** (defaults: 50k target triangles, 30 px/m, canopy overlay on):
- Defaults are reasonable for outdoor scans

### ScanMetadata fields
- `original_count`, `filtered_count`, `ground_count`, `obstacle_count`
- `bbox: BoundingBox { min: [f32; 3], max: [f32; 3] }`
- `ground_plane: Plane { normal: [f32; 3], d: f32 }`

### TerrainMetadata fields
- `bbox`, `elevation_range: [f32; 2]`
- `original_point_count`, `decimated_triangle_count`, `vertex_count`
- `processing_time_ms`

## Dependencies

pt-scan's Cargo.toml has no `[[example]]` section yet. Cargo discovers examples automatically from `examples/` directory. The example needs only `pt-scan` as a dependency (plus `std::time`, `std::env`, `std::fs`, `std::path` from stdlib).

## Sample Data

- File: `assets/scans/samples/Scan at 09.23.ply` (294 MB, binary LE, RGB)
- Gitignored: `*.ply` in assets/scans/samples/
- Output files (.glb, .png) should also be gitignored

## Existing Patterns

### pt-proposal example pattern
- `crates/pt-proposal/examples/test_proposal.rs`
- Run via: `cargo run -p pt-proposal --example test_proposal`
- Takes CLI args, prints status, writes output files

### Justfile patterns
- Existing recipes use `#!/usr/bin/env bash` for multi-line
- No scan-related recipes exist yet
- Recipe should accept a PLY path argument

## Gitignore

Need to verify `.gitignore` covers output artifacts (.glb, .png) in the scans directory. The scan PLY files are already gitignored.

## Performance Considerations

- 20M points at 5cm voxels → ~400k-800k points after downsampling (depending on spatial extent)
- k-NN for outlier removal is the expensive step (O(n log n) with KD-tree)
- RANSAC is cheap at 1000 iterations regardless of point count
- Delaunay triangulation on ground points should be fast (sub-second for <500k points)
- Target: <60s total on M-series Mac

## Pipeline Stages for Timing

1. Parse PLY (I/O bound — reading 294 MB)
2. Voxel downsample (CPU, single pass with hashmap)
3. Outlier removal (CPU, KD-tree construction + k-NN queries)
4. RANSAC ground fitting (CPU, fast)
5. Terrain generation (triangulation + decimation + GLB/PNG export)
6. File write (I/O)

## Gaps / Constraints

- `process_scan` takes `impl Read`, not a file path — example must open the file and pass the reader
- No per-stage timing in the library — example must wrap each call with `Instant::now()`
- Library does process_scan as one atomic call; can't time internal stages separately
- The example can time: (1) parse+filter via process_scan, (2) terrain gen via generate_terrain, (3) file writes
- For finer-grained timing, we'd need to call parser/filter/ransac directly (they're pub modules)

## Decision Point

The ticket wants timing "for each stage." Since `process_scan` bundles parse → downsample → outlier → RANSAC, we have two options:
1. Call `process_scan` as a black box and only time the two high-level calls
2. Call the sub-modules directly (parser::parse_ply, filter::*, ransac::*) for per-stage timing

Option 2 is more informative and all sub-modules are pub. This is the right choice for a diagnostic example.
