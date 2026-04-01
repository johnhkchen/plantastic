# T-033-02 Structure: Feature Candidates

## Files Created

### `crates/pt-scan/src/feature.rs` (~200 lines)

New module. Contains:

```
// Types
pub struct FeatureCandidate { ... }    // 10 fields per ticket AC

// Public API
pub fn extract_candidates(
    clusters: &[Cluster],
    points: &[Point],
    ground_plane: &Plane,
) -> Vec<FeatureCandidate>

// Internal helpers
fn compute_height_ft(points: &[Point], indices: &[usize], plane: &Plane) -> f64
fn compute_spread_ft(bbox: &BoundingBox) -> f64
fn compute_density(point_count: usize, bbox: &BoundingBox) -> f64
fn classify_color(points: &[Point], indices: &[usize]) -> String
fn classify_vertical_profile(height_ft: f64, spread_ft: f64, points: &[Point], indices: &[usize], bbox: &BoundingBox) -> String

// Unit tests (cfg(test))
mod tests { ... }
```

## Files Modified

### `crates/pt-scan/src/lib.rs`

- Add `pub mod feature;` declaration
- Add `pub use feature::{FeatureCandidate, extract_candidates};` to exports

### `crates/pt-scan/examples/process_sample.rs`

- Import `cluster_obstacles, ClusterConfig, extract_candidates`
- After terrain generation, add stage 6:
  - Cluster obstacles with default config
  - Extract candidates
  - Print formatted table (cluster_id, height_ft, spread_ft, point_count,
    dominant_color, vertical_profile, density)

### `crates/pt-scan/tests/integration.rs`

- Add `test_feature_candidates_synthetic` — synthetic PLY → full pipeline → candidates
- Add `test_powell_market_candidates` — real scan → candidates with field validation

## Module Boundaries

```
cluster.rs                    feature.rs
┌─────────────────────┐      ┌──────────────────────────┐
│ cluster_obstacles()  │─────>│ extract_candidates()     │
│ -> ClusterResult     │      │ -> Vec<FeatureCandidate>  │
│   .clusters: [Cluster]      │                          │
│   .noise_indices     │      │ Internal:                │
└─────────────────────┘      │  compute_height_ft       │
                              │  compute_spread_ft       │
types.rs                      │  compute_density         │
┌─────────────────────┐      │  classify_color          │
│ Point                │─────>│  classify_vertical_profile│
│ Plane                │      └──────────────────────────┘
│ BoundingBox          │
└─────────────────────┘
```

`feature.rs` depends on `cluster.rs` types (Cluster) and `types.rs` (Point, Plane,
BoundingBox). No reverse dependency — cluster.rs does not know about features.

## Public Interface Changes

New exports from `pt_scan`:
- `FeatureCandidate` — struct, Serialize
- `extract_candidates` — function

No changes to existing public API. Fully additive.

## Ordering

1. Create `feature.rs` with struct + function + helpers + unit tests
2. Wire into `lib.rs` (module declaration + re-exports)
3. Extend CLI example
4. Add integration tests
5. Run `just check`
