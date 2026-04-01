# T-015-01 Plan: PLY Parsing & Point Cloud Filtering

## Step Sequence

### Step 1: Scaffold crate with types and error

Create `crates/pt-scan/` with Cargo.toml, lib.rs, types.rs, error.rs.
Verify `cargo check -p pt-scan` passes.

**Files**: Cargo.toml, src/lib.rs, src/types.rs, src/error.rs
**Verify**: `cargo check -p pt-scan`

### Step 2: PLY parser

Implement `parse_ply()` in `src/parser.rs`. Handle binary LE, binary BE, and ASCII
formats. Extract x/y/z (required) and r/g/b (optional).

Write a test helper that generates synthetic binary PLY bytes in-memory (no file I/O).

**Tests**:
- `test_parse_binary_ply`: 100 points, verify count and coordinate range
- `test_parse_ascii_ply`: same data as ASCII, verify identical output
- `test_parse_missing_color`: PLY without r/g/b, verify color is None
- `test_parse_empty_ply`: 0 vertices, verify empty vec returned

**Verify**: `cargo test -p pt-scan`

### Step 3: Voxel downsampling

Implement `voxel_downsample()` in `src/filter.rs`. HashMap-based cell averaging.

**Tests**:
- `test_voxel_downsample_reduces_count`: 1000 points in a 1m cube with 0.5m voxels
  → expect 8 output points (2×2×2 grid). Independently computed.
- `test_voxel_downsample_preserves_bounds`: output bbox within input bbox
- `test_voxel_downsample_averages_color`: two points with different colors in same
  cell → verify averaged color
- `test_voxel_downsample_empty_input`: empty input → empty output

**Verify**: `cargo test -p pt-scan`

### Step 4: Statistical outlier removal

Implement `remove_outliers()` in `src/filter.rs`. Build KD-tree, compute mean
neighbor distances, filter by threshold.

**Tests**:
- `test_remove_outliers_filters_distant_points`: 100 clustered points + 5 outliers
  at distance 100.0 → outliers removed, cluster retained
- `test_remove_outliers_preserves_uniform_cloud`: all points equidistant → none removed
- `test_remove_outliers_empty_input`: empty → empty

**Verify**: `cargo test -p pt-scan`

### Step 5: RANSAC ground plane fitting

Implement `fit_ground_plane()` in `src/ransac.rs`. Random 3-point sampling, cross
product normal, inlier counting, best plane tracking.

**Tests**:
- `test_fit_horizontal_plane`: 500 points at z≈0 (noise σ=0.005) + 100 points at
  z=1.0 → plane normal ≈ (0,0,1), d ≈ 0, ground_indices has ~500, obstacle ~100
- `test_fit_tilted_plane`: points on z = 0.1x + 0.2y → verify normal direction
- `test_insufficient_points_error`: 2 points → ScanError::InsufficientPoints
- `test_no_ground_plane`: all points identical → ScanError::NoGroundPlane (degenerate)
- `test_classifies_obstacles`: verify obstacle points above plane threshold are classified

**Verify**: `cargo test -p pt-scan`

### Step 6: Top-level process_scan pipeline

Wire everything together in `lib.rs`: `process_scan(reader, config)` calls
parse_ply → voxel_downsample → remove_outliers → fit_ground_plane → build PointCloud.

Compute ScanMetadata (bbox, counts, plane).

**Tests** (integration test in `tests/integration.rs`):
- `test_full_pipeline`: Generate 1250-point synthetic PLY (1000 ground + 200 obstacle
  + 50 outlier). Process with default config. Verify:
  - No outliers in output (all z > 5 removed)
  - Ground points have z ≈ 0
  - Obstacle points have z > 0.2
  - Metadata counts are consistent
  - BBox covers expected range
- `test_default_config_values`: verify ScanConfig::default() field values

**Verify**: `cargo test -p pt-scan`

### Step 7: Claim milestone + update scenario harness

1. Update `tests/scenarios/src/progress.rs`:
   - Set pt-scan milestone `delivered_by: Some("T-015-01")`
   - Write note describing what was delivered
2. Add `pt-scan` to `tests/scenarios/Cargo.toml` dependencies
3. Update S.1.1 in `tests/scenarios/src/suites/site_assessment.rs`:
   - Change from `NotImplemented` to a real test that exercises `process_scan`
   - Since T-015-01 only delivers parsing+filtering (not mesh gen/export), the
     scenario should test what's available and return `OneStar` for the partial
     capability, OR remain `NotImplemented` if the scenario definition requires
     full mesh output
   - Decision: Update S.1.1 to test PLY → PointCloud pipeline, return `OneStar`.
     The scenario validates that scan data can be processed into a classified
     point cloud. T-015-02 will upgrade to higher star rating when mesh + export
     are added.

**Verify**: `just scenarios` — S.1.1 should show PASS ★☆☆☆☆

### Step 8: Quality gate

Run `just check` (fmt + lint + test + scenarios). Fix any issues.

**Verify**: `just check` exits 0

## Testing Strategy Summary

| Layer | What | Count |
|-------|------|-------|
| Unit (parser) | PLY parsing edge cases | 4 tests |
| Unit (filter) | Downsampling + outlier removal | 7 tests |
| Unit (ransac) | Plane fitting + classification | 5 tests |
| Integration | Full pipeline end-to-end | 2 tests |
| Scenario | S.1.1 scan processing | 1 scenario |

All unit tests use `pt_test_utils::timed()` for 10s enforcement.
Integration test uses `timed()` — synthetic data is small enough.
Expected values in tests are independently computed from known fixture geometry.

## Risk Mitigation

1. **ply-rs-bw API uncertainty**: If the API differs from documented, fall back to
   reading raw bytes after header. PLY binary format is simple (just packed structs).
2. **kiddo API changes**: Pin to v5. The `ImmutableKdTree` API is stable.
3. **RANSAC determinism**: Use seeded RNG (`rand::SeedableRng`) in tests for
   reproducible results. Production uses thread_rng.
4. **Performance**: Synthetic test data is small (< 2K points). Add a `#[ignore]`
   benchmark test with 1M points if performance verification is needed, referencing
   T-015-01 as the unblock ticket.
