# T-015-01 Structure: PLY Parsing & Point Cloud Filtering

## New Files

### crates/pt-scan/Cargo.toml

```toml
[package]
name = "pt-scan"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
ply-rs-bw = "3"
kiddo = "5"
nalgebra = "0.34"
rand = "0.10"
thiserror.workspace = true
serde.workspace = true

[dev-dependencies]
pt-test-utils = { path = "../pt-test-utils" }
approx = "0.5"

[lints]
workspace = true
```

### crates/pt-scan/src/lib.rs

Public API re-exports. Follows existing crate pattern.

```
pub mod error;
pub mod parser;
pub mod filter;
pub mod ransac;
pub mod types;

pub use error::ScanError;
pub use types::{BoundingBox, Plane, Point, PointCloud, ScanConfig, ScanMetadata};

/// Top-level processing function.
pub fn process_scan(reader: impl std::io::Read, config: &ScanConfig) -> Result<PointCloud, ScanError>;
```

### crates/pt-scan/src/types.rs

Core data types. All derive `Debug, Clone, serde::Serialize, serde::Deserialize`.

Types:
- `Point` — position: [f32; 3], color: Option<[u8; 3]>
- `PointCloud` — ground: Vec<Point>, obstacles: Vec<Point>, metadata: ScanMetadata
- `ScanMetadata` — bbox, original_count, filtered_count, ground/obstacle counts, ground_plane
- `BoundingBox` — min: [f32; 3], max: [f32; 3]
- `Plane` — normal: [f32; 3], d: f32 (ax + by + cz + d = 0)
- `ScanConfig` — voxel_size, outlier_k, outlier_threshold, ransac_iterations, ransac_threshold

`ScanConfig` implements `Default` with production values:
- voxel_size: 0.02 (2cm)
- outlier_k: 30
- outlier_threshold: 2.0
- ransac_iterations: 1000
- ransac_threshold: 0.02 (2cm)

### crates/pt-scan/src/error.rs

Error type using `thiserror`.

```
pub enum ScanError {
    InvalidPly(String),
    InsufficientPoints { found: usize, needed: usize },
    NoGroundPlane,
    Io(std::io::Error),
}
```

### crates/pt-scan/src/parser.rs

PLY file parsing. Single public function:

```
pub fn parse_ply(reader: impl Read) -> Result<Vec<Point>, ScanError>
```

Internal logic:
1. Create `ply_rs_bw::parser::Parser::<DefaultElement>::new()`
2. Read header, validate format
3. Find "vertex" element, extract x/y/z (required) and r/g/b (optional) properties
4. Convert to `Vec<Point>`
5. Compute and return points

Handles: ASCII, binary little-endian, binary big-endian. Handles missing color gracefully.
Maps ply-rs-bw errors to `ScanError::InvalidPly`.

### crates/pt-scan/src/filter.rs

Point cloud filtering. Two public functions:

```
pub fn voxel_downsample(points: &[Point], voxel_size: f32) -> Vec<Point>
pub fn remove_outliers(points: &[Point], k: usize, threshold: f32) -> Vec<Point>
```

#### voxel_downsample

1. Quantize each point position to `(i32, i32, i32)` cell key
2. Accumulate position sums, color sums, and counts per cell in HashMap
3. Output averaged point per cell
4. Preserve color averaging (round to nearest u8)

#### remove_outliers

1. Build `ImmutableKdTree<f32, 3>` from point positions
2. For each point, query `nearest_n(k)`, compute mean distance
3. Compute global mean μ and stddev σ of all mean distances
4. Retain points where mean_distance ≤ μ + threshold × σ
5. Return filtered vec

### crates/pt-scan/src/ransac.rs

RANSAC ground plane fitting. One public function:

```
pub fn fit_ground_plane(
    points: &[Point],
    iterations: usize,
    distance_threshold: f32,
) -> Result<GroundClassification, ScanError>

pub struct GroundClassification {
    pub ground_indices: Vec<usize>,
    pub obstacle_indices: Vec<usize>,
    pub plane: Plane,
}
```

Internal logic:
1. Validate ≥ 3 points
2. For each iteration:
   a. Sample 3 random indices via `rand::seq::index::sample`
   b. Compute plane: normal = cross product, d = -dot(normal, p0)
   c. Normalize the normal vector
   d. Count inliers: |dot(normal, point) + d| ≤ threshold
   e. Track best plane (most inliers)
3. If best_count == 0, return `ScanError::NoGroundPlane`
4. Final classification: all points against best plane
5. Return indices + plane

Uses `nalgebra::Vector3<f32>` internally for cross/dot products. Converts from/to
`[f32; 3]` at function boundaries.

## Modified Files

### Cargo.toml (workspace root)

No changes needed. `crates/*` glob already includes new crate.

### tests/scenarios/src/progress.rs

Update the pt-scan milestone after implementation:
- Set `delivered_by: Some("T-015-01")`
- Add note describing what was delivered

### tests/scenarios/Cargo.toml

Add `pt-scan` as a dependency so the scenario test can exercise it.

## Module Dependency Graph

```
lib.rs
  ├── types.rs      (no internal deps)
  ├── error.rs       (no internal deps)
  ├── parser.rs      (depends on: types, error)
  ├── filter.rs      (depends on: types)
  └── ransac.rs      (depends on: types, error)
```

All modules depend on `types` for `Point`, `Plane`, etc. Only `parser` and `ransac`
depend on `error` (they return `Result`). `filter` functions are infallible (worst
case: empty output).

## Public Interface Summary

The crate exports:
- `process_scan(reader, config) → Result<PointCloud, ScanError>` — top-level pipeline
- `parse_ply(reader) → Result<Vec<Point>, ScanError>` — standalone PLY parser
- `voxel_downsample(points, voxel_size) → Vec<Point>` — standalone filter
- `remove_outliers(points, k, threshold) → Vec<Point>` — standalone filter
- `fit_ground_plane(points, iters, threshold) → Result<GroundClassification, ScanError>`
- All types: Point, PointCloud, ScanConfig, ScanMetadata, BoundingBox, Plane, ScanError

Individual functions are public so T-015-02 can compose them differently if needed,
and so tests can exercise each stage independently.

## Test File Structure

### crates/pt-scan/src/parser.rs (inline tests)

- `test_parse_binary_ply` — parse synthetic binary fixture
- `test_parse_ascii_ply` — parse synthetic ASCII fixture
- `test_parse_missing_color` — PLY without color properties

### crates/pt-scan/src/filter.rs (inline tests)

- `test_voxel_downsample_reduces_count`
- `test_voxel_downsample_preserves_bounds`
- `test_remove_outliers_filters_distant_points`
- `test_remove_outliers_preserves_inliers`

### crates/pt-scan/src/ransac.rs (inline tests)

- `test_fit_horizontal_plane`
- `test_fit_tilted_plane`
- `test_insufficient_points_error`
- `test_classifies_obstacles_above_ground`

### crates/pt-scan/tests/integration.rs

- `test_full_pipeline` — PLY → process_scan → verify ground/obstacle split
- `test_default_config` — verify ScanConfig::default() values are sane
