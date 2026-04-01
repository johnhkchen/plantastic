---
id: T-033-03
story: S-033
title: eigenvalue-features
type: task
status: open
priority: high
phase: implement
depends_on: [T-033-01]
---

## Context

DBSCAN over-segments because it clusters on XYZ alone. Points on a flat brick path that are 30cm apart get split into separate clusters, even though they have identical geometric character (flat, brown, same normal). Adding eigenvalue features from local covariance matrices lets clustering group by *what kind of surface* a point is, not just where it is.

## Background

Weinmann et al. (2015) established that three eigenvalue ratios from the local covariance matrix reliably distinguish surface types in point clouds:
- **Planarity** = (λ2 − λ3) / λ1 — high for flat surfaces (patios, paths, walls)
- **Linearity** = (λ1 − λ2) / λ1 — high for edges, poles, fences, wires
- **Sphericity** = λ3 / λ1 — high for vegetation, scattered debris
- **Omnivariance** = (λ1 × λ2 × λ3)^(1/3) — higher for vegetation than hardscape
- **Normal direction** — smallest eigenvector = surface normal

These are computed per-point from K nearest neighbors. For 122K points with K=20, that's 2.4M neighbor lookups — fast with a KD-tree.

## Acceptance Criteria

- Add `features` module to pt-scan (or extend existing)
- `compute_point_features(points: &[Point3], k: usize) -> Vec<PointFeatures>`
- PointFeatures struct: planarity, linearity, sphericity, omnivariance, normal ([f32; 3]), curvature
- Uses kiddo 5 `ImmutableKdTree::nearest_n` (already a dep) for K-NN queries
- Uses nalgebra 0.34 `SymmetricEigen::new` (already a dep) for 3×3 eigendecomposition
- Uses rayon (added to workspace) for parallel per-point computation
- Performance: < 2s for 122K points (K=20) on M-series Mac
- Unit tests:
  - Flat grid → planarity ≈ 1.0, sphericity ≈ 0.0
  - Points on a line → linearity ≈ 1.0
  - Random sphere → sphericity ≈ 1.0
- Powell & Market validation: brick path points have high planarity, trunk points have high linearity

## Implementation Notes

- The 3×3 symmetric eigendecomposition is `nalgebra::SymmetricEigen::new(matrix)`
- Sort eigenvalues: λ1 ≥ λ2 ≥ λ3 (largest first)
- K=20-30 is standard; use adaptive K based on local density if needed
- Per-point features are the input to both improved clustering AND the feature candidate summaries sent to BAML
- Consider rayon for parallel feature computation (each point is independent)
