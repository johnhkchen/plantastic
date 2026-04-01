# T-033-01 Structure: DBSCAN Clustering

## Files Created

### `crates/pt-scan/src/cluster.rs` (~120 lines)
New module implementing DBSCAN clustering.

**Public types:**
```rust
pub struct ClusterConfig {
    pub epsilon: f32,
    pub min_points: usize,
}

impl Default for ClusterConfig { ... }  // epsilon=0.3, min_points=50

pub struct Cluster {
    pub id: u32,
    pub point_indices: Vec<usize>,
    pub centroid: [f32; 3],
    pub bbox: BoundingBox,
}

pub struct ClusterResult {
    pub clusters: Vec<Cluster>,
    pub noise_indices: Vec<usize>,
}
```

**Public function:**
```rust
pub fn cluster_obstacles(points: &[Point], config: &ClusterConfig) -> ClusterResult
```

**Internal implementation:**
- Build `ImmutableKdTree<f32, 3>` from point positions
- Maintain `labels: Vec<Option<u32>>` (None = unvisited, Some(id) = cluster assignment)
- Maintain `visited: Vec<bool>` for DBSCAN traversal
- Use `tree.within::<SquaredEuclidean>(&pos, eps_sq)` for range queries
- After clustering, compute centroid and bbox per cluster
- Collect unlabeled points into `noise_indices`

## Files Modified

### `crates/pt-scan/src/lib.rs` (+3 lines)
- Add `pub mod cluster;`
- Add `pub use cluster::{Cluster, ClusterConfig, ClusterResult};` to re-exports

### `crates/pt-scan/src/types.rs` (+8 lines)
- Add `BoundingBox::from_positions(positions: &[[f32; 3]]) -> Option<Self>` method
  - Like `from_points()` but takes raw position arrays instead of `Point` structs
  - Used by cluster.rs to compute bbox from indexed positions without cloning Points

## Files Unchanged

- `Cargo.toml` — no new dependencies (kiddo already present)
- `lib.rs` pipeline functions — clustering is standalone, not wired into process_scan
- `report.rs` — no report changes in this ticket
- `tests/integration.rs` — existing tests untouched; new tests go in cluster.rs

## Module Boundaries

```
pt-scan/src/
├── lib.rs          ← adds `pub mod cluster` + re-exports
├── cluster.rs      ← NEW: DBSCAN implementation + unit tests
├── types.rs        ← adds BoundingBox::from_positions()
├── filter.rs       ← unchanged (k-d tree pattern reference)
├── ransac.rs       ← unchanged
├── parser.rs       ← unchanged
├── report.rs       ← unchanged
├── export.rs       ← unchanged
├── mesh.rs         ← unchanged
└── error.rs        ← unchanged
```

## Public Interface After This Ticket

```rust
// New exports from pt_scan
use pt_scan::{Cluster, ClusterConfig, ClusterResult};

// Usage
let cloud = pt_scan::process_scan(reader, &scan_config)?;
let cluster_config = ClusterConfig::default(); // eps=0.3, min_pts=50
let result = pt_scan::cluster::cluster_obstacles(&cloud.obstacles, &cluster_config);
// result.clusters: Vec<Cluster>
// result.noise_indices: Vec<usize>
```

## Test Organization

Unit tests in `cluster.rs` `#[cfg(test)] mod tests` block:
- `test_two_separated_clusters` — two blobs → two clusters
- `test_noise_not_merged` — sparse points between blobs → noise
- `test_single_cluster` — one blob → one cluster
- `test_empty_input` — empty → empty
- `test_cluster_metadata` — centroid and bbox correctness

Integration test for Powell & Market in `tests/integration.rs`:
- `test_powell_market_two_clusters` — real scan → exactly 2 clusters
- Requires `assets/scans/samples/Scan at 09.23.ply` (skipped if absent)
