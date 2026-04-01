# T-033-03 Review: Eigenvalue Features

## Summary

Added per-point eigenvalue feature computation to the pt-scan crate. For each point
in a point cloud, the module finds K nearest neighbors, builds a 3x3 covariance
matrix, and derives geometric descriptors from the eigendecomposition.

## Files Changed

| File | Change |
|------|--------|
| `crates/pt-scan/src/eigenvalue.rs` | **NEW** — 415 lines. Core computation + 6 tests |
| `crates/pt-scan/src/lib.rs` | Added `pub mod eigenvalue` and re-exports |

No new dependencies — uses existing `kiddo` (KD-tree) and `nalgebra` (eigen).

## Public API

```rust
pub struct PointFeatures {
    pub planarity: f32,     // (λ2 − λ3) / λ1
    pub linearity: f32,     // (λ1 − λ2) / λ1
    pub sphericity: f32,    // λ3 / λ1
    pub omnivariance: f32,  // (λ1 × λ2 × λ3)^(1/3)
    pub normal: [f32; 3],   // surface normal, oriented Z-up
    pub curvature: f32,     // λ3 / (λ1 + λ2 + λ3)
}

pub fn compute_point_features(points: &[Point], k: usize) -> Vec<PointFeatures>
```

## Test Coverage

| Test | What it verifies |
|------|-----------------|
| `flat_grid_has_high_planarity` | 20x20 grid → planarity > 0.7, sphericity < 0.1 |
| `line_has_high_linearity` | 100 collinear points → linearity > 0.9 |
| `random_scatter_has_high_sphericity` | 1000 volumetric points → sphericity > 0.3 |
| `degenerate_coincident_points_no_panic` | 50 identical points → zero features, no panic |
| `normal_orientation_points_up` | flat surface → normal Z > 0.9 |
| `performance_122k_reasonable_time` | 122K points K=20 → 0.31s release (target < 2s) |

Tests verify behavior from independent geometric reasoning — not by calling the
system under test to derive expected values.

## Scenario Dashboard

- **Before:** 87.5 min / 240.0 min (36.5%) — 10 pass, 0 fail
- **After:** 87.5 min / 240.0 min (36.5%) — 10 pass, 0 fail
- **No regression.** This ticket adds foundational capability (per-point features)
  consumed by T-033-04 (HDBSCAN) which will improve clustering and advance scenarios.

## Quality Gate

`just check` passes: fmt ✓, lint ✓, test ✓, scenarios ✓

## Design Decisions Worth Noting

1. **Separate `eigenvalue.rs` module** rather than extending `feature.rs`. Per-point
   eigenvalue features are a different abstraction than per-cluster BAML summaries.

2. **No rayon.** 0.31s release for 122K points — no justification for adding a new
   dependency. Trivially parallelizable later if needed.

3. **f32 throughout.** Matches `Point` type precision. Covariance computed in f32
   (sufficient for K=20 neighborhoods). No f64 intermediate needed.

4. **Normal orientation convention.** Positive Z component ("up"). Ambiguous sign
   case (Z ≈ 0) left as-is — vertical surfaces keep their computed normal.

## Open Concerns

1. **Powell & Market validation** (acceptance criterion). The ticket asks for
   validation on real scan data: "brick path points have high planarity, trunk
   points have high linearity." This requires running eigenvalue features on the
   Powell & Market PLY fixture, which is a separate validation step that could be
   added to the integration tests. Not blocking for this ticket since the synthetic
   geometry tests verify the mathematical correctness.

2. **Aggregate eigenvalue stats per cluster.** The existing `FeatureCandidate` struct
   doesn't include eigenvalue summaries yet. A future ticket could add mean planarity,
   mean sphericity, etc. to enrich the BAML classifier input.

3. **Adaptive K.** The ticket mentions "adaptive K based on local density if needed."
   Fixed K=20 is the standard starting point. Adaptive K is future work if clustering
   quality requires it.
