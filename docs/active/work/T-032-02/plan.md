# T-032-02 Plan: scan-metadata-report

## Step 1: Add best_iteration to GroundClassification

**File:** `crates/pt-scan/src/types.rs`
- Add `pub best_iteration: usize` field to `GroundClassification`.

**File:** `crates/pt-scan/src/ransac.rs`
- Track `best_iteration` variable in `fit_ground_plane()` — update whenever a new best inlier count is found.
- Set `best_iteration` on the returned `GroundClassification`.
- Same change in `fit_ground_plane_seeded()`.

**Verify:** `cargo check -p pt-scan` compiles. Existing tests pass.

## Step 2: Create report module with all types

**File:** `crates/pt-scan/src/report.rs` (new)
- Define `ScanReport`, `InputInfo`, `ProcessingInfo`, `GroundInfo`, `ObstacleInfo`, `StageTiming`, `OutputInfo`.
- All derive `Debug, Clone, Serialize, Deserialize, PartialEq`.
- `OutputInfo::from_terrain(output: &TerrainOutput) -> Self` convenience method.
- Default implementations where sensible (InputInfo defaults, StageTiming zeroed).

**Verify:** `cargo check -p pt-scan` compiles (module not yet wired).

## Step 3: Wire report module and add process_scan_timed

**File:** `crates/pt-scan/src/lib.rs`
- Add `pub mod report;`
- Add re-exports for all report types.
- Implement `process_scan_timed()`:
  - Same pipeline as `process_scan` but with `Instant::now()` around each stage.
  - After RANSAC: compute obstacle heights (distance above ground plane), obstacle bbox, ground area estimate.
  - Build and return `ScanReport` alongside `PointCloud`.

**Verify:** `cargo check -p pt-scan` compiles.

## Step 4: Update CLI example

**File:** `crates/pt-scan/examples/process_sample.rs`
- Replace manual pipeline calls with `process_scan_timed()`.
- Fill `report.input.filename` and `report.input.file_size_bytes`.
- Run `generate_terrain()` as before, time it.
- Fill `report.output` via `OutputInfo::from_terrain()`.
- Fill `report.timing.terrain_export_ms`.
- Write `{stem}-report.json` via `serde_json::to_string_pretty`.
- Simplify stdout output to use report fields instead of manual computation.

**Verify:** `cargo build -p pt-scan --examples` compiles.

## Step 5: Add tests

**File:** `crates/pt-scan/tests/integration.rs`
- `test_scan_report_round_trip`: Run `process_scan_timed` on synthetic PLY, serialize report to JSON, deserialize back, assert equality.
- `test_scan_report_fields_populated`: Verify timing > 0, counts match between report and cloud metadata, obstacle height range is sensible for synthetic data.

**Verify:** `cargo test -p pt-scan` — all tests pass.

## Step 6: Quality gate

- `just check` — format, lint, test, scenarios all pass.
- No regressions in scenario dashboard.

## Testing Strategy

| Test | Type | What it verifies |
|---|---|---|
| `test_scan_report_round_trip` | Integration | JSON serialize → deserialize equality |
| `test_scan_report_fields_populated` | Integration | All fields have sensible values from synthetic data |
| Existing RANSAC tests | Unit | best_iteration doesn't break existing behavior |
| Existing integration tests | Integration | process_scan unchanged, pipeline still works |
