# T-033-03 Research: Eigenvalue Features for Point Cloud Segmentation

## Problem Statement

DBSCAN clusters on XYZ position alone, so spatially distant points with identical
geometric character (e.g., flat brick path) get split into separate clusters. Adding
per-point eigenvalue features from local covariance matrices lets downstream clustering
group by *surface type* (flat, linear, scattered) rather than just proximity.

## Existing Codebase Map

### Point Type (`crates/pt-scan/src/types.rs`)

```rust
pub struct Point {
    pub position: [f32; 3],
    pub color: Option<[u8; 3]>,
}
```

No `Point3` type exists. Positions are `[f32; 3]` arrays everywhere. Nalgebra
`Vector3<f32>` is used only inside `ransac.rs` for plane fitting.

### KD-Tree Usage (`crates/pt-scan/src/filter.rs`, `cluster.rs`)

Both modules use `kiddo::ImmutableKdTree<f32, 3>`:
- **filter.rs** — `nearest_n::<SquaredEuclidean>()` for K-NN in outlier removal
- **cluster.rs** — `within::<SquaredEuclidean>()` for range queries in DBSCAN

Pattern: build tree from `Vec<[f32; 3]>` positions, query returns `NearestNeighbour`
items with `.item` (u64 index) and `.distance`. The `nearest_n` API takes a
`NonZeroUsize` for K.

### Nalgebra Usage (`crates/pt-scan/src/ransac.rs`)

Only `nalgebra::Vector3` used for cross products and dot products. The `Matrix3` and
`SymmetricEigen` types needed for covariance decomposition are available in nalgebra
0.34 but not yet imported anywhere.

### Feature Extraction (`crates/pt-scan/src/feature.rs`)

Existing `FeatureCandidate` computes per-*cluster* summaries (height, spread, density,
color, vertical profile). This is the BAML classifier input. Eigenvalue features are
per-*point* — a different level of abstraction. They feed into clustering (to improve
it) and could also enrich `FeatureCandidate` with aggregate stats.

### Clustering (`crates/pt-scan/src/cluster.rs`)

DBSCAN clusters `&[Point]` using spatial proximity only. `ClusterConfig` has `epsilon`
and `min_points`. The eigenvalue features will eventually augment the feature space
used for clustering, but T-033-03 only computes them — T-033-04 (HDBSCAN) will
consume them.

### Pipeline (`crates/pt-scan/src/lib.rs`)

`process_scan_timed()` runs: parse → downsample → outlier removal → RANSAC.
Clustering and feature extraction happen downstream (called separately by consumers).
Eigenvalue computation would slot between RANSAC and clustering.

### Dependencies

- `kiddo = "5"` — KD-tree, already in pt-scan deps
- `nalgebra = "0.34"` — linear algebra, already in pt-scan deps
- `rayon` — NOT in workspace. Ticket suggests it for parallelism.
- `approx = "0.5"` — float comparison, in dev-deps

### Test Patterns

- Unit tests use `pt_test_utils::timed` wrapper for 10s timeout
- Synthetic geometry (grids, blobs, cubes) for deterministic assertions
- `approx` crate for float tolerance (`assert!((a - b).abs() < eps)` pattern, not
  `assert_relative_eq!` — both are available)

## Constraints and Observations

1. **No rayon in workspace.** Adding it is straightforward but increases compile time.
   Alternative: compute features sequentially first, add rayon later if perf requires.

2. **nalgebra 3x3 eigendecomposition.** `SymmetricEigen::new(Matrix3::from(...))` is
   the right API. For 3x3 symmetric matrices this is O(1) (closed-form / Jacobi).
   No need for iterative solvers.

3. **Kiddo `nearest_n` returns sorted by distance.** We need the K nearest neighbors'
   positions to build the covariance matrix. The indices come from `.item`.

4. **Eigenvalue ordering.** nalgebra `SymmetricEigen` does NOT guarantee sorted
   eigenvalues. We must sort λ1 ≥ λ2 ≥ λ3 ourselves.

5. **Degenerate cases.** If K neighbors are collinear (rank-1) or coplanar (rank-2),
   some eigenvalues will be ≈ 0. λ1 could be 0 for coincident points. Must guard
   division by λ1.

6. **Performance target: < 2s for 122K points, K=20.** That's 122K × 20 = 2.4M
   neighbor lookups + 122K eigendecompositions. Kiddo K-NN on 122K points is ~μs
   per query. 122K × ~5μs ≈ 610ms for queries + ~1μs per 3x3 eigen ≈ 122ms.
   Total ~730ms single-threaded — well under 2s. Rayon likely unnecessary.

7. **Public API shape.** The ticket specifies:
   `compute_point_features(points: &[Point3], k: usize) -> Vec<PointFeatures>`
   But our point type is `Point` (with color), not `Point3`. We should accept
   `&[Point]` to match existing API conventions, or accept `&[[f32; 3]]` for
   flexibility.

8. **Downstream consumers.** T-033-04 (HDBSCAN) will use eigenvalue features to
   augment the clustering feature space. `FeatureCandidate` (BAML input) may
   eventually include aggregate eigenvalue stats per cluster.

## Files Relevant to This Ticket

| File | Role |
|------|------|
| `crates/pt-scan/src/types.rs` | Point type definition |
| `crates/pt-scan/src/lib.rs` | Module registration, pipeline, re-exports |
| `crates/pt-scan/src/feature.rs` | Existing per-cluster feature extraction |
| `crates/pt-scan/src/filter.rs` | KD-tree + K-NN pattern to follow |
| `crates/pt-scan/src/cluster.rs` | DBSCAN — downstream consumer of eigenvalue features |
| `crates/pt-scan/Cargo.toml` | Dependencies (kiddo, nalgebra already present) |
