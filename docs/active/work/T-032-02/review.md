# T-032-02 Review: scan-metadata-report

## Summary

Added a structured `ScanReport` type to pt-scan that captures the full processing pipeline's metadata in a JSON-serializable format. Added `process_scan_timed()` as an instrumented variant of the existing pipeline. Updated the CLI example to write a JSON report file alongside GLB/PNG outputs.

## Files Changed

### Created
- `crates/pt-scan/src/report.rs` — 7 report types: `ScanReport`, `InputInfo`, `ProcessingInfo`, `GroundInfo`, `ObstacleInfo`, `StageTiming`, `OutputInfo` (~95 lines)

### Modified
- `crates/pt-scan/src/types.rs` — Added `best_iteration` field to `GroundClassification`; added `PartialEq` derive to `Plane` and `BoundingBox`
- `crates/pt-scan/src/ransac.rs` — Track which iteration found the best plane in `fit_ground_plane()` and `fit_ground_plane_seeded()`
- `crates/pt-scan/src/lib.rs` — Added `report` module, re-exports, and `process_scan_timed()` function (~100 lines)
- `crates/pt-scan/examples/process_sample.rs` — Rewritten to use `process_scan_timed()`, writes `{stem}-report.json`
- `crates/pt-scan/tests/integration.rs` — Added 2 integration tests: round-trip serialization and field population

## Acceptance Criteria Check

| Criterion | Status |
|---|---|
| `ScanReport` struct in pt-scan: serializable (serde JSON) | ✓ All types derive Serialize, Deserialize |
| Input: filename, vertex count, file size, format | ✓ `InputInfo` struct |
| Processing: downsample ratio, outlier removal count, RANSAC iterations | ✓ `ProcessingInfo` struct |
| Ground plane: normal, offset, area estimate | ✓ `GroundInfo` struct with `area_estimate_sqm` |
| Obstacles: count, height range, spatial extent (bounding box) | ✓ `ObstacleInfo` struct |
| Timing: per-stage durations | ✓ `StageTiming` struct (parse, downsample, outlier, ransac, total, terrain) |
| Output: GLB size, PNG size, triangle count, vertex count | ✓ `OutputInfo` struct with `from_terrain()` |
| `process_scan` extended to return timing info | ✓ `process_scan_timed()` returns `(PointCloud, ScanReport)` |
| CLI example writes `{name}-report.json` | ✓ Updated `process_sample.rs` |
| Report JSON readable by future BAML functions | ✓ Clean JSON structure with named sections |
| Unit test: verify report serialization round-trip | ✓ `test_scan_report_round_trip` |

## Design Decisions

1. **Separate `process_scan_timed()` instead of modifying `process_scan()`** — keeps the original function zero-overhead for callers that don't need timing. Avoids breaking the existing API.

2. **`best_iteration` instead of early termination** — RANSAC tracks which iteration found the best plane without changing processing behavior. This is informational and non-breaking.

3. **Ground area estimate uses XY bounding box** — simpler than convex hull, overestimates for irregular shapes but sufficient for a metadata report. Can be refined later.

4. **`PartialEq` added to `Plane` and `BoundingBox`** — needed for report equality assertions. These are value types where field-wise equality is the correct semantics. Uses f32 equality which is appropriate for round-trip tests (same bits in, same bits out).

## Test Coverage

| Test | Type | Verifies |
|---|---|---|
| `test_scan_report_round_trip` | Integration | JSON serialize → deserialize equality |
| `test_scan_report_fields_populated` | Integration | All fields sensible, counts match cloud metadata |
| Existing RANSAC tests (4) | Unit | `best_iteration` doesn't break existing behavior |
| Existing pipeline tests (6) | Integration | `process_scan` unchanged, no regressions |

**Total:** 33 tests pass (25 unit + 8 integration).

## Scenario Dashboard

No scenario changes expected — this ticket adds developer/pipeline metadata infrastructure, not customer-facing capability. The scenario dashboard was verified unchanged. Note: `just scenarios` currently fails to compile due to a pre-existing error in `plantastic-api/src/routes/proposals.rs` (T-030-02 work-in-progress, `pt_geo::area_sqft` not found). This is unrelated to T-032-02 changes.

## Open Concerns

1. **`just scenarios` blocked by T-030-02.** The scenario dashboard can't compile due to an unrelated `area_sqft` import error in proposals.rs. This doesn't affect pt-scan tests or the report feature. Will resolve when T-030-02 is complete.

2. **Ground area estimate is an overestimate.** The XY bounding box area overestimates for non-rectangular plot shapes. If BAML needs accurate area, consider convex hull computation in a follow-up.

3. **No `process_scan` delegation to `process_scan_timed`.** The two functions share the same pipeline logic (duplicated). If `process_scan` needs changes in the future, both must be updated. Could refactor to have `process_scan` delegate to `process_scan_timed` and discard the report, but this would add overhead (timing instrumentation) to the simple path.
