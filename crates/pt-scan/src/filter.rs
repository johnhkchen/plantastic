//! Point cloud filtering: voxel downsampling and statistical outlier removal.

use std::collections::HashMap;
use std::num::NonZeroUsize;

use kiddo::ImmutableKdTree;

use crate::types::Point;

/// Reduce point cloud density by averaging points within each voxel cell.
///
/// Divides space into a uniform grid of `voxel_size`-meter cells. Points
/// within the same cell are averaged to produce a single output point.
/// Color values are averaged when present.
pub fn voxel_downsample(points: &[Point], voxel_size: f32) -> Vec<Point> {
    if points.is_empty() || voxel_size <= 0.0 {
        return Vec::new();
    }

    let inv_size = 1.0 / voxel_size;

    struct Accumulator {
        pos_sum: [f64; 3],
        color_sum: [u32; 3],
        color_count: u32,
        count: u32,
    }

    let mut cells: HashMap<(i32, i32, i32), Accumulator> = HashMap::new();

    for p in points {
        #[allow(clippy::cast_possible_truncation)]
        let key = (
            (p.position[0] * inv_size).floor() as i32,
            (p.position[1] * inv_size).floor() as i32,
            (p.position[2] * inv_size).floor() as i32,
        );

        let acc = cells.entry(key).or_insert(Accumulator {
            pos_sum: [0.0; 3],
            color_sum: [0; 3],
            color_count: 0,
            count: 0,
        });

        acc.pos_sum[0] += f64::from(p.position[0]);
        acc.pos_sum[1] += f64::from(p.position[1]);
        acc.pos_sum[2] += f64::from(p.position[2]);
        acc.count += 1;

        if let Some(c) = p.color {
            acc.color_sum[0] += u32::from(c[0]);
            acc.color_sum[1] += u32::from(c[1]);
            acc.color_sum[2] += u32::from(c[2]);
            acc.color_count += 1;
        }
    }

    cells
        .into_values()
        .map(|acc| {
            let n = f64::from(acc.count);
            #[allow(clippy::cast_possible_truncation)]
            let position = [
                (acc.pos_sum[0] / n) as f32,
                (acc.pos_sum[1] / n) as f32,
                (acc.pos_sum[2] / n) as f32,
            ];

            let color = if acc.color_count > 0 {
                #[allow(clippy::cast_possible_truncation)]
                let c = [
                    (acc.color_sum[0] / acc.color_count) as u8,
                    (acc.color_sum[1] / acc.color_count) as u8,
                    (acc.color_sum[2] / acc.color_count) as u8,
                ];
                Some(c)
            } else {
                None
            };

            Point { position, color }
        })
        .collect()
}

/// Remove statistical outliers from a point cloud.
///
/// For each point, computes the mean distance to its `k` nearest neighbors.
/// Points whose mean neighbor distance exceeds `global_mean + threshold * global_stddev`
/// are removed.
///
/// If there are fewer than `k + 1` points, returns the input unchanged (not
/// enough neighbors to compute meaningful statistics).
pub fn remove_outliers(points: &[Point], k: usize, threshold: f32) -> Vec<Point> {
    if points.len() <= k {
        return points.to_vec();
    }

    // Build KD-tree from positions
    let positions: Vec<[f32; 3]> = points.iter().map(|p| p.position).collect();
    let tree: ImmutableKdTree<f32, 3> = ImmutableKdTree::new_from_slice(&positions);

    // Compute mean neighbor distance for each point
    // Query k+1 because the point itself is included in results
    let query_k = k + 1;
    let mean_distances: Vec<f32> = positions
        .iter()
        .map(|pos| {
            let neighbors = tree.nearest_n::<kiddo::SquaredEuclidean>(
                pos,
                NonZeroUsize::new(query_k).expect("k+1 > 0"),
            );

            // Skip the first result (the point itself, distance 0) and compute
            // mean of remaining squared distances → take sqrt for actual distance
            let sum: f32 = neighbors.iter().skip(1).map(|n| n.distance.sqrt()).sum();

            #[allow(clippy::cast_precision_loss)]
            let mean = sum / k as f32;
            mean
        })
        .collect();

    // Compute global mean and stddev of mean distances
    #[allow(clippy::cast_precision_loss)]
    let n = mean_distances.len() as f64;
    let global_mean = mean_distances.iter().map(|&d| f64::from(d)).sum::<f64>() / n;
    let variance = mean_distances
        .iter()
        .map(|&d| {
            let diff = f64::from(d) - global_mean;
            diff * diff
        })
        .sum::<f64>()
        / n;
    let global_stddev = variance.sqrt();

    #[allow(clippy::cast_possible_truncation)]
    let cutoff = (global_mean + f64::from(threshold) * global_stddev) as f32;

    // Keep points within the threshold
    points
        .iter()
        .zip(mean_distances.iter())
        .filter(|(_, &dist)| dist <= cutoff)
        .map(|(point, _)| point.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    #[test]
    fn test_voxel_downsample_reduces_count() {
        timed(|| {
            // Create 1000 points uniformly distributed in a 1m cube.
            // With 0.5m voxels, we expect 2×2×2 = 8 cells.
            let mut points = Vec::new();
            for xi in 0..10 {
                for yi in 0..10 {
                    for zi in 0..10 {
                        points.push(Point {
                            position: [
                                xi as f32 * 0.1 + 0.05,
                                yi as f32 * 0.1 + 0.05,
                                zi as f32 * 0.1 + 0.05,
                            ],
                            color: Some([100, 100, 100]),
                        });
                    }
                }
            }

            assert_eq!(points.len(), 1000);
            let result = voxel_downsample(&points, 0.5);

            // 1m cube / 0.5m voxel = 2 divisions per axis = 8 cells
            assert_eq!(result.len(), 8);
        });
    }

    #[test]
    fn test_voxel_downsample_preserves_bounds() {
        timed(|| {
            let points = vec![
                Point {
                    position: [0.0, 0.0, 0.0],
                    color: None,
                },
                Point {
                    position: [10.0, 10.0, 10.0],
                    color: None,
                },
            ];

            // Large voxel size — each point in its own cell
            let result = voxel_downsample(&points, 100.0);
            // Both points land in the same cell (floor(0/100)=0, floor(10/100)=0)
            assert_eq!(result.len(), 1);

            // The averaged point should be at (5.0, 5.0, 5.0)
            let p = &result[0];
            assert!((p.position[0] - 5.0).abs() < 0.01);
            assert!((p.position[1] - 5.0).abs() < 0.01);
            assert!((p.position[2] - 5.0).abs() < 0.01);
        });
    }

    #[test]
    fn test_voxel_downsample_averages_color() {
        timed(|| {
            let points = vec![
                Point {
                    position: [0.1, 0.1, 0.1],
                    color: Some([0, 100, 200]),
                },
                Point {
                    position: [0.2, 0.2, 0.2],
                    color: Some([100, 200, 50]),
                },
            ];

            // Both in same 1m voxel
            let result = voxel_downsample(&points, 1.0);
            assert_eq!(result.len(), 1);

            // Color average: (0+100)/2=50, (100+200)/2=150, (200+50)/2=125
            let c = result[0].color.unwrap();
            assert_eq!(c, [50, 150, 125]);
        });
    }

    #[test]
    fn test_voxel_downsample_empty_input() {
        timed(|| {
            let result = voxel_downsample(&[], 0.5);
            assert!(result.is_empty());
        });
    }

    #[test]
    fn test_remove_outliers_filters_distant_points() {
        timed(|| {
            // 100 clustered points near origin
            let mut points: Vec<Point> = (0..100)
                .map(|i| {
                    let f = (i as f32) * 0.01;
                    Point {
                        position: [f, f, 0.0],
                        color: None,
                    }
                })
                .collect();

            // 5 outliers far away
            for i in 0..5 {
                points.push(Point {
                    position: [100.0 + i as f32, 100.0, 100.0],
                    color: None,
                });
            }

            assert_eq!(points.len(), 105);

            // k=10, threshold=2.0 should remove the 5 far outliers
            let filtered = remove_outliers(&points, 10, 2.0);

            // Outliers should be removed. We expect ~100 points remaining.
            assert!(
                filtered.len() >= 95 && filtered.len() <= 100,
                "expected ~100 points, got {}",
                filtered.len()
            );

            // No filtered point should be near (100, 100, 100)
            for p in &filtered {
                assert!(
                    p.position[0] < 50.0,
                    "outlier not removed: {:?}",
                    p.position
                );
            }
        });
    }

    #[test]
    fn test_remove_outliers_preserves_uniform_cloud() {
        timed(|| {
            // Uniform grid — all points are equidistant, none should be removed
            let mut points = Vec::new();
            for x in 0..5 {
                for y in 0..5 {
                    for z in 0..5 {
                        points.push(Point {
                            position: [x as f32, y as f32, z as f32],
                            color: None,
                        });
                    }
                }
            }

            let original_count = points.len(); // 125
            let filtered = remove_outliers(&points, 10, 2.0);

            // Uniform distribution: all mean distances are similar,
            // so threshold should keep most/all points.
            // Allow small margin — edge points may have slightly higher mean distance.
            assert!(
                filtered.len() >= original_count - 10,
                "too many points removed from uniform cloud: {} → {}",
                original_count,
                filtered.len()
            );
        });
    }

    #[test]
    fn test_remove_outliers_empty_input() {
        timed(|| {
            let result = remove_outliers(&[], 10, 2.0);
            assert!(result.is_empty());
        });
    }
}
