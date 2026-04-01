# T-033-04 Plan: HDBSCAN Clustering

## Step 1: Add `HdbscanConfig` and `hdbscan_cluster` to `cluster.rs`

1. Add imports: `hdbscan::{Hdbscan, HdbscanHyperParams}`, `crate::eigenvalue::PointFeatures`
2. Add `HdbscanConfig` struct with `Default` impl
3. Add `build_feature_vectors()` private function:
   - Compute min/max for each spatial dimension
   - Normalize XYZ to [0, 1], multiply by `spatial_weight`
   - Append planarity, linearity, sphericity
   - Return `Vec<Vec<f64>>`
4. Add `labels_to_cluster_result()` private function:
   - Group indices by label (skip -1)
   - Build `Cluster` structs with centroid + bbox
   - Collect noise indices
5. Add `hdbscan_cluster()` public function:
   - Assert `points.len() == features.len()`
   - Handle empty input
   - Build feature vectors
   - Construct `HdbscanHyperParams` from config
   - Call `Hdbscan::new(&data, params).cluster()`
   - Convert labels to `ClusterResult` (fallback to all-noise on error)

**Verification:** Compiles with `cargo check -p pt-scan`

## Step 2: Update `lib.rs` re-exports

Add `HdbscanConfig` and `hdbscan_cluster` to the `pub use cluster::` line.

**Verification:** `cargo check -p pt-scan`

## Step 3: Add unit tests

Add to `cluster.rs::tests`:
- `test_hdbscan_two_separated_clusters` — two 200-pt blobs at [0,0,0] and [5,5,5] with uniform features → 2 clusters
- `test_hdbscan_noise_not_merged` — blobs + sparse noise → 2 clusters, noise in `noise_indices`
- `test_hdbscan_single_cluster` — one 400-pt blob → 1 cluster
- `test_hdbscan_empty_input` — empty → empty result
- `test_hdbscan_feature_separation` — two interleaved blobs at same position, one with planarity=1.0 and one with sphericity=1.0 → 2 clusters (proves feature space works)
- `test_hdbscan_all_noise_fallback` — 5 scattered points, min_cluster_size=100 → 0 clusters, all noise

**Verification:** `cargo test -p pt-scan -- hdbscan`

## Step 4: Add Powell & Market integration test

Add `test_powell_market_hdbscan` to `tests/integration.rs`:
1. Load scan from `assets/scans/samples/Scan at 09.23.ply`
2. Process through pipeline (parse, downsample, RANSAC)
3. Compute eigenvalue features on obstacle points
4. Run `hdbscan_cluster()` with default config
5. Assert 2–4 clusters, two largest ≥500 points
6. Print cluster info for debugging

**Verification:** `cargo test -p pt-scan -- powell_market_hdbscan`

## Step 5: Run quality gate

- `cargo fmt --check -p pt-scan`
- `cargo clippy -p pt-scan -- -D warnings`
- `cargo test -p pt-scan`

Fix any issues.

## Step 6: Run scenario dashboard

`cargo run -p pt-scenarios` — verify no regressions.

## Testing Strategy

- **Unit tests:** Synthetic blobs with known geometry. Each test uses `pt_test_utils::timed()`.
- **Integration test:** Real Powell & Market scan — the ground truth validation for "does HDBSCAN produce 2–4 clusters instead of 12?"
- **Performance:** The integration test on 122K points implicitly validates <1s (the `timed()` wrapper enforces 10s, and release should be well under 1s).
- **No mocking:** All tests use real implementations per project policy.
