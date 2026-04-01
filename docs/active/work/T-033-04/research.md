# T-033-04 Research: HDBSCAN Clustering

## Problem Statement

DBSCAN with a single epsilon (0.3m) over-segments the Powell & Market scan into 12+ clusters when 2–4 are expected. HDBSCAN eliminates the epsilon sensitivity by evaluating cluster stability across all epsilon values.

## Current Clustering Implementation

### `crates/pt-scan/src/cluster.rs` (362 lines)

**Public API:**
- `ClusterConfig { epsilon: f32, min_points: usize }` — defaults: 0.3m, 50 pts
- `Cluster { id: u32, point_indices: Vec<usize>, centroid: [f32; 3], bbox: BoundingBox }`
- `ClusterResult { clusters: Vec<Cluster>, noise_indices: Vec<usize> }`
- `cluster_obstacles(points: &[Point], config: &ClusterConfig) -> ClusterResult`

**Algorithm:** Standard DBSCAN with `kiddo::ImmutableKdTree<f32, 3>` for range queries. BFS expansion from core points. Operates in pure XYZ space — no feature augmentation.

**Internal helpers (reusable):**
- `compute_centroid(positions: &[[f32; 3]]) -> [f32; 3]` — mean position
- `range_query(tree, pos, eps_sq) -> Vec<usize>` — KD-tree range search

### `crates/pt-scan/src/types.rs`

- `Point { position: [f32; 3], color: Option<[u8; 3]> }` — the actual type (ticket says "Point3" but it's `Point`)
- `BoundingBox { min: [f32; 3], max: [f32; 3] }` with `from_positions(&[[f32; 3]])` and `from_points(&[Point])`

### `crates/pt-scan/src/eigenvalue.rs`

- `PointFeatures { planarity, linearity, sphericity, omnivariance, normal, curvature }` — all f32
- `compute_point_features(points: &[Point], k: usize) -> Vec<PointFeatures>` — K-NN covariance eigendecomposition
- Already uses `kiddo::ImmutableKdTree` and `nalgebra::SymmetricEigen`

## The `hdbscan` Crate (v0.12)

**Already in workspace Cargo.toml and pt-scan's Cargo.toml.**

### API

```rust
// Constructor
Hdbscan::new(data: &[Vec<T>], hyper_params: HdbscanHyperParams) -> Hdbscan<T>
Hdbscan::default_hyper_params(data: &[Vec<T>]) -> Hdbscan<T>

// Clustering
fn cluster(&self) -> Result<Vec<i32>, HdbscanError>
// Returns: positive i32 = cluster label, -1 = noise

// Hyper parameters (via builder)
HdbscanHyperParams::builder()
    .min_cluster_size(usize)    // minimum points per cluster
    .min_samples(usize)         // core distance K
    .max_cluster_size(usize)    // upper bound
    .epsilon(f64)               // DBSCAN-style lower bound (0.0 = pure HDBSCAN)
    .dist_metric(DistanceMetric)
    .nn_algorithm(NnAlgorithm)
    .allow_single_cluster(bool)
    .build()
```

**Input format:** `Vec<Vec<f64>>` — each inner vec is a feature vector. Generic over `Float`.

**Output format:** `Vec<i32>` — label per point. -1 = noise, 0+ = cluster ID.

**Distance metrics available:** Euclidean (default), Manhattan, others via `DistanceMetric` enum.

## Pipeline Integration Points

### Current pipeline flow (from `lib.rs` and `examples/process_sample.rs`):
1. Parse PLY → raw points
2. Voxel downsample
3. Outlier removal
4. RANSAC ground fitting → `PointCloud { ground, obstacles }`
5. **Clustering:** `cluster_obstacles(&cloud.obstacles, &ClusterConfig) -> ClusterResult`
6. Feature extraction: `extract_candidates(&clusters, &obstacles, &ground_plane) -> Vec<FeatureCandidate>`
7. Gap measurement
8. Annotated plan view

The clustering step (5) is the insertion point. The function takes `&[Point]` (obstacle points) and returns `ClusterResult`. Downstream code (`extract_candidates`, `measure_gaps`, `annotate_plan_view_png`) consumes `ClusterResult` — so the return type must stay compatible.

### Callers of `cluster_obstacles`:
- `examples/process_sample.rs:129-130` — direct call
- `tests/integration.rs` — `test_powell_market_two_clusters`, `test_feature_candidates_synthetic`

## Existing Tests

### Unit tests in `cluster.rs` (5 tests):
- `test_two_separated_clusters` — 2 blobs of 100 pts, epsilon=0.5, min_points=3
- `test_noise_not_merged` — 2 blobs + 5 isolated noise points
- `test_single_cluster` — 200 pts in one blob
- `test_empty_input` — empty slice
- `test_cluster_metadata` — centroid and bbox validation

### Integration test:
- `test_powell_market_two_clusters` — real scan (122K pts), expects ≥2 clusters with ≥500 pts each

## Feature Space Considerations

The ticket requires clustering in augmented feature space: `[x, y, z, planarity, linearity, sphericity]` (6D). Key considerations:

1. **Normalization:** Spatial coords are in meters (range ~0–20m for a scan). Eigenvalue features are in [0, 1]. Without normalization, XYZ dominates.
2. **Weighting:** A configurable spatial weight factor controls the balance. Too much XYZ weight = over-segmentation (same as DBSCAN). Too much feature weight = merges distant objects with similar geometry.
3. **Type conversion:** Points are `f32`, hdbscan crate wants `f64`. Conversion is straightforward but allocation-heavy for 122K × 6D = 732K f64 values (~5.7 MB).

## Performance Constraints

- Target: <1s for 122K points on M-series Mac
- The hdbscan crate uses its own KD-tree internally — no need to pass our kiddo tree
- Feature vector construction is O(n) and trivial
- The bottleneck will be the hdbscan crate's internal MST construction

## Constraints and Risks

1. **hdbscan crate maturity:** v0.12, pure Rust, no ndarray dependency. Already in workspace.
2. **API mismatch:** Crate returns `Vec<i32>`, we need `ClusterResult` with `Cluster` structs (centroids, bboxes). Post-processing needed.
3. **No dendrogram output:** The crate's `cluster()` returns flat labels only. The ticket mentions dendrogram as "valuable output" but marks it as future work for interactive viewer. Not blocking.
4. **Fallback:** If all points are noise (-1), return empty `ClusterResult` with all indices in `noise_indices`.
5. **Backward compatibility:** Existing `cluster_obstacles` callers need to keep working. New function is additive.
