# T-033-03 Design: Eigenvalue Features

## Decision Summary

Add a new `eigenvalue` module to pt-scan that computes per-point geometric features
from local covariance matrices. Sequential (no rayon). Accepts `&[Point]` positions.
Returns `Vec<PointFeatures>` with planarity, linearity, sphericity, omnivariance,
normal, and curvature.

## Options Evaluated

### Option A: New `eigenvalue` module in pt-scan (CHOSEN)

Add `crates/pt-scan/src/eigenvalue.rs` with:
- `PointFeatures` struct
- `compute_point_features(points: &[Point], k: usize) -> Vec<PointFeatures>`

**Pros:**
- Keeps per-point geometric features separate from per-cluster feature candidates
- `feature.rs` stays focused on cluster-level BAML summaries
- Clear module boundary: eigenvalue.rs = per-point geometry, feature.rs = per-cluster
- No new crate overhead; kiddo + nalgebra already in deps

**Cons:**
- Two "feature" modules could confuse naming. Mitigated by clear naming: `eigenvalue`
  vs `feature`.

### Option B: Extend existing `feature.rs`

Add eigenvalue computation inside the existing feature module.

**Rejected:** feature.rs is about per-cluster summaries for the BAML classifier.
Per-point eigenvalue features are a different abstraction level (computed before
clustering, consumed by clustering). Mixing them violates the module's single
responsibility.

### Option C: New `pt-eigen` crate

Create a standalone crate for eigenvalue features.

**Rejected:** Overkill. The code is ~150 lines, uses the same deps as pt-scan,
and is only consumed within pt-scan's pipeline. A new crate adds Cargo.toml
boilerplate, cross-crate dep management, and workspace churn for no benefit.

### Option D: Add rayon for parallel computation

Use `par_iter()` for the per-point feature loop.

**Rejected for now:** Performance estimate from research: ~730ms single-threaded
for 122K points. Well under the 2s target. Rayon adds ~30s compile time to the
workspace. Not justified until profiling shows a need. The function signature
accepts `&[Point]` and returns `Vec<PointFeatures>` — trivially parallelizable
later by swapping `iter().map()` for `par_iter().map()`.

## Key Design Decisions

### 1. Input type: `&[Point]` not `&[[f32; 3]]`

The ticket says `&[Point3]` but our type is `Point`. Using `&[Point]` keeps the
API consistent with `cluster_obstacles()`, `extract_candidates()`, etc. Internally
we extract positions into `Vec<[f32; 3]>` for the KD-tree, same pattern as filter.rs.

### 2. Eigenvalue computation via nalgebra `SymmetricEigen`

For each point:
1. Find K nearest neighbors via kiddo `nearest_n`
2. Compute centroid of neighbors
3. Build 3x3 covariance matrix from centered neighbors
4. Decompose with `SymmetricEigen::new()`
5. Sort eigenvalues descending: λ1 ≥ λ2 ≥ λ3
6. Compute ratios

Alternative considered: hand-coded 3x3 eigendecomposition (Cardano's formula).
Rejected — nalgebra is already a dep, well-tested, and the 3x3 case is fast.

### 3. Degenerate eigenvalue handling

When λ1 ≈ 0 (coincident or near-coincident neighbors), all ratios are undefined.
Strategy: if λ1 < ε (1e-10), set all ratios to 0.0 and normal to [0, 0, 1].
This handles edge cases without NaN propagation.

### 4. Normal orientation

The smallest eigenvector gives the surface normal direction, but its sign is
ambiguous (could point up or down). Convention: orient normals so they have a
positive Z component (pointing "up"). If Z ≈ 0, keep as-is.

### 5. PointFeatures struct

```rust
pub struct PointFeatures {
    pub planarity: f32,     // (λ2 - λ3) / λ1
    pub linearity: f32,     // (λ1 - λ2) / λ1
    pub sphericity: f32,    // λ3 / λ1
    pub omnivariance: f32,  // (λ1 * λ2 * λ3)^(1/3)
    pub normal: [f32; 3],   // surface normal (smallest eigenvector)
    pub curvature: f32,     // λ3 / (λ1 + λ2 + λ3)
}
```

All f32 — consistent with Point positions, sufficient precision for downstream
clustering, and half the memory of f64 (matters at 122K points).

### 6. K parameter

Accept as argument, default recommendation K=20. Caller decides. No adaptive K
in this ticket — that's future work if needed.

## Testing Strategy

Three synthetic geometry tests per acceptance criteria:
1. **Flat grid** — 20x20 grid on XY plane, K=20 → planarity ≈ 1.0, sphericity ≈ 0.0
2. **Line** — 100 points on X axis, K=20 → linearity ≈ 1.0
3. **Sphere** — 200 points on unit sphere surface, K=20 → sphericity high

Plus:
4. **Degenerate input** — coincident points → no panics, zero features
5. **Performance** — 122K random points, K=20, assert < 2s
6. **Normal orientation** — flat horizontal surface → normal Z > 0
