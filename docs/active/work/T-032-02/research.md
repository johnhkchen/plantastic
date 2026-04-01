# T-032-02 Research: scan-metadata-report

## Objective

Map the codebase to understand where metadata is currently produced, what's missing for a full `ScanReport`, and how downstream systems would consume it.

## Current Metadata Landscape

### ScanMetadata (types.rs:57-65)

The existing metadata struct, embedded in `PointCloud`:

```rust
pub struct ScanMetadata {
    pub bbox: BoundingBox,
    pub original_count: usize,
    pub filtered_count: usize,
    pub ground_count: usize,
    pub obstacle_count: usize,
    pub ground_plane: Plane,
}
```

Already has Serde derives. Covers point counts and ground plane but lacks: input file info, processing config, per-stage timing, obstacle spatial extent, and output artifact sizes.

### TerrainMetadata (export.rs:28-37)

Produced by `generate_terrain()`, covers the export stage:

```rust
pub struct TerrainMetadata {
    pub bbox: BoundingBox,
    pub elevation_range: [f32; 2],
    pub original_point_count: usize,
    pub decimated_triangle_count: usize,
    pub vertex_count: usize,
    pub processing_time_ms: u64,  // terrain gen only
}
```

Also has Serde derives. The `processing_time_ms` only covers terrain generation (mesh + GLB + PNG), not the full pipeline.

### Timing in process_scan (lib.rs:28-95)

`process_scan()` does NOT collect any timing. It runs parse → downsample → outlier → RANSAC sequentially but discards all intermediate measurements.

### Timing in process_sample.rs (examples/)

The CLI example manually instruments each stage with `Instant::now()` / `elapsed()`. This is ad-hoc — the timing lives in the example, not in the library. The ticket wants this in the library.

### RANSAC Internals (ransac.rs:24-103)

`fit_ground_plane()` runs up to `iterations` RANSAC loops, keeping the best plane. It does NOT report how many iterations it actually needed to converge (it always runs all iterations). The ticket asks for "RANSAC iterations to convergence" — this would require either early termination logic or tracking when the best plane was last updated.

### Obstacle Spatial Extent

Not computed anywhere in the library. The CLI example computes obstacle height range manually (lines 173-195). Obstacle bounding box is not computed — only the combined bbox exists.

### File-Level Input Info

Not tracked. `process_scan()` takes a `reader`, not a path. Filename, file size, and format are only known at the call site (CLI example or API).

## Key Functions and Their Signatures

| Function | File | Returns | Timing? |
|---|---|---|---|
| `process_scan(reader, config)` | lib.rs:28 | `PointCloud` | No |
| `generate_terrain(cloud, config)` | export.rs:68 | `TerrainOutput` | Yes (terrain only) |
| `parser::parse_ply(reader)` | parser.rs:23 | `Vec<Point>` | No |
| `filter::voxel_downsample(points, voxel)` | filter.rs:15 | `Vec<Point>` | No |
| `filter::remove_outliers(points, k, thresh)` | filter.rs:95 | `Vec<Point>` | No |
| `ransac::fit_ground_plane(points, iters, thresh)` | ransac.rs:24 | `GroundClassification` | No |

## Downstream Consumers

1. **CLI example** (`process_sample.rs`) — prints metadata to stdout, could write JSON report alongside GLB/PNG.
2. **BAML classification** — future LLM pipeline that needs scan context as structured JSON input.
3. **API responses** — `plantastic-api` imports pt-scan; would include report in scan results.

## Serialization Patterns

All public types in types.rs already derive `Serialize, Deserialize`. The new `ScanReport` should follow this pattern. JSON is the target format for BAML consumption.

## What's Missing (Gap Analysis)

| Ticket Requirement | Current State | Gap |
|---|---|---|
| Input: filename, vertex count, file size, format | Not tracked in library | Need `InputInfo` struct, populated at call site |
| Processing: downsample ratio | `original_count` and `filtered_count` exist | Ratio = filtered/original, computable but not stored |
| Processing: outlier removal count | Not tracked | Need `downsampled_count` before outlier removal |
| Processing: RANSAC iterations to convergence | Always runs all iterations | Need to track best-iteration index or add early termination |
| Ground plane: normal, offset | ✅ `Plane` in `ScanMetadata` | Already covered |
| Ground plane: area estimate | Not computed | Need ground footprint area (convex hull or bbox area) |
| Obstacles: count | ✅ `obstacle_count` in `ScanMetadata` | Already covered |
| Obstacles: height range | Computed ad-hoc in example | Need in library |
| Obstacles: spatial extent (bbox) | Not computed | Need obstacle-only bbox |
| Timing: per-stage durations | Ad-hoc in example only | Need `StageTiming` struct |
| Output: GLB size, PNG size | Known after export | Need to capture in report |
| Output: triangle count, vertex count | ✅ In `TerrainMetadata` | Already covered |

## Constraints

- `process_scan()` takes a `reader`, not a file path — input metadata (filename, file size) must be provided externally or in a wrapper.
- RANSAC doesn't track convergence — simplest fix: report which iteration found the best plane.
- The report must be a single JSON-serializable struct that captures the full pipeline.
- Existing `ScanMetadata` and `TerrainMetadata` should not be removed (they're used by other code), but the report can compose or reference them.

## Files Relevant to This Ticket

- `crates/pt-scan/src/types.rs` — ScanReport struct goes here (or new report.rs)
- `crates/pt-scan/src/lib.rs` — process_scan_timed or extended process_scan
- `crates/pt-scan/src/ransac.rs` — convergence tracking
- `crates/pt-scan/src/export.rs` — TerrainMetadata, output sizes
- `crates/pt-scan/examples/process_sample.rs` — update to write report JSON
- `crates/pt-scan/tests/integration.rs` — round-trip serialization test
