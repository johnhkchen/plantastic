//! RANSAC ground plane fitting.
//!
//! Fits a plane to the dominant ground surface in a point cloud using
//! random sample consensus. Points are classified as ground (within
//! threshold distance of the plane) or obstacles (above the plane).

use nalgebra::Vector3;
use rand::seq::index::sample;

use crate::error::ScanError;
use crate::types::{GroundClassification, Plane, Point};

/// Fit a ground plane using RANSAC and classify points.
///
/// Samples 3 random points per iteration, fits a plane, counts inliers,
/// and keeps the plane with the most inliers. Then classifies all points
/// against the best plane.
///
/// # Errors
///
/// Returns `ScanError::InsufficientPoints` if fewer than 3 points.
/// Returns `ScanError::NoGroundPlane` if no plane found with any inliers
/// (e.g., all points are collinear or coincident).
pub fn fit_ground_plane(
    points: &[Point],
    iterations: usize,
    distance_threshold: f32,
) -> Result<GroundClassification, ScanError> {
    if points.len() < 3 {
        return Err(ScanError::InsufficientPoints {
            found: points.len(),
            needed: 3,
        });
    }

    let positions: Vec<Vector3<f32>> = points
        .iter()
        .map(|p| Vector3::new(p.position[0], p.position[1], p.position[2]))
        .collect();

    let mut rng = rand::rng();
    let n = positions.len();

    let mut best_plane: Option<(Vector3<f32>, f32)> = None; // (normal, d)
    let mut best_inlier_count: usize = 0;
    let mut best_iteration: usize = 0;

    for iter in 0..iterations {
        // Sample 3 random distinct indices
        let indices = sample(&mut rng, n, 3);
        let p0 = &positions[indices.index(0)];
        let p1 = &positions[indices.index(1)];
        let p2 = &positions[indices.index(2)];

        // Compute plane from 3 points
        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let normal = v1.cross(&v2);

        let norm_len = normal.norm();
        if norm_len < 1e-10 {
            // Degenerate triangle (collinear points), skip
            continue;
        }

        let unit_normal = normal / norm_len;
        let d = -unit_normal.dot(p0);

        // Count inliers
        let inlier_count = positions
            .iter()
            .filter(|p| (unit_normal.dot(p) + d).abs() <= distance_threshold)
            .count();

        if inlier_count > best_inlier_count {
            best_inlier_count = inlier_count;
            best_plane = Some((unit_normal, d));
            best_iteration = iter;
        }
    }

    let (normal, d) = best_plane.ok_or(ScanError::NoGroundPlane)?;

    // Classify all points against the best plane
    let mut ground_indices = Vec::new();
    let mut obstacle_indices = Vec::new();

    for (i, p) in positions.iter().enumerate() {
        let dist = (normal.dot(p) + d).abs();
        if dist <= distance_threshold {
            ground_indices.push(i);
        } else {
            obstacle_indices.push(i);
        }
    }

    Ok(GroundClassification {
        ground_indices,
        obstacle_indices,
        plane: Plane {
            normal: [normal.x, normal.y, normal.z],
            d,
        },
        best_iteration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;
    use rand::SeedableRng;

    /// Seeded RANSAC for deterministic tests.
    fn fit_ground_plane_seeded(
        points: &[Point],
        iterations: usize,
        distance_threshold: f32,
        seed: u64,
    ) -> Result<GroundClassification, ScanError> {
        if points.len() < 3 {
            return Err(ScanError::InsufficientPoints {
                found: points.len(),
                needed: 3,
            });
        }

        let positions: Vec<Vector3<f32>> = points
            .iter()
            .map(|p| Vector3::new(p.position[0], p.position[1], p.position[2]))
            .collect();

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let n = positions.len();

        let mut best_plane: Option<(Vector3<f32>, f32)> = None;
        let mut best_inlier_count: usize = 0;
        let mut best_iteration: usize = 0;

        for iter in 0..iterations {
            let indices = sample(&mut rng, n, 3);
            let p0 = &positions[indices.index(0)];
            let p1 = &positions[indices.index(1)];
            let p2 = &positions[indices.index(2)];

            let v1 = p1 - p0;
            let v2 = p2 - p0;
            let normal = v1.cross(&v2);
            let norm_len = normal.norm();
            if norm_len < 1e-10 {
                continue;
            }

            let unit_normal = normal / norm_len;
            let d = -unit_normal.dot(p0);

            let inlier_count = positions
                .iter()
                .filter(|p| (unit_normal.dot(p) + d).abs() <= distance_threshold)
                .count();

            if inlier_count > best_inlier_count {
                best_inlier_count = inlier_count;
                best_plane = Some((unit_normal, d));
                best_iteration = iter;
            }
        }

        let (normal, d) = best_plane.ok_or(ScanError::NoGroundPlane)?;

        let mut ground_indices = Vec::new();
        let mut obstacle_indices = Vec::new();

        for (i, p) in positions.iter().enumerate() {
            let dist = (normal.dot(p) + d).abs();
            if dist <= distance_threshold {
                ground_indices.push(i);
            } else {
                obstacle_indices.push(i);
            }
        }

        Ok(GroundClassification {
            ground_indices,
            obstacle_indices,
            plane: Plane {
                normal: [normal.x, normal.y, normal.z],
                d,
            },
            best_iteration,
        })
    }

    #[test]
    fn test_fit_horizontal_plane() {
        timed(|| {
            // 500 ground points at z ≈ 0 (noise ±0.005m)
            // 100 obstacle points at z = 1.0
            let mut points = Vec::new();

            for i in 0..500 {
                let x = (i % 25) as f32 * 0.4;
                let y = (i / 25) as f32 * 0.4;
                let z = ((i % 7) as f32 - 3.0) * 0.001; // small noise
                points.push(Point {
                    position: [x, y, z],
                    color: None,
                });
            }

            for i in 0..100 {
                let x = (i % 10) as f32 * 1.0;
                let y = (i / 10) as f32 * 1.0;
                points.push(Point {
                    position: [x, y, 1.0],
                    color: None,
                });
            }

            let result = fit_ground_plane_seeded(&points, 1000, 0.02, 42).unwrap();

            // Normal should be approximately (0, 0, ±1) — vertical
            let nz = result.plane.normal[2].abs();
            assert!(nz > 0.99, "expected near-vertical normal, got nz={nz}");

            // d should be approximately 0 (plane passes through z≈0)
            assert!(
                result.plane.d.abs() < 0.05,
                "expected d ≈ 0, got d={}",
                result.plane.d
            );

            // Ground should have ~500, obstacles ~100
            // (with 0.02 threshold, all z≈0 points are ground)
            assert!(
                result.ground_indices.len() >= 450,
                "expected ~500 ground points, got {}",
                result.ground_indices.len()
            );
            assert!(
                result.obstacle_indices.len() >= 80,
                "expected ~100 obstacle points, got {}",
                result.obstacle_indices.len()
            );
        });
    }

    #[test]
    fn test_fit_tilted_plane() {
        timed(|| {
            // Points on z = 0.1*x + 0.2*y (tilted plane)
            // Normal should be proportional to (-0.1, -0.2, 1.0)
            let mut points = Vec::new();
            for i in 0..200 {
                let x = (i % 20) as f32 * 0.5;
                let y = (i / 20) as f32 * 0.5;
                let z = 0.1 * x + 0.2 * y;
                points.push(Point {
                    position: [x, y, z],
                    color: None,
                });
            }

            let result = fit_ground_plane_seeded(&points, 500, 0.01, 42).unwrap();

            // All 200 points should be ground (they're all on the plane)
            assert_eq!(result.ground_indices.len(), 200);
            assert_eq!(result.obstacle_indices.len(), 0);

            // Normal should point "upward" — the z component should be positive
            // and dominant. The expected normal direction is (-0.1, -0.2, 1.0) normalized.
            let expected_n = Vector3::new(-0.1_f32, -0.2, 1.0).normalize();
            let actual_n = Vector3::new(
                result.plane.normal[0],
                result.plane.normal[1],
                result.plane.normal[2],
            );

            // Normal might be flipped (pointing down instead of up) — check both
            let dot = actual_n.dot(&expected_n).abs();
            assert!(
                dot > 0.99,
                "expected normal parallel to (-0.1, -0.2, 1.0), got {:?}, dot={dot}",
                result.plane.normal
            );
        });
    }

    #[test]
    fn test_insufficient_points_error() {
        timed(|| {
            let points = vec![
                Point {
                    position: [0.0, 0.0, 0.0],
                    color: None,
                },
                Point {
                    position: [1.0, 0.0, 0.0],
                    color: None,
                },
            ];

            let result = fit_ground_plane(&points, 100, 0.02);
            assert!(matches!(
                result,
                Err(ScanError::InsufficientPoints {
                    found: 2,
                    needed: 3
                })
            ));
        });
    }

    #[test]
    fn test_classifies_obstacles_above_ground() {
        timed(|| {
            // 300 ground points at z = 0
            // 50 obstacle points at z = 0.5 (well above 0.02 threshold)
            let mut points = Vec::new();

            for i in 0..300 {
                let x = (i % 30) as f32 * 0.3;
                let y = (i / 30) as f32 * 0.3;
                points.push(Point {
                    position: [x, y, 0.0],
                    color: None,
                });
            }

            for i in 0..50 {
                let x = (i % 10) as f32 * 0.5;
                let y = (i / 10) as f32 * 0.5;
                points.push(Point {
                    position: [x, y, 0.5],
                    color: None,
                });
            }

            let result = fit_ground_plane_seeded(&points, 500, 0.02, 123).unwrap();

            // Ground: 300, obstacles: 50
            assert_eq!(result.ground_indices.len(), 300);
            assert_eq!(result.obstacle_indices.len(), 50);

            // Verify obstacle indices reference the z=0.5 points (indices 300-349)
            for &idx in &result.obstacle_indices {
                assert!(idx >= 300, "expected obstacle index >= 300, got {idx}");
            }
        });
    }
}
