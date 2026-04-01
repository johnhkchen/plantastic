# T-033-04 Structure: HDBSCAN Clustering

## Files Modified

### `crates/pt-scan/src/cluster.rs`

**Changes:**
1. Add `HdbscanConfig` struct with `min_cluster_size`, `min_samples`, `spatial_weight` fields + `Default` impl
2. Add `hdbscan_cluster(points: &[Point], features: &[PointFeatures], config: &HdbscanConfig) -> ClusterResult` public function
3. Add private helper `build_feature_vectors(points, features, spatial_weight) -> Vec<Vec<f64>>`
4. Add private helper `labels_to_cluster_result(labels: &[i32], points: &[Point]) -> ClusterResult`
5. Make `compute_centroid` `pub(crate)` (currently private, needed by conversion logic — but actually it stays in the same file, so no visibility change needed)
6. Add new unit tests in the existing `#[cfg(test)] mod tests` block

**New imports:** `use hdbscan::{Hdbscan, HdbscanHyperParams};` and `use crate::eigenvalue::PointFeatures;`

### `crates/pt-scan/src/lib.rs`

**Changes:**
1. Add `HdbscanConfig` and `hdbscan_cluster` to the `pub use cluster::` re-export line

### `crates/pt-scan/tests/integration.rs`

**Changes:**
1. Add `test_powell_market_hdbscan` integration test — runs HDBSCAN on the real scan, validates 2–4 clusters

## Files NOT Modified

- `eigenvalue.rs` — no changes, used as-is
- `feature.rs` — no changes, consumes `ClusterResult` unchanged
- `types.rs` — no changes
- `Cargo.toml` — `hdbscan` dependency already present
- `examples/process_sample.rs` — not changing the example (it uses DBSCAN; callers choose)

## Module Boundaries

```
cluster.rs
├── ClusterConfig          (existing, unchanged)
├── Cluster                (existing, unchanged)
├── ClusterResult          (existing, unchanged)
├── cluster_obstacles()    (existing, unchanged)
├── HdbscanConfig          (NEW)
├── hdbscan_cluster()      (NEW)
├── build_feature_vectors() (NEW, private)
├── labels_to_cluster_result() (NEW, private)
├── compute_centroid()     (existing, private, reused internally)
└── range_query()          (existing, private, unchanged)
```

## Public Interface Additions

```rust
// New config type
pub struct HdbscanConfig {
    pub min_cluster_size: usize,  // default: 200
    pub min_samples: usize,       // default: 10
    pub spatial_weight: f64,      // default: 1.0
}

// New clustering function
pub fn hdbscan_cluster(
    points: &[Point],
    features: &[PointFeatures],
    config: &HdbscanConfig,
) -> ClusterResult;
```

Re-exported from `lib.rs` alongside existing cluster types.

## Data Flow

```
points: &[Point]  ──────────────────────┐
                                         ├─→ build_feature_vectors() ──→ Vec<Vec<f64>>
features: &[PointFeatures] ────────────┘                                      │
                                                                               ▼
config: &HdbscanConfig ──→ HdbscanHyperParams ──→ Hdbscan::new() ──→ .cluster()
                                                                               │
                                                                               ▼
                                                                         Vec<i32> labels
                                                                               │
points: &[Point] ─────────────────────────────────→ labels_to_cluster_result() │
                                                                               ▼
                                                                         ClusterResult
```

## Test Structure

Unit tests added to `cluster.rs::tests`:
- `test_hdbscan_two_separated_clusters` — 2 blobs with distinct features
- `test_hdbscan_noise_not_merged` — blobs + isolated noise
- `test_hdbscan_single_cluster` — uniform blob
- `test_hdbscan_empty_input` — empty slice
- `test_hdbscan_feature_separation` — overlapping positions, different features → 2 clusters
- `test_hdbscan_all_noise_fallback` — insufficient density → empty clusters

Integration test in `tests/integration.rs`:
- `test_powell_market_hdbscan` — real scan, expects 2–4 clusters, ≥500 pts each
