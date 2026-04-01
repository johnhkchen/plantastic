# T-033-03 Structure: Eigenvalue Features

## Files Modified

### `crates/pt-scan/src/lib.rs`
- Add `pub mod eigenvalue;`
- Add re-export: `pub use eigenvalue::{compute_point_features, PointFeatures};`

### `crates/pt-scan/Cargo.toml`
- No changes needed. `kiddo` and `nalgebra` already in deps.

## Files Created

### `crates/pt-scan/src/eigenvalue.rs` (NEW)

The sole new file. Contains all eigenvalue feature computation logic.

#### Public Types

```rust
/// Per-point geometric features derived from local covariance eigenvalues.
#[derive(Debug, Clone, Serialize)]
pub struct PointFeatures {
    pub planarity: f32,
    pub linearity: f32,
    pub sphericity: f32,
    pub omnivariance: f32,
    pub normal: [f32; 3],
    pub curvature: f32,
}
```

#### Public Functions

```rust
/// Compute eigenvalue-based geometric features for every point.
///
/// For each point, finds `k` nearest neighbors, builds the 3x3 covariance
/// matrix, and derives surface descriptors from the eigenvalues/eigenvectors.
///
/// # Panics
/// Panics if `k < 3` (need at least 3 neighbors for a valid covariance).
pub fn compute_point_features(points: &[Point], k: usize) -> Vec<PointFeatures>
```

#### Internal Functions

```rust
/// Build KD-tree and compute features for each point.
fn compute_features_sequential(positions: &[[f32; 3]], k: usize) -> Vec<PointFeatures>

/// Compute covariance matrix from K neighbor positions.
fn covariance_matrix(neighbors: &[[f32; 3]]) -> nalgebra::Matrix3<f32>

/// Extract sorted eigenvalues and eigenvectors from a 3x3 symmetric matrix.
/// Returns (eigenvalues_desc, eigenvectors) with λ1 ≥ λ2 ≥ λ3.
fn sorted_eigen(cov: &nalgebra::Matrix3<f32>) -> ([f32; 3], [nalgebra::Vector3<f32>; 3])

/// Compute PointFeatures from sorted eigenvalues and eigenvectors.
fn features_from_eigen(eigenvalues: [f32; 3], eigenvectors: [nalgebra::Vector3<f32>; 3]) -> PointFeatures
```

#### Test Module (`#[cfg(test)] mod tests`)

```rust
#[test] fn flat_grid_has_high_planarity()
#[test] fn line_has_high_linearity()
#[test] fn sphere_has_high_sphericity()
#[test] fn degenerate_coincident_points_no_panic()
#[test] fn normal_orientation_points_up()
#[test] fn performance_122k_under_2s()
```

## Module Boundaries

```
                        pt-scan
┌──────────────────────────────────────────────┐
│  parser → filter → ransac → [eigenvalue] → cluster → feature  │
│                                ▲ NEW                           │
│                                │                               │
│  Types: Point, PointFeatures   │                               │
│  Deps: kiddo (KNN), nalgebra (eigen)                          │
└──────────────────────────────────────────────┘
```

Eigenvalue features sit between RANSAC (ground separation) and clustering.
They consume `&[Point]` (obstacle points) and produce `Vec<PointFeatures>`.
Clustering (T-033-04) will consume both points and their features.

## Files NOT Changed

- `feature.rs` — per-cluster summaries stay as-is. Aggregate eigenvalue stats
  per cluster may be added in a future ticket.
- `cluster.rs` — DBSCAN unchanged. HDBSCAN (T-033-04) will consume features.
- `types.rs` — `Point` struct unchanged. `PointFeatures` lives in eigenvalue.rs.
- `Cargo.toml` — no new deps needed.

## Ordering

Single file creation — no ordering constraints. The module is self-contained
with no dependencies on other pt-scan modules beyond `types::Point`.
