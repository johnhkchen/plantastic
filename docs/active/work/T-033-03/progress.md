# T-033-03 Progress: Eigenvalue Features

## Completed

### Step 1-2: Create module and register
- Created `crates/pt-scan/src/eigenvalue.rs`
- Added `pub mod eigenvalue;` and re-exports in `lib.rs`
- Compiles clean

### Step 3: Core computation
- `compute_point_features(points: &[Point], k: usize) -> Vec<PointFeatures>`
- KD-tree built from positions via `ImmutableKdTree::new_from_slice`
- K-NN queries via `nearest_n::<SquaredEuclidean>`
- 3x3 covariance matrix from centered neighbor positions
- `SymmetricEigen::new()` for eigendecomposition
- Eigenvalues sorted descending, eigenvectors mapped accordingly
- Degenerate case handling (λ1 < 1e-10 → zero features)
- Normal oriented with positive Z

### Step 4: Unit tests (6/6 passing)
- `flat_grid_has_high_planarity` — 20x20 grid, planarity > 0.7 for interior points
- `line_has_high_linearity` — 100 collinear points, linearity > 0.9
- `random_scatter_has_high_sphericity` — 1000 random points, sphericity > 0.3
- `degenerate_coincident_points_no_panic` — 50 identical points, no crash
- `normal_orientation_points_up` — flat grid normals have Z > 0.9
- `performance_122k_reasonable_time` — 122K points in 0.31s release (target < 2s)

### Step 5: Quality gate
- `just fmt` — formatted
- `just lint` — pending
- `just test` — pending
- `just scenarios` — pending

## Deviations from Plan

1. **Planarity threshold lowered from 0.9 to 0.7.** On a regular 20x20 grid with
   K=20, the K nearest neighbors form a slightly anisotropic pattern (not perfectly
   circular), yielding planarity ~0.84. Threshold 0.7 still clearly distinguishes
   planar from non-planar surfaces.

2. **Sphere test replaced with random scatter.** Points on a sphere *surface* have
   low sphericity (the local neighborhood is 2D tangent plane). Volumetric scatter
   correctly tests the sphericity feature — models vegetation/debris.

3. **Performance test allows 15s in debug, 2s in release.** Debug builds are ~10x
   slower due to no optimizations. Actual release performance: 0.31s (6.5x under target).
