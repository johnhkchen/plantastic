# T-036-01 Structure: Measure Feature Gaps

## Files Modified

### `crates/pt-scan/src/gap.rs` (NEW)

New module — ~120 lines. Contains:

```
pub struct Gap               // 8 fields: feature_a_id, feature_b_id, centroid_distance_ft,
                             // clear_width_ft, clear_length_ft, area_sqft,
                             // ground_elevation_ft, midpoint
pub struct GapConfig         // 1 field: max_distance_ft (default: 30.0)
pub fn measure_gaps(         // &[FeatureCandidate], &Plane, &GapConfig -> Vec<Gap>
    candidates, ground_plane, config
) -> Vec<Gap>

// Private helpers:
fn centroid_distance_2d_ft(a, b) -> f64
fn ground_elevation_ft(mx, my, plane) -> f64

#[cfg(test)] mod tests       // ~100 lines
  - test_two_features_known_gap      // ticket's unit test case
  - test_overlapping_features        // clear_width ≤ 0 → no gap
  - test_beyond_threshold            // distance > max → filtered out
  - test_empty_candidates            // 0 candidates → empty result
  - test_single_candidate            // 1 candidate → no pairs → empty
  - test_ground_elevation            // verifies elevation computation
  - test_three_features_pairwise     // 3 features → up to 3 gaps
```

### `crates/pt-scan/src/lib.rs` (MODIFIED)

Add module declaration and re-exports:
```rust
pub mod gap;
pub use gap::{measure_gaps, Gap, GapConfig};
```

### `crates/pt-scan/tests/integration.rs` (MODIFIED)

Add integration test using synthetic PLY data:
- Process scan → cluster → extract candidates → measure gaps
- Verify gap count and dimensions for known geometry

### `crates/pt-scan/examples/process_sample.rs` (MODIFIED)

Add stage 7: gap measurement after feature extraction.
Print gap table showing feature pairs, distances, clear widths, and areas.

## Module Boundaries

```
cluster.rs ──→ feature.rs ──→ gap.rs
  Cluster        FeatureCandidate    Gap
                                     GapConfig

types.rs provides: Plane, Point, BoundingBox
```

`gap.rs` depends on:
- `crate::feature::FeatureCandidate` — input data
- `crate::types::Plane` — ground plane for elevation
- `serde::Serialize` — JSON output
- No new external dependencies

## Interface Contract

```rust
// Input: candidates from extract_candidates(), ground plane from scan metadata
let gaps = measure_gaps(&candidates, &cloud.metadata.ground_plane, &GapConfig::default());

// Output: Vec<Gap> sorted by centroid_distance_ft ascending
// Filtered: only pairs within max_distance_ft with positive clear_width
```

## Constants

- `M_TO_FT: f64 = 3.28084` — already in feature.rs, will define locally in gap.rs
  (same pattern — each module is self-contained)

## Ordering of Changes

1. Create `gap.rs` with types + function + tests
2. Register module in `lib.rs` with re-exports
3. Add integration test in `tests/integration.rs`
4. Update `examples/process_sample.rs` with gap table output
5. Run `just check` to validate
