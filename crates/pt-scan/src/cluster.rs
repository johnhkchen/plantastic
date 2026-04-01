//! Clustering for obstacle point clouds.
//!
//! Groups spatially proximate obstacle points into distinct clusters using
//! DBSCAN or HDBSCAN. Each cluster becomes a candidate for downstream
//! feature classification.
//!
//! DBSCAN uses a fixed epsilon radius. HDBSCAN evaluates cluster stability
//! across all epsilon values and supports augmented feature spaces (spatial +
//! eigenvalue features) for better separation of geometrically distinct objects.

use hdbscan::{Hdbscan, HdbscanHyperParams};
use kiddo::ImmutableKdTree;
use serde::{Deserialize, Serialize};

use crate::eigenvalue::PointFeatures;
use crate::types::{BoundingBox, Point};

/// Configuration for DBSCAN clustering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Neighborhood radius in meters. Points within this distance are neighbors.
    pub epsilon: f32,
    /// Minimum number of points required to form a dense cluster.
    pub min_points: usize,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            epsilon: 0.3,
            min_points: 50,
        }
    }
}

/// A single cluster of spatially proximate points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    /// Cluster identifier (0-indexed).
    pub id: u32,
    /// Indices into the input point slice.
    pub point_indices: Vec<usize>,
    /// Mean position of all member points.
    pub centroid: [f32; 3],
    /// Axis-aligned bounding box enclosing all member points.
    pub bbox: BoundingBox,
}

/// Result of DBSCAN clustering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResult {
    /// Identified clusters, sorted by id.
    pub clusters: Vec<Cluster>,
    /// Indices of noise points (not belonging to any cluster).
    pub noise_indices: Vec<usize>,
}

/// Cluster obstacle points using DBSCAN with a k-d tree spatial index.
///
/// Points within `config.epsilon` meters of each other are considered neighbors.
/// A core point has at least `config.min_points` neighbors (including itself).
/// Connected core points and their reachable neighbors form a cluster.
/// Points not reachable from any core point are classified as noise.
pub fn cluster_obstacles(points: &[Point], config: &ClusterConfig) -> ClusterResult {
    if points.is_empty() {
        return ClusterResult {
            clusters: Vec::new(),
            noise_indices: Vec::new(),
        };
    }

    let positions: Vec<[f32; 3]> = points.iter().map(|p| p.position).collect();
    let tree: ImmutableKdTree<f32, 3> = ImmutableKdTree::new_from_slice(&positions);
    let eps_sq = config.epsilon * config.epsilon;

    let n = points.len();
    // None = unassigned, Some(id) = cluster id
    let mut labels: Vec<Option<u32>> = vec![None; n];
    let mut visited = vec![false; n];
    let mut cluster_id: u32 = 0;

    for i in 0..n {
        if visited[i] {
            continue;
        }
        visited[i] = true;

        let neighbors = range_query(&tree, &positions[i], eps_sq);

        if neighbors.len() < config.min_points {
            // Will be classified as noise if never reached by a cluster expansion
            continue;
        }

        // Start a new cluster
        labels[i] = Some(cluster_id);

        // Use a VecDeque for the expansion queue; track membership via labels/visited
        let mut queue = std::collections::VecDeque::with_capacity(neighbors.len());
        for &nb in &neighbors {
            if nb != i {
                queue.push_back(nb);
            }
        }

        while let Some(j) = queue.pop_front() {
            if !visited[j] {
                visited[j] = true;
                let j_neighbors = range_query(&tree, &positions[j], eps_sq);
                if j_neighbors.len() >= config.min_points {
                    // j is a core point — enqueue unvisited neighbors
                    for &nb in &j_neighbors {
                        if !visited[nb] && labels[nb].is_none() {
                            queue.push_back(nb);
                        }
                    }
                }
            }

            if labels[j].is_none() {
                labels[j] = Some(cluster_id);
            }
        }

        cluster_id += 1;
    }

    // Build cluster structs
    let mut cluster_points: Vec<Vec<usize>> = vec![Vec::new(); cluster_id as usize];
    let mut noise_indices = Vec::new();

    for (i, label) in labels.iter().enumerate() {
        match label {
            Some(id) => cluster_points[*id as usize].push(i),
            None => noise_indices.push(i),
        }
    }

    let clusters = cluster_points
        .into_iter()
        .enumerate()
        .map(|(id, indices)| {
            let member_positions: Vec<[f32; 3]> = indices.iter().map(|&i| positions[i]).collect();
            let centroid = compute_centroid(&member_positions);
            let bbox = BoundingBox::from_positions(&member_positions)
                .expect("cluster has at least min_points members");

            #[allow(clippy::cast_possible_truncation)]
            Cluster {
                id: id as u32,
                point_indices: indices,
                centroid,
                bbox,
            }
        })
        .collect();

    ClusterResult {
        clusters,
        noise_indices,
    }
}

/// Configuration for HDBSCAN clustering in augmented feature space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdbscanConfig {
    /// Minimum number of points for a group to be considered a cluster.
    pub min_cluster_size: usize,
    /// Number of neighbors used to compute core distances.
    pub min_samples: usize,
    /// Weight applied to normalized spatial coordinates relative to eigenvalue
    /// features. 1.0 = equal weight, >1.0 = more spatial influence.
    pub spatial_weight: f64,
}

impl Default for HdbscanConfig {
    fn default() -> Self {
        Self {
            min_cluster_size: 200,
            min_samples: 10,
            spatial_weight: 1.0,
        }
    }
}

/// Cluster obstacle points using HDBSCAN in an augmented feature space.
///
/// Builds 6D feature vectors `[x, y, z, planarity, linearity, sphericity]`
/// where spatial coordinates are min-max normalized and weighted by
/// `config.spatial_weight`. HDBSCAN extracts the most stable clusters from
/// the density hierarchy, eliminating the need for a fixed epsilon.
///
/// # Panics
///
/// Panics if `points.len() != features.len()`.
pub fn hdbscan_cluster(
    points: &[Point],
    features: &[PointFeatures],
    config: &HdbscanConfig,
) -> ClusterResult {
    assert_eq!(
        points.len(),
        features.len(),
        "points and features must have the same length"
    );

    if points.is_empty() {
        return ClusterResult {
            clusters: Vec::new(),
            noise_indices: Vec::new(),
        };
    }

    // If fewer points than min_cluster_size, everything is noise
    if points.len() < config.min_cluster_size {
        return ClusterResult {
            clusters: Vec::new(),
            noise_indices: (0..points.len()).collect(),
        };
    }

    let data = build_feature_vectors(points, features, config.spatial_weight);

    // Clamp min_samples to data length to avoid out-of-bounds in the crate
    let min_samples = config
        .min_samples
        .min(points.len().saturating_sub(1))
        .max(1);

    let hyper_params = HdbscanHyperParams::builder()
        .min_cluster_size(config.min_cluster_size)
        .min_samples(min_samples)
        .build();

    let clusterer = Hdbscan::new(&data, hyper_params);
    match clusterer.cluster() {
        Ok(labels) => labels_to_cluster_result(&labels, points),
        Err(_) => {
            // Graceful fallback: treat all points as noise
            ClusterResult {
                clusters: Vec::new(),
                noise_indices: (0..points.len()).collect(),
            }
        }
    }
}

/// Build normalized 6D feature vectors for HDBSCAN.
///
/// Dimensions: [x_norm * w, y_norm * w, z_norm * w, planarity, linearity, sphericity]
/// where spatial coords are min-max normalized to [0, 1].
fn build_feature_vectors(
    points: &[Point],
    features: &[PointFeatures],
    spatial_weight: f64,
) -> Vec<Vec<f64>> {
    // Compute spatial min/max for normalization
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for p in points {
        for d in 0..3 {
            let v = f64::from(p.position[d]);
            if v < min[d] {
                min[d] = v;
            }
            if v > max[d] {
                max[d] = v;
            }
        }
    }

    let range: [f64; 3] = [
        (max[0] - min[0]).max(1e-10),
        (max[1] - min[1]).max(1e-10),
        (max[2] - min[2]).max(1e-10),
    ];

    points
        .iter()
        .zip(features.iter())
        .map(|(p, f)| {
            vec![
                (f64::from(p.position[0]) - min[0]) / range[0] * spatial_weight,
                (f64::from(p.position[1]) - min[1]) / range[1] * spatial_weight,
                (f64::from(p.position[2]) - min[2]) / range[2] * spatial_weight,
                f64::from(f.planarity),
                f64::from(f.linearity),
                f64::from(f.sphericity),
            ]
        })
        .collect()
}

/// Convert HDBSCAN labels (-1 = noise, 0+ = cluster id) to `ClusterResult`.
fn labels_to_cluster_result(labels: &[i32], points: &[Point]) -> ClusterResult {
    // Find max label to size the cluster_points vec
    let max_label = labels.iter().copied().max().unwrap_or(-1);

    if max_label < 0 {
        // All noise
        return ClusterResult {
            clusters: Vec::new(),
            noise_indices: (0..points.len()).collect(),
        };
    }

    #[allow(clippy::cast_sign_loss)]
    let num_clusters = (max_label + 1) as usize;
    let mut cluster_points: Vec<Vec<usize>> = vec![Vec::new(); num_clusters];
    let mut noise_indices = Vec::new();

    for (i, &label) in labels.iter().enumerate() {
        if label < 0 {
            noise_indices.push(i);
        } else {
            #[allow(clippy::cast_sign_loss)]
            cluster_points[label as usize].push(i);
        }
    }

    let clusters = cluster_points
        .into_iter()
        .enumerate()
        .filter(|(_, indices)| !indices.is_empty())
        .map(|(id, indices)| {
            let member_positions: Vec<[f32; 3]> =
                indices.iter().map(|&i| points[i].position).collect();
            let centroid = compute_centroid(&member_positions);
            let bbox = BoundingBox::from_positions(&member_positions)
                .expect("cluster has at least one member");

            #[allow(clippy::cast_possible_truncation)]
            Cluster {
                id: id as u32,
                point_indices: indices,
                centroid,
                bbox,
            }
        })
        .collect();

    ClusterResult {
        clusters,
        noise_indices,
    }
}

/// Range query: find all point indices within squared distance of target.
#[allow(clippy::cast_possible_truncation)]
fn range_query(tree: &ImmutableKdTree<f32, 3>, pos: &[f32; 3], eps_sq: f32) -> Vec<usize> {
    tree.within::<kiddo::SquaredEuclidean>(pos, eps_sq)
        .iter()
        .map(|n| n.item as usize)
        .collect()
}

/// Compute the centroid (mean position) of a set of positions.
fn compute_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    let mut sum = [0.0_f64; 3];
    for p in positions {
        sum[0] += f64::from(p[0]);
        sum[1] += f64::from(p[1]);
        sum[2] += f64::from(p[2]);
    }
    #[allow(clippy::cast_possible_truncation)]
    let n = positions.len() as f64;
    #[allow(clippy::cast_possible_truncation)]
    [
        (sum[0] / n) as f32,
        (sum[1] / n) as f32,
        (sum[2] / n) as f32,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    fn make_blob(center: [f32; 3], count: usize, spread: f32) -> Vec<Point> {
        (0..count)
            .map(|i| {
                // Deterministic spread: points arranged in a small grid around center
                let fi = i as f32;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let side = (count as f32).cbrt().ceil() as usize;
                let x = (i % side) as f32 / side as f32 * spread - spread / 2.0;
                let y = ((i / side) % side) as f32 / side as f32 * spread - spread / 2.0;
                let z = (i / (side * side)) as f32 / side as f32 * spread - spread / 2.0;
                let _ = fi; // suppress unused warning
                Point {
                    position: [center[0] + x, center[1] + y, center[2] + z],
                    color: None,
                }
            })
            .collect()
    }

    #[test]
    fn test_two_separated_clusters() {
        timed(|| {
            let mut points = make_blob([0.0, 0.0, 0.0], 100, 0.4);
            points.extend(make_blob([5.0, 5.0, 5.0], 100, 0.4));

            let config = ClusterConfig {
                epsilon: 0.5,
                min_points: 3,
            };
            let result = cluster_obstacles(&points, &config);

            assert_eq!(
                result.clusters.len(),
                2,
                "expected 2 clusters, got {}",
                result.clusters.len()
            );

            // Each cluster should have approximately 100 points
            for c in &result.clusters {
                assert!(
                    c.point_indices.len() >= 90,
                    "cluster {} has only {} points",
                    c.id,
                    c.point_indices.len()
                );
            }
        });
    }

    #[test]
    fn test_noise_not_merged() {
        timed(|| {
            let mut points = make_blob([0.0, 0.0, 0.0], 100, 0.4);
            points.extend(make_blob([5.0, 5.0, 5.0], 100, 0.4));

            // Add 5 isolated noise points between the blobs, each >epsilon apart
            for i in 0..5 {
                points.push(Point {
                    position: [1.0 + i as f32 * 0.8, 2.5, 2.5],
                    color: None,
                });
            }

            let config = ClusterConfig {
                epsilon: 0.5,
                min_points: 3,
            };
            let result = cluster_obstacles(&points, &config);

            // Still exactly 2 clusters — isolated noise points don't form clusters
            assert_eq!(
                result.clusters.len(),
                2,
                "noise merged clusters: got {} clusters",
                result.clusters.len()
            );

            // Noise points should be in noise_indices
            assert!(
                !result.noise_indices.is_empty(),
                "expected noise points but got none"
            );
        });
    }

    #[test]
    fn test_single_cluster() {
        timed(|| {
            let points = make_blob([0.0, 0.0, 0.0], 200, 0.4);

            let config = ClusterConfig {
                epsilon: 0.5,
                min_points: 3,
            };
            let result = cluster_obstacles(&points, &config);

            assert_eq!(
                result.clusters.len(),
                1,
                "expected 1 cluster, got {}",
                result.clusters.len()
            );
            assert_eq!(result.clusters[0].point_indices.len(), 200);
        });
    }

    #[test]
    fn test_empty_input() {
        timed(|| {
            let config = ClusterConfig::default();
            let result = cluster_obstacles(&[], &config);

            assert!(result.clusters.is_empty());
            assert!(result.noise_indices.is_empty());
        });
    }

    #[test]
    fn test_cluster_metadata() {
        timed(|| {
            // 8 points at corners of a 1m cube centered at (1, 2, 3)
            let points: Vec<Point> = [
                [0.5, 1.5, 2.5],
                [1.5, 1.5, 2.5],
                [0.5, 2.5, 2.5],
                [1.5, 2.5, 2.5],
                [0.5, 1.5, 3.5],
                [1.5, 1.5, 3.5],
                [0.5, 2.5, 3.5],
                [1.5, 2.5, 3.5],
            ]
            .into_iter()
            .map(|pos| Point {
                position: pos,
                color: None,
            })
            .collect();

            let config = ClusterConfig {
                epsilon: 2.0,
                min_points: 3,
            };
            let result = cluster_obstacles(&points, &config);

            assert_eq!(result.clusters.len(), 1);
            let c = &result.clusters[0];

            // Centroid should be (1.0, 2.0, 3.0) — the center of the cube
            assert!(
                (c.centroid[0] - 1.0).abs() < 0.01,
                "centroid x: {}",
                c.centroid[0]
            );
            assert!(
                (c.centroid[1] - 2.0).abs() < 0.01,
                "centroid y: {}",
                c.centroid[1]
            );
            assert!(
                (c.centroid[2] - 3.0).abs() < 0.01,
                "centroid z: {}",
                c.centroid[2]
            );

            // BBox should be [0.5, 1.5, 2.5] to [1.5, 2.5, 3.5]
            assert!((c.bbox.min[0] - 0.5).abs() < 0.01);
            assert!((c.bbox.max[0] - 1.5).abs() < 0.01);
            assert!((c.bbox.min[2] - 2.5).abs() < 0.01);
            assert!((c.bbox.max[2] - 3.5).abs() < 0.01);
        });
    }

    // --- HDBSCAN tests ---

    fn uniform_features() -> PointFeatures {
        PointFeatures {
            planarity: 0.33,
            linearity: 0.33,
            sphericity: 0.33,
            omnivariance: 0.1,
            normal: [0.0, 0.0, 1.0],
            curvature: 0.1,
        }
    }

    fn planar_features() -> PointFeatures {
        PointFeatures {
            planarity: 0.9,
            linearity: 0.05,
            sphericity: 0.05,
            omnivariance: 0.01,
            normal: [0.0, 0.0, 1.0],
            curvature: 0.01,
        }
    }

    fn spherical_features() -> PointFeatures {
        PointFeatures {
            planarity: 0.05,
            linearity: 0.05,
            sphericity: 0.9,
            omnivariance: 0.5,
            normal: [0.0, 0.0, 1.0],
            curvature: 0.3,
        }
    }

    #[test]
    fn test_hdbscan_two_separated_clusters() {
        timed(|| {
            let mut points = make_blob([0.0, 0.0, 0.0], 200, 0.5);
            points.extend(make_blob([10.0, 10.0, 10.0], 200, 0.5));

            let features: Vec<PointFeatures> = points.iter().map(|_| uniform_features()).collect();

            let config = HdbscanConfig {
                min_cluster_size: 20,
                min_samples: 5,
                spatial_weight: 1.0,
            };
            let result = hdbscan_cluster(&points, &features, &config);

            assert_eq!(
                result.clusters.len(),
                2,
                "expected 2 clusters, got {}",
                result.clusters.len()
            );

            for c in &result.clusters {
                assert!(
                    c.point_indices.len() >= 150,
                    "cluster {} has only {} points",
                    c.id,
                    c.point_indices.len()
                );
            }
        });
    }

    #[test]
    fn test_hdbscan_noise_not_merged() {
        timed(|| {
            let mut points = make_blob([0.0, 0.0, 0.0], 200, 0.5);
            points.extend(make_blob([10.0, 10.0, 10.0], 200, 0.5));

            // Add isolated noise points far from both clusters, each far apart
            for i in 0..5 {
                points.push(Point {
                    position: [50.0 + i as f32 * 20.0, 50.0, 50.0],
                    color: None,
                });
            }

            let features: Vec<PointFeatures> = points.iter().map(|_| uniform_features()).collect();

            let config = HdbscanConfig {
                min_cluster_size: 20,
                min_samples: 5,
                spatial_weight: 1.0,
            };
            let result = hdbscan_cluster(&points, &features, &config);

            // Should still have 2 main clusters
            assert!(
                result.clusters.len() >= 2,
                "expected >=2 clusters, got {}",
                result.clusters.len()
            );

            // Total assigned points should not include all 405 — some must be noise
            let assigned: usize = result.clusters.iter().map(|c| c.point_indices.len()).sum();
            assert!(
                assigned < points.len(),
                "expected some noise points, but all {} points were assigned",
                points.len()
            );
        });
    }

    #[test]
    fn test_hdbscan_uniform_blob_no_crash() {
        timed(|| {
            // A single uniform-density blob may produce 0 clusters (all noise) or
            // a few sub-clusters depending on the grid structure. HDBSCAN needs
            // density contrast to form stable clusters. This test verifies graceful
            // handling, not a specific cluster count.
            let points = make_blob([0.0, 0.0, 0.0], 400, 0.5);
            let features: Vec<PointFeatures> = points.iter().map(|_| uniform_features()).collect();

            let config = HdbscanConfig {
                min_cluster_size: 50,
                min_samples: 10,
                spatial_weight: 1.0,
            };
            let result = hdbscan_cluster(&points, &features, &config);

            // Total should equal input size (assigned + noise)
            let assigned: usize = result.clusters.iter().map(|c| c.point_indices.len()).sum();
            assert_eq!(
                assigned + result.noise_indices.len(),
                400,
                "all points must be accounted for"
            );
        });
    }

    #[test]
    fn test_hdbscan_empty_input() {
        timed(|| {
            let config = HdbscanConfig::default();
            let result = hdbscan_cluster(&[], &[], &config);

            assert!(result.clusters.is_empty());
            assert!(result.noise_indices.is_empty());
        });
    }

    #[test]
    fn test_hdbscan_feature_separation() {
        timed(|| {
            // Two groups at the SAME spatial position but with different features.
            // Group A: planar features, Group B: spherical features.
            // With spatial_weight=0 (features only), HDBSCAN should separate them.
            let blob = make_blob([0.0, 0.0, 0.0], 200, 0.5);
            let mut points = blob.clone();
            points.extend(blob);

            let mut features = Vec::with_capacity(400);
            for _ in 0..200 {
                features.push(planar_features());
            }
            for _ in 0..200 {
                features.push(spherical_features());
            }

            let config = HdbscanConfig {
                min_cluster_size: 20,
                min_samples: 5,
                spatial_weight: 0.0, // features only
            };
            let result = hdbscan_cluster(&points, &features, &config);

            assert_eq!(
                result.clusters.len(),
                2,
                "expected 2 clusters from feature separation, got {}",
                result.clusters.len()
            );
        });
    }

    #[test]
    fn test_hdbscan_all_noise_fallback() {
        timed(|| {
            // 3 scattered points, min_cluster_size=100 — nothing can form a cluster
            let points: Vec<Point> = (0..3)
                .map(|i| Point {
                    position: [i as f32 * 100.0, 0.0, 0.0],
                    color: None,
                })
                .collect();
            let features: Vec<PointFeatures> = points.iter().map(|_| uniform_features()).collect();

            let config = HdbscanConfig {
                min_cluster_size: 100,
                min_samples: 5,
                spatial_weight: 1.0,
            };
            let result = hdbscan_cluster(&points, &features, &config);

            assert!(
                result.clusters.is_empty(),
                "expected 0 clusters, got {}",
                result.clusters.len()
            );
            assert_eq!(result.noise_indices.len(), 3);
        });
    }
}
