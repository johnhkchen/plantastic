//! DBSCAN clustering for obstacle point clouds.
//!
//! Groups spatially proximate obstacle points into distinct clusters using
//! Density-Based Spatial Clustering of Applications with Noise (DBSCAN).
//! Each cluster becomes a candidate for downstream feature classification.

use kiddo::ImmutableKdTree;
use serde::{Deserialize, Serialize};

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
        let mut queue = neighbors;
        let mut qi = 0;

        while qi < queue.len() {
            let j = queue[qi];
            qi += 1;

            if !visited[j] {
                visited[j] = true;
                let j_neighbors = range_query(&tree, &positions[j], eps_sq);
                if j_neighbors.len() >= config.min_points {
                    // j is a core point — add its neighbors to the expansion queue
                    for &nb in &j_neighbors {
                        if labels[nb].is_none() && !queue.contains(&nb) {
                            queue.push(nb);
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
}
