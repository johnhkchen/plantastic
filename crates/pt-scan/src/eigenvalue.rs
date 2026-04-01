//! Per-point eigenvalue features from local covariance matrices.
//!
//! For each point, computes K nearest neighbors, builds a 3x3 covariance matrix
//! from the local neighborhood, and derives geometric descriptors (planarity,
//! linearity, sphericity, omnivariance, surface normal, curvature) from the
//! eigenvalues and eigenvectors.
//!
//! Reference: Weinmann et al. (2015) — eigenvalue features for point cloud
//! classification.

use std::num::NonZeroUsize;

use kiddo::ImmutableKdTree;
use nalgebra::{Matrix3, SymmetricEigen, Vector3};
use serde::Serialize;

use crate::types::Point;

/// Minimum eigenvalue threshold to avoid division by zero.
const EIGEN_EPS: f32 = 1e-10;

/// Per-point geometric features derived from local covariance eigenvalues.
#[derive(Debug, Clone, Serialize)]
pub struct PointFeatures {
    /// (λ2 − λ3) / λ1 — high for flat surfaces (patios, paths, walls).
    pub planarity: f32,
    /// (λ1 − λ2) / λ1 — high for edges, poles, fences, wires.
    pub linearity: f32,
    /// λ3 / λ1 — high for vegetation, scattered debris.
    pub sphericity: f32,
    /// (λ1 × λ2 × λ3)^(1/3) — higher for vegetation than hardscape.
    pub omnivariance: f32,
    /// Surface normal (smallest eigenvector), oriented with positive Z.
    pub normal: [f32; 3],
    /// λ3 / (λ1 + λ2 + λ3) — surface variation / change of curvature.
    pub curvature: f32,
}

impl PointFeatures {
    /// Degenerate features for coincident or insufficient neighbors.
    fn degenerate() -> Self {
        Self {
            planarity: 0.0,
            linearity: 0.0,
            sphericity: 0.0,
            omnivariance: 0.0,
            normal: [0.0, 0.0, 1.0],
            curvature: 0.0,
        }
    }
}

/// Compute eigenvalue-based geometric features for every point.
///
/// For each point, finds `k` nearest neighbors (including itself), builds
/// the 3x3 covariance matrix, and derives surface descriptors from the
/// eigenvalues and eigenvectors.
///
/// # Panics
///
/// Panics if `k < 3` (need at least 3 neighbors for a valid covariance).
pub fn compute_point_features(points: &[Point], k: usize) -> Vec<PointFeatures> {
    assert!(k >= 3, "k must be >= 3 for valid covariance, got {k}");

    if points.is_empty() {
        return Vec::new();
    }

    let positions: Vec<[f32; 3]> = points.iter().map(|p| p.position).collect();
    let tree: ImmutableKdTree<f32, 3> = ImmutableKdTree::new_from_slice(&positions);

    let k_nz = NonZeroUsize::new(k).expect("k >= 3");

    positions
        .iter()
        .map(|pos| {
            let neighbors = tree.nearest_n::<kiddo::SquaredEuclidean>(pos, k_nz);

            // Collect neighbor positions
            #[allow(clippy::cast_possible_truncation)]
            let neighbor_positions: Vec<[f32; 3]> = neighbors
                .iter()
                .map(|n| positions[n.item as usize])
                .collect();

            if neighbor_positions.len() < 3 {
                return PointFeatures::degenerate();
            }

            let cov = covariance_matrix(&neighbor_positions);
            let (eigenvalues, eigenvectors) = sorted_eigen(&cov);
            features_from_eigen(eigenvalues, eigenvectors)
        })
        .collect()
}

/// Compute 3x3 covariance matrix from neighbor positions.
fn covariance_matrix(neighbors: &[[f32; 3]]) -> Matrix3<f32> {
    let n = neighbors.len() as f32;

    // Compute centroid
    let mut cx = 0.0_f32;
    let mut cy = 0.0_f32;
    let mut cz = 0.0_f32;
    for p in neighbors {
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n;
    cy /= n;
    cz /= n;

    // Build covariance matrix (symmetric, so we compute 6 unique elements)
    let mut cov = Matrix3::zeros();
    for p in neighbors {
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        let dz = p[2] - cz;
        cov[(0, 0)] += dx * dx;
        cov[(0, 1)] += dx * dy;
        cov[(0, 2)] += dx * dz;
        cov[(1, 1)] += dy * dy;
        cov[(1, 2)] += dy * dz;
        cov[(2, 2)] += dz * dz;
    }
    // Fill symmetric lower triangle
    cov[(1, 0)] = cov[(0, 1)];
    cov[(2, 0)] = cov[(0, 2)];
    cov[(2, 1)] = cov[(1, 2)];

    // Normalize by N (not N-1; this is a descriptive covariance of the neighborhood)
    cov /= n;
    cov
}

/// Eigendecomposition of a 3x3 symmetric matrix, sorted descending.
///
/// Returns `(eigenvalues, eigenvectors)` with λ1 ≥ λ2 ≥ λ3.
fn sorted_eigen(cov: &Matrix3<f32>) -> ([f32; 3], [Vector3<f32>; 3]) {
    let eigen = SymmetricEigen::new(*cov);

    // Pair eigenvalues with their eigenvector column indices and sort descending
    let mut indexed: [(f32, usize); 3] = [
        (eigen.eigenvalues[0], 0),
        (eigen.eigenvalues[1], 1),
        (eigen.eigenvalues[2], 2),
    ];
    indexed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let eigenvalues = [indexed[0].0, indexed[1].0, indexed[2].0];
    let eigenvectors = [
        eigen.eigenvectors.column(indexed[0].1).into(),
        eigen.eigenvectors.column(indexed[1].1).into(),
        eigen.eigenvectors.column(indexed[2].1).into(),
    ];

    (eigenvalues, eigenvectors)
}

/// Derive `PointFeatures` from sorted eigenvalues and eigenvectors.
fn features_from_eigen(eigenvalues: [f32; 3], eigenvectors: [Vector3<f32>; 3]) -> PointFeatures {
    let [l1, l2, l3] = eigenvalues;

    // Degenerate case: all eigenvalues near zero (coincident points)
    if l1 < EIGEN_EPS {
        return PointFeatures::degenerate();
    }

    let planarity = (l2 - l3) / l1;
    let linearity = (l1 - l2) / l1;
    let sphericity = l3 / l1;

    // Omnivariance: cube root of product of eigenvalues
    // Clamp negative eigenvalues to 0 (numerical noise)
    let l1_c = l1.max(0.0);
    let l2_c = l2.max(0.0);
    let l3_c = l3.max(0.0);
    let omnivariance = (l1_c * l2_c * l3_c).cbrt();

    // Surface normal: eigenvector of smallest eigenvalue (index 2)
    let mut normal = eigenvectors[2];

    // Orient normal to have positive Z component (pointing "up")
    if normal[2] < 0.0 {
        normal = -normal;
    }

    let eigen_sum = l1 + l2 + l3;
    let curvature = if eigen_sum > EIGEN_EPS {
        l3 / eigen_sum
    } else {
        0.0
    };

    PointFeatures {
        planarity,
        linearity,
        sphericity,
        omnivariance,
        normal: [normal[0], normal[1], normal[2]],
        curvature,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    fn point(x: f32, y: f32, z: f32) -> Point {
        Point {
            position: [x, y, z],
            color: None,
        }
    }

    /// Generate a flat grid on the z=`height` plane.
    fn flat_grid(side: usize, spacing: f32, height: f32) -> Vec<Point> {
        let mut points = Vec::with_capacity(side * side);
        for i in 0..side {
            for j in 0..side {
                points.push(point(i as f32 * spacing, j as f32 * spacing, height));
            }
        }
        points
    }

    /// Generate points along the X axis.
    fn line_points(count: usize, spacing: f32) -> Vec<Point> {
        (0..count)
            .map(|i| point(i as f32 * spacing, 0.0, 0.0))
            .collect()
    }

    /// Generate points on a unit sphere using Fibonacci lattice.
    #[allow(dead_code)] // Test utility for future spherical feature tests
    fn fibonacci_sphere(n: usize) -> Vec<Point> {
        let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;
        (0..n)
            .map(|i| {
                let theta = 2.0 * std::f32::consts::PI * i as f32 / golden_ratio;
                let phi = (1.0 - 2.0 * (i as f32 + 0.5) / n as f32).acos();
                point(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos())
            })
            .collect()
    }

    #[test]
    fn flat_grid_has_high_planarity() {
        timed(|| {
            let points = flat_grid(20, 0.1, 0.0);
            let features = compute_point_features(&points, 20);

            assert_eq!(features.len(), 400);

            // Check interior points (away from edges where K-NN is unbalanced)
            // Interior: rows 5..15, cols 5..15 → indices i*20+j
            // On a uniform 20x20 grid with K=20, the K nearest form a slightly
            // non-circular pattern, so planarity is ~0.84 not 1.0. Threshold 0.7
            // clearly distinguishes planar from non-planar.
            let mut planar_count = 0;
            let mut total_checked = 0;
            for i in 5..15 {
                for j in 5..15 {
                    let idx = i * 20 + j;
                    let f = &features[idx];
                    total_checked += 1;
                    if f.planarity > 0.7 && f.sphericity < 0.1 {
                        planar_count += 1;
                    }
                }
            }
            // At least 90% of interior points should be clearly planar
            assert!(
                planar_count as f64 / total_checked as f64 > 0.9,
                "only {planar_count}/{total_checked} interior points are planar"
            );
        });
    }

    #[test]
    fn line_has_high_linearity() {
        timed(|| {
            let points = line_points(100, 0.1);
            let features = compute_point_features(&points, 20);

            assert_eq!(features.len(), 100);

            // Check interior points (indices 20..80)
            let mut linear_count = 0;
            let total_checked = 60;
            for feat in features.iter().take(80).skip(20) {
                if feat.linearity > 0.9 {
                    linear_count += 1;
                }
            }
            assert!(
                linear_count as f64 / total_checked as f64 > 0.9,
                "only {linear_count}/{total_checked} interior points are linear"
            );
        });
    }

    #[test]
    fn random_scatter_has_high_sphericity() {
        timed(|| {
            // Volumetric scatter in a cube — all three eigenvalues similar,
            // giving high sphericity. This models vegetation/debris points.
            let mut points = Vec::with_capacity(1000);
            let mut seed: u64 = 12345;
            for _ in 0..1000 {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let x = ((seed >> 32) as f32) / (u32::MAX as f32);
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let y = ((seed >> 32) as f32) / (u32::MAX as f32);
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let z = ((seed >> 32) as f32) / (u32::MAX as f32);
                points.push(point(x, y, z));
            }

            let features = compute_point_features(&points, 20);
            assert_eq!(features.len(), 1000);

            // Interior points should have roughly equal eigenvalues → high sphericity
            // Skip boundary points (near cube faces) — check middle region
            let interior: Vec<_> = features
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    let p = &points[*i].position;
                    p[0] > 0.2 && p[0] < 0.8 && p[1] > 0.2 && p[1] < 0.8 && p[2] > 0.2 && p[2] < 0.8
                })
                .collect();

            let high_sph = interior.iter().filter(|(_, f)| f.sphericity > 0.3).count();
            assert!(
                high_sph as f64 / interior.len() as f64 > 0.7,
                "only {high_sph}/{} interior points have sphericity > 0.3",
                interior.len()
            );
        });
    }

    #[test]
    fn degenerate_coincident_points_no_panic() {
        timed(|| {
            // 50 copies of the same point
            let points: Vec<Point> = (0..50).map(|_| point(1.0, 2.0, 3.0)).collect();
            let features = compute_point_features(&points, 20);

            assert_eq!(features.len(), 50);
            // All features should be degenerate (zero ratios)
            for f in &features {
                assert!(
                    f.planarity.abs() < 0.01,
                    "expected ~0 planarity for coincident points, got {}",
                    f.planarity
                );
            }
        });
    }

    #[test]
    fn normal_orientation_points_up() {
        timed(|| {
            let points = flat_grid(20, 0.1, 5.0);
            let features = compute_point_features(&points, 20);

            // Interior points should have normal ≈ [0, 0, 1]
            for i in 5..15 {
                for j in 5..15 {
                    let idx = i * 20 + j;
                    let n = &features[idx].normal;
                    assert!(
                        n[2] > 0.9,
                        "normal at ({i},{j}) has Z={}, expected > 0.9",
                        n[2]
                    );
                }
            }
        });
    }

    #[test]
    fn performance_122k_reasonable_time() {
        // 122K points, K=20.
        // Target: < 2s in release on M-series Mac. Debug builds are ~5-10x slower,
        // so we allow 15s in debug to avoid false failures.
        let mut points = Vec::with_capacity(122_000);
        let mut seed: u64 = 42;
        for _ in 0..122_000 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let x = ((seed >> 32) as f32) / (u32::MAX as f32) * 10.0;
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let y = ((seed >> 32) as f32) / (u32::MAX as f32) * 10.0;
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let z = ((seed >> 32) as f32) / (u32::MAX as f32) * 10.0;
            points.push(point(x, y, z));
        }

        let start = std::time::Instant::now();
        let features = compute_point_features(&points, 20);
        let elapsed = start.elapsed();

        assert_eq!(features.len(), 122_000);

        let limit = if cfg!(debug_assertions) { 15.0 } else { 2.0 };
        assert!(
            elapsed.as_secs_f64() < limit,
            "took {:.2}s, expected < {limit}s",
            elapsed.as_secs_f64()
        );
    }
}
