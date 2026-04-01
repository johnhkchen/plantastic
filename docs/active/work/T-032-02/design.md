# T-032-02 Design: scan-metadata-report

## Decision

**Option B: Composite `ScanReport` with `process_scan_timed`** — a new `ScanReport` struct that composes sub-structs for each metadata category, built by a new `process_scan_timed()` function that wraps the existing pipeline with per-stage timing. The CLI example populates input/output sections and writes the JSON.

## Options Evaluated

### Option A: Extend existing ScanMetadata

Add timing fields, obstacle bbox, etc. directly to `ScanMetadata`.

- **Pro:** Minimal new types. No API surface growth.
- **Con:** `ScanMetadata` is embedded in `PointCloud` which is serialized everywhere. Adding timing and output sizes to it bloats a struct that should be lightweight point-cloud metadata. Mixes concerns (processing diagnostics vs. data description). Breaking change for existing serialized `PointCloud` JSON.
- **Rejected:** Violates single-responsibility. `ScanMetadata` describes the cloud, not the processing history.

### Option B: Composite ScanReport struct (chosen)

New `ScanReport` struct with sub-sections: `input`, `processing`, `ground`, `obstacles`, `timing`, `output`. Built by `process_scan_timed()` which returns `(PointCloud, ScanReport)`. Input/output sections filled externally (CLI or API).

- **Pro:** Clean separation. Existing types unchanged. Sub-structs are independently useful. JSON structure maps directly to ticket requirements. BAML can reference specific sections.
- **Pro:** `process_scan_timed()` is additive — `process_scan()` stays unchanged for callers that don't need timing.
- **Con:** More types. Slightly more API surface.
- **Chosen:** Best fit for ticket requirements and downstream BAML consumption.

### Option C: Builder pattern / report collector

Pass a mutable `ReportCollector` through the pipeline that each stage writes to.

- **Pro:** Most flexible. Stages don't need to return extra data.
- **Con:** Over-engineered for 5 stages. Adds mutable state threading through pure functions. Makes the pipeline harder to reason about.
- **Rejected:** Premature abstraction for a fixed pipeline.

## ScanReport Structure

```rust
pub struct ScanReport {
    pub input: InputInfo,
    pub processing: ProcessingInfo,
    pub ground: GroundInfo,
    pub obstacles: ObstacleInfo,
    pub timing: StageTiming,
    pub output: Option<OutputInfo>,  // None if terrain export not run
}
```

### Sub-structs

**InputInfo** — populated by caller (CLI/API), not by `process_scan_timed`:
```rust
pub struct InputInfo {
    pub filename: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub format: String,              // "ply"
    pub original_vertex_count: usize,
}
```

**ProcessingInfo** — populated by pipeline:
```rust
pub struct ProcessingInfo {
    pub downsample_ratio: f32,       // filtered / original
    pub downsampled_count: usize,
    pub outliers_removed: usize,
    pub ransac_iterations_config: usize,
    pub ransac_best_iteration: usize, // iteration that found best plane
}
```

**GroundInfo** — from RANSAC results:
```rust
pub struct GroundInfo {
    pub plane: Plane,
    pub point_count: usize,
    pub area_estimate_sqm: f32,      // bbox area of ground points projected onto XY
}
```

**ObstacleInfo** — from classification:
```rust
pub struct ObstacleInfo {
    pub count: usize,
    pub height_range: Option<[f32; 2]>,  // [min, max] distance above ground plane
    pub bbox: Option<BoundingBox>,        // obstacle-only bounding box
}
```

**StageTiming** — per-stage durations in milliseconds:
```rust
pub struct StageTiming {
    pub parse_ms: u64,
    pub downsample_ms: u64,
    pub outlier_removal_ms: u64,
    pub ransac_ms: u64,
    pub total_processing_ms: u64,
    pub terrain_export_ms: Option<u64>,  // filled after generate_terrain
}
```

**OutputInfo** — filled after terrain export:
```rust
pub struct OutputInfo {
    pub glb_size_bytes: usize,
    pub png_size_bytes: usize,
    pub triangle_count: usize,
    pub vertex_count: usize,
}
```

## RANSAC Convergence Tracking

The ticket asks for "RANSAC iterations to convergence." Current implementation always runs all configured iterations. Two options:

1. **Track best iteration** — record which iteration last improved the best plane. Report this as `ransac_best_iteration`. Cheap, no behavior change.
2. **Early termination** — stop when inlier ratio exceeds a threshold. Changes pipeline behavior, could produce different results.

**Decision:** Option 1 (track best iteration). It's informational, non-breaking, and answers the diagnostic question without changing processing results. Early termination can be a separate ticket if needed.

This requires `fit_ground_plane` to return `best_iteration` alongside `GroundClassification`. Rather than changing the return type (breaking), we'll add a `best_iteration` field to `GroundClassification`.

## Ground Area Estimate

The ticket asks for "area estimate." Convex hull would be most accurate but adds complexity. For a report, projecting ground points onto XY and computing the bounding box area is sufficient and fast:

```
area = (bbox.max[0] - bbox.min[0]) * (bbox.max[1] - bbox.min[1])
```

This is an overestimate for non-rectangular plots but good enough for a metadata report. Can be refined later if needed.

## process_scan_timed Design

```rust
pub fn process_scan_timed(
    reader: impl std::io::Read,
    config: &ScanConfig,
) -> Result<(PointCloud, ScanReport), ScanError>
```

- Runs the same pipeline as `process_scan` but wraps each stage with `Instant::now()`.
- `process_scan` is left unchanged — it just calls `process_scan_timed` and discards the report (or stays as-is for zero overhead when timing isn't needed).
- Input info (filename, file size) set to defaults; caller fills them after.
- Output info left as `None`; caller fills after `generate_terrain`.

## CLI Example Update

The `process_sample.rs` example already does manual per-stage timing. Updated version:
1. Calls `process_scan_timed` instead of manual stage calls.
2. Fills `report.input` with filename and file size.
3. After `generate_terrain`, fills `report.output` and `report.timing.terrain_export_ms`.
4. Writes `{stem}-report.json` alongside GLB and PNG.

This simplifies the example while demonstrating the report API.

## Testing Strategy

- **Unit test:** Create a `ScanReport` with known values, serialize to JSON, deserialize back, assert equality. This verifies the round-trip requirement.
- **Integration test:** Run `process_scan_timed` on synthetic PLY, verify report fields are populated and sensible (timing > 0, counts match, etc.).
