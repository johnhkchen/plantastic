# T-015-01 Review: PLY Parsing & Point Cloud Filtering

## Summary

Created `crates/pt-scan/` — a pure-computation Rust crate that reads PLY files and
produces classified point clouds (ground vs. obstacles). This is the first half of the
scan processing pipeline; T-015-02 will add mesh generation and export.

## Files Created

| File | Purpose |
|------|---------|
| `crates/pt-scan/Cargo.toml` | Crate manifest: ply-rs-bw, kiddo, nalgebra, rand, indexmap |
| `crates/pt-scan/src/lib.rs` | Public API: `process_scan()`, re-exports |
| `crates/pt-scan/src/types.rs` | Point, PointCloud, ScanConfig, ScanMetadata, BoundingBox, Plane |
| `crates/pt-scan/src/error.rs` | ScanError enum (InvalidPly, InsufficientPoints, NoGroundPlane, Io) |
| `crates/pt-scan/src/parser.rs` | PLY parsing via ply-rs-bw (binary LE/BE + ASCII) |
| `crates/pt-scan/src/filter.rs` | voxel_downsample() + remove_outliers() (kiddo KD-tree) |
| `crates/pt-scan/src/ransac.rs` | fit_ground_plane() (RANSAC via nalgebra + rand) |
| `crates/pt-scan/tests/integration.rs` | Full pipeline integration tests |

## Files Modified

| File | Change |
|------|--------|
| `tests/scenarios/Cargo.toml` | Added `pt-scan` dependency |
| `tests/scenarios/src/suites/site_assessment.rs` | S.1.1: NotImplemented → Pass(OneStar) |
| `tests/scenarios/src/progress.rs` | Milestone claimed by T-015-01 with delivery note |

## Test Coverage

| Module | Tests | What's covered |
|--------|-------|----------------|
| parser | 4 | Binary LE, ASCII, missing color, empty PLY |
| filter (voxel) | 4 | Reduces count, preserves bounds, averages color, empty |
| filter (outlier) | 3 | Filters distant points, preserves uniform cloud, empty |
| ransac | 4 | Horizontal plane, tilted plane, insufficient points, obstacle classification |
| integration | 3 | Full pipeline, default config values, insufficient points |
| scenario | 1 | S.1.1 exercises process_scan end-to-end |
| **Total** | **19** | |

All expected values in tests are independently computed from known fixture geometry
(per CLAUDE.md rule 2). No cross-crate mocks (rule 3). No bare `#[ignore]` (rule 4).

### Coverage gaps

- **Binary big-endian PLY**: Not explicitly tested. ply-rs-bw handles it internally,
  but we have no test fixture for it. Low risk — SiteScape exports little-endian.
- **Real SiteScape PLY data**: All tests use synthetic fixtures. Edge cases in real
  LiDAR data (NaN coordinates, degenerate geometries, non-standard properties) are
  untested. Should be validated when a real PLY file is available.
- **Performance at 5M points**: Not benchmarked. Synthetic test data is intentionally
  small (~1250 points) for fast tests. The pipeline order (downsample first) ensures
  k-NN runs on reduced data, which should meet the 10-second target.

## Scenario Dashboard: Before / After

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Effective savings | 48.0 min | 54.0 min | +6.0 |
| Percentage | 20.0% | 22.5% | +2.5pp |
| Scenarios passing | 6 | 7 | +1 |
| Milestones delivered | 9 | 10 | +1 |

S.1.1 "Scan processing" went from NotImplemented to PASS at OneStar (30 min raw × 0.2 = 6 min effective). No regressions — all previously passing scenarios remain passing.

## Public API

```rust
// Top-level pipeline
pub fn process_scan(reader: impl Read, config: &ScanConfig) -> Result<PointCloud, ScanError>;

// Individual stages (composable for T-015-02)
pub fn parse_ply(reader: impl Read) -> Result<Vec<Point>, ScanError>;
pub fn voxel_downsample(points: &[Point], voxel_size: f32) -> Vec<Point>;
pub fn remove_outliers(points: &[Point], k: usize, threshold: f32) -> Vec<Point>;
pub fn fit_ground_plane(points: &[Point], iterations: usize, threshold: f32)
    -> Result<GroundClassification, ScanError>;
```

## Dependencies Added

| Crate | Version | Purpose | Downloads |
|-------|---------|---------|-----------|
| ply-rs-bw | 3 | PLY file parsing (maintained fork) | 346K |
| kiddo | 5 | KD-tree for k-NN queries | 5.2M |
| nalgebra | 0.34 | Linear algebra (cross product, normals) | 61.5M |
| rand | 0.10 | RANSAC random sampling | 1B+ |
| indexmap | 2 | Required by ply-rs-bw's public API | transitive |

All dependencies are well-maintained, high-download crates. No unsafe code in pt-scan.

## Open Concerns

1. **No real PLY fixture**: The ticket notes "If no real PLY available yet, generate
   synthetic test fixture." We've done this. When a real SiteScape scan is captured,
   the parser should be validated against it. Filing this as known — not a blocker.

2. **Performance validation deferred**: The 5M-point performance target is designed
   for but not measured. The pipeline order (downsample → outlier removal) ensures
   k-NN runs on reduced data (~500K points at 2cm voxels). A benchmark test should
   be added when real data is available.

3. **f32 coordinate precision**: LiDAR data uses absolute coordinates. At scales beyond
   ~1km from origin, f32 precision degrades to ~0.1mm which is still acceptable for
   landscaping (typical yard is < 100m). If scans cover larger areas, coordinate
   re-centering would be needed.

## What T-015-02 Needs

T-015-02 (mesh generation + export) should use:
- `pt_scan::parse_ply()` or `pt_scan::process_scan()` to get a classified PointCloud
- `cloud.ground` for Delaunay triangulation input
- `cloud.metadata.bbox` for orthographic projection bounds
- `Point.color` for vertex colors in glTF export
- `ScanMetadata` fields for the metadata JSON output

The individual stage functions are public so T-015-02 can compose a different pipeline
if needed (e.g., skip voxel downsampling for mesh generation, or use different params).

## Quality Gate

`just check` passes: format, lint (clippy strict), test (19 pass, 0 fail), scenarios
(7 pass, 0 fail, 10 not implemented, 0 blocked).
