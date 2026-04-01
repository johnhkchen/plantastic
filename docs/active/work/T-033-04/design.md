# T-033-04 Design: HDBSCAN Clustering

## Decision: Thin Wrapper Around `hdbscan` Crate

Add a new public function `hdbscan_cluster()` alongside the existing `cluster_obstacles()`. The new function builds a normalized 6D feature vector from points + eigenvalue features, delegates to the `hdbscan` crate, and post-processes labels into `ClusterResult`.

## Options Evaluated

### Option A: Implement HDBSCAN from scratch
- **Pro:** Full control, can expose dendrogram, can use our kiddo KD-tree.
- **Con:** 500+ lines of MST construction, condensed tree, excess-of-mass extraction. Significant testing burden. The algorithm is well-understood but fiddly to get right.
- **Rejected:** The crate is already in the workspace, pure Rust, and handles the hard parts. Rolling our own adds risk with no near-term benefit.

### Option B: Thin wrapper around `hdbscan` crate ← CHOSEN
- **Pro:** ~100 lines of new code. Feature vector construction + label-to-ClusterResult conversion. Leverages tested implementation.
- **Pro:** Config struct maps directly to `HdbscanHyperParams::builder()`.
- **Con:** No dendrogram access (crate doesn't expose it). f32→f64 conversion overhead.
- **Chosen because:** Minimum code for maximum value. Dendrogram is explicitly future work per ticket.

### Option C: Replace `cluster_obstacles` entirely
- **Pro:** Single clustering path, less code to maintain.
- **Con:** Breaks existing callers. HDBSCAN requires precomputed PointFeatures, which DBSCAN doesn't. Some callers may not have features.
- **Rejected:** The two functions serve different needs. DBSCAN is simpler and useful when eigenvalue features aren't available.

## Architecture

### New Types

```rust
pub struct HdbscanConfig {
    pub min_cluster_size: usize,   // default: 200
    pub min_samples: usize,        // default: 10
    pub spatial_weight: f64,       // default: 1.0 — weight for normalized XYZ vs features
}
```

Why these defaults:
- `min_cluster_size: 200` — landscape features (trees, structures) have hundreds of points at our voxel density. 100 is too small (picks up noise clusters); 500 is too large (misses small features).
- `min_samples: 10` — controls core distance smoothing. Lower = more clusters, higher = more conservative. 10 is the ticket's suggested default.
- `spatial_weight: 1.0` — equal weight between spatial and feature dimensions. Tunable per the ticket's guidance.

### New Function

```rust
pub fn hdbscan_cluster(
    points: &[Point],
    features: &[PointFeatures],
    config: &HdbscanConfig,
) -> ClusterResult
```

### Feature Vector Construction

For each point `i`, build a 6D vector:
```
[x_norm * w, y_norm * w, z_norm * w, planarity, linearity, sphericity]
```

Where:
- `x_norm, y_norm, z_norm` = spatial coords normalized to [0, 1] range (min-max normalization)
- `w` = `config.spatial_weight` — scaling factor for spatial dimensions
- `planarity, linearity, sphericity` already in [0, 1] by definition

**Why min-max normalization:** Eigenvalue features are bounded [0, 1]. Spatial coords span the scan extent (typically 5–20m). Min-max maps them to [0, 1] to match. Standard normalization (z-score) would give spatial coords different variance characteristics depending on scan shape — min-max is simpler and more predictable.

**Why only 3 eigenvalue features (not 6):** Omnivariance, normal, and curvature are correlated with planarity/linearity/sphericity. Adding them would increase dimensionality without independent information. The three ratios (planarity + linearity + sphericity ≈ 1) span the eigenvalue feature space.

### Label-to-ClusterResult Conversion

1. Iterate `Vec<i32>` labels from hdbscan crate
2. Group point indices by label (skip -1 = noise)
3. For each cluster: compute centroid and bbox from member positions
4. Collect -1 indices into `noise_indices`
5. Sort clusters by id (label value)

This reuses `compute_centroid()` from the existing cluster module (needs to be made `pub(crate)` or extracted).

### Error Handling

- `points.is_empty()` → return empty `ClusterResult` (match existing DBSCAN behavior)
- `points.len() != features.len()` → panic with assert (programming error, not runtime)
- hdbscan crate returns `HdbscanError` → all points become noise (graceful fallback per acceptance criteria)

### Integration with Pipeline

The example and callers choose between DBSCAN and HDBSCAN at call site. No change to `extract_candidates` or downstream — both return `ClusterResult`.

## Testing Strategy

1. **Unit tests (mirror DBSCAN tests):**
   - Two separated blobs → 2 clusters
   - Noise points not merged → noise in `noise_indices`
   - Single cluster → 1 cluster
   - Empty input → empty result
   - Features length mismatch → panic

2. **Feature space validation:**
   - Two blobs with same geometry but different positions → 2 clusters (spatial separates)
   - Two overlapping blobs with different features (one planar, one spherical) → 2 clusters (features separate)

3. **Powell & Market integration test:**
   - Should produce 2–4 clusters (vs 12+ with DBSCAN)
   - Two largest clusters ≥ 500 points

4. **Performance test:**
   - 122K points < 1s (release), generous limit in debug

## Open Questions

- **Spatial weight tuning:** Starting with 1.0 (equal weight). May need adjustment after Powell & Market validation. The config makes this tunable without code changes.
- **max_cluster_size:** Not setting (unlimited by default). Landscape scans don't have a natural upper bound on feature size.
