# T-032-02 Structure: scan-metadata-report

## File Changes

### New File: `crates/pt-scan/src/report.rs`

New module containing all report types and a helper to populate output info.

**Public types:**
- `ScanReport` — top-level composite struct
- `InputInfo` — input file metadata
- `ProcessingInfo` — pipeline processing stats
- `GroundInfo` — ground plane details + area estimate
- `ObstacleInfo` — obstacle count, height range, bbox
- `StageTiming` — per-stage durations in milliseconds
- `OutputInfo` — export artifact sizes and counts

**Public functions:**
- `OutputInfo::from_terrain(output: &TerrainOutput) -> OutputInfo` — convenience to fill output info from terrain export results

All types derive `Debug, Clone, Serialize, Deserialize`.

### Modified: `crates/pt-scan/src/types.rs`

Add `best_iteration: usize` field to `GroundClassification` struct. This is an internal type (no Serde), so no serialization concerns.

### Modified: `crates/pt-scan/src/ransac.rs`

Track which iteration produced the best plane. Update `fit_ground_plane()` and `fit_ground_plane_seeded()` to set `best_iteration` on the returned `GroundClassification`.

### Modified: `crates/pt-scan/src/lib.rs`

1. Add `pub mod report;` declaration.
2. Add re-exports: `pub use report::{ScanReport, InputInfo, ProcessingInfo, GroundInfo, ObstacleInfo, StageTiming, OutputInfo};`
3. Add `pub fn process_scan_timed(reader, config) -> Result<(PointCloud, ScanReport), ScanError>` — instrumented version of the pipeline.
4. Keep `process_scan()` unchanged.

### Modified: `crates/pt-scan/examples/process_sample.rs`

1. Replace manual per-stage pipeline with call to `process_scan_timed()`.
2. Fill `report.input` with filename and file size.
3. After `generate_terrain()`, fill `report.output` and `report.timing.terrain_export_ms`.
4. Write `{stem}-report.json` alongside GLB and PNG.
5. Print report summary to stdout (replaces current ad-hoc metadata printing).

### Modified: `crates/pt-scan/tests/integration.rs`

Add test: `test_scan_report_serialization_round_trip` — creates a report from `process_scan_timed`, serializes to JSON, deserializes back, asserts field equality.

## Module Boundaries

```
lib.rs
├── report.rs    (NEW — ScanReport and sub-types)
├── types.rs     (GroundClassification gains best_iteration)
├── ransac.rs    (tracks best_iteration)
├── parser.rs    (unchanged)
├── filter.rs    (unchanged)
├── mesh.rs      (unchanged)
└── export.rs    (unchanged)
```

## Public API Surface Changes

| Item | Change |
|---|---|
| `ScanReport` | New public type |
| `InputInfo` | New public type |
| `ProcessingInfo` | New public type |
| `GroundInfo` | New public type |
| `ObstacleInfo` | New public type |
| `StageTiming` | New public type |
| `OutputInfo` | New public type |
| `process_scan_timed()` | New public function |
| `GroundClassification.best_iteration` | New field (internal type, not public API) |
| `process_scan()` | Unchanged |
| `ScanMetadata` | Unchanged |
| `TerrainMetadata` | Unchanged |

## Interface Details

### process_scan_timed

```rust
pub fn process_scan_timed(
    reader: impl std::io::Read,
    config: &ScanConfig,
) -> Result<(PointCloud, ScanReport), ScanError>
```

Returns the same `PointCloud` as `process_scan()` plus a `ScanReport` with:
- `input.original_vertex_count` populated, other input fields at defaults
- `processing` fully populated
- `ground` fully populated including area estimate
- `obstacles` fully populated including height range and bbox
- `timing` populated for processing stages (terrain_export_ms = None)
- `output` = None (caller fills after export)

### ScanReport mutation points

The caller is expected to fill:
1. `report.input.filename` — from CLI arg or API request
2. `report.input.file_size_bytes` — from fs::metadata
3. `report.output` — from `OutputInfo::from_terrain(&terrain_output)`
4. `report.timing.terrain_export_ms` — from terrain generation timing

These are public fields, no builder pattern needed.

## Ordering of Changes

1. `types.rs` — add `best_iteration` to `GroundClassification` (other code compiles with field added)
2. `ransac.rs` — track and set `best_iteration` (fixes compile after step 1)
3. `report.rs` — new module, all types (compiles independently)
4. `lib.rs` — add module, re-exports, `process_scan_timed` (depends on steps 1-3)
5. `examples/process_sample.rs` — update to use new API (depends on step 4)
6. `tests/integration.rs` — add round-trip test (depends on step 4)
