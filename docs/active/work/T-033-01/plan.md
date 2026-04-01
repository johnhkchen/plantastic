# T-033-01 Plan: DBSCAN Clustering

## Step 1: Add `BoundingBox::from_positions()` to types.rs

Add a method that computes a bounding box from `&[[f32; 3]]` (raw position arrays). This avoids requiring full `Point` structs when we only have positions indexed from a slice.

**Verification:** Existing tests still pass. No new test needed — the existing `from_points()` tests cover the logic; this is just a type-level convenience.

## Step 2: Create `cluster.rs` with types and DBSCAN implementation

Write the full module:
1. Define `ClusterConfig` with `Default` impl (epsilon=0.3, min_points=50)
2. Define `Cluster` struct (id, point_indices, centroid, bbox)
3. Define `ClusterResult` struct (clusters, noise_indices)
4. Implement `cluster_obstacles(points: &[Point], config: &ClusterConfig) -> ClusterResult`
   - Early return for empty input
   - Build k-d tree from positions
   - DBSCAN loop: iterate all points, range query, expand clusters
   - Post-process: compute centroid and bbox per cluster
   - Collect noise indices
5. Add serde derives to ClusterConfig, Cluster, ClusterResult for downstream serialization

**Verification:** Module compiles, `cargo clippy` clean.

## Step 3: Wire module into lib.rs

- Add `pub mod cluster;`
- Add re-exports: `pub use cluster::{Cluster, ClusterConfig, ClusterResult};`

**Verification:** `cargo build -p pt-scan`

## Step 4: Write unit tests

In `cluster.rs` `#[cfg(test)] mod tests`:

1. `test_two_separated_clusters`: 100 points near (0,0,0) + 100 points near (5,5,5), eps=0.5, min_pts=3 → exactly 2 clusters, each ~100 points
2. `test_noise_not_merged`: Two blobs + 5 scattered points between → 2 clusters + 5 noise
3. `test_single_cluster`: 200 points near (0,0,0) → 1 cluster
4. `test_empty_input`: no points → 0 clusters, 0 noise
5. `test_cluster_metadata`: verify centroid is average of member positions, bbox encloses all members

All tests use `pt_test_utils::timed` wrapper.

**Verification:** `cargo test -p pt-scan` all pass.

## Step 5: Write Powell & Market integration test

In `tests/integration.rs`:
- `test_powell_market_two_clusters`: load real PLY, run full pipeline, cluster obstacles, assert 2 clusters
- Gate on file existence: skip if `assets/scans/samples/Scan at 09.23.ply` is absent
- May need to tune epsilon/min_points from defaults if the real scan has different density

**Verification:** Test passes with the real scan file present.

## Step 6: Lint + format + full quality gate

- `just fmt`
- `just lint`
- `just test`
- `just scenarios`

**Verification:** `just check` passes with no regressions.
