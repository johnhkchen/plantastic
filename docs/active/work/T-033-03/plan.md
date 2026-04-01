# T-033-03 Plan: Eigenvalue Features

## Step 1: Create `eigenvalue.rs` with `PointFeatures` struct and stub

Create the file with:
- Module doc comment
- Imports (kiddo, nalgebra, serde, Point)
- `PointFeatures` struct with Serialize derive
- `compute_point_features()` public function — stub returning empty vec

**Verify:** `cargo check -p pt-scan` compiles.

## Step 2: Register module in `lib.rs`

- Add `pub mod eigenvalue;`
- Add `pub use eigenvalue::{compute_point_features, PointFeatures};`

**Verify:** `cargo check -p pt-scan` compiles.

## Step 3: Implement core computation

Fill in the function body:
1. Extract positions from `&[Point]`
2. Build `ImmutableKdTree<f32, 3>`
3. For each point, query `nearest_n` for K neighbors
4. Compute centroid of neighbors
5. Build 3x3 covariance matrix (centered positions)
6. `SymmetricEigen::new()` decomposition
7. Sort eigenvalues descending, get corresponding eigenvectors
8. Compute ratios with λ1 ≈ 0 guard
9. Orient normal (positive Z)
10. Return `PointFeatures`

Internal helpers:
- `covariance_matrix(neighbors: &[[f32; 3]]) -> Matrix3<f32>`
- `sorted_eigen(cov: &Matrix3<f32>) -> ([f32; 3], [Vector3<f32>; 3])`
- `features_from_eigen(eigenvalues, eigenvectors) -> PointFeatures`

**Verify:** `cargo check -p pt-scan` compiles.

## Step 4: Add unit tests

### Test 4a: `flat_grid_has_high_planarity`
- 20x20 grid on z=0 plane, spacing 0.1m (400 points)
- K=20, check interior points have planarity > 0.9, sphericity < 0.1

### Test 4b: `line_has_high_linearity`
- 100 points along X axis, spacing 0.1m
- K=20, check interior points have linearity > 0.9

### Test 4c: `sphere_has_high_sphericity`
- 500 points uniformly on unit sphere (Fibonacci lattice)
- K=20, check interior points have sphericity > 0.3
  (sphericity won't be 1.0 on a surface — that requires volumetric scatter)

### Test 4d: `degenerate_coincident_points_no_panic`
- 50 copies of the same point
- K=20, should not panic, returns zero features

### Test 4e: `normal_orientation_points_up`
- Flat grid on z=5 plane
- K=20, check normal Z component > 0.9

### Test 4f: `performance_122k_under_2s`
- 122K random points in 10m cube
- K=20, assert elapsed < 2s
- Uses `std::time::Instant` directly (not timed wrapper since we need custom timeout)

**Verify:** `cargo test -p pt-scan -- eigenvalue` all pass.

## Step 5: Run quality gate

- `just fmt`
- `just lint`
- `just test`
- `just scenarios`

Fix any issues.

## Commit Strategy

Single atomic commit after all tests pass: "T-033-03: Add per-point eigenvalue
features (planarity, linearity, sphericity, omnivariance, normal, curvature)"
