//! Feature candidate extraction from DBSCAN clusters.
//!
//! Summarizes each cluster into a [`FeatureCandidate`] — the structured input
//! the BAML classifier sees. The LLM never touches raw point data; it reasons
//! about geometric summaries.

use serde::Serialize;

use crate::cluster::Cluster;
use crate::types::{BoundingBox, Plane, Point};

/// Meters-to-feet conversion factor.
const M_TO_FT: f64 = 3.28084;

/// Minimum bounding-box dimension (meters) to avoid division by zero in density.
const MIN_DIM: f64 = 0.01;

/// Geometric summary of a single cluster, suitable for LLM classification.
#[derive(Debug, Clone, Serialize)]
pub struct FeatureCandidate {
    pub cluster_id: usize,
    pub centroid: [f64; 3],
    pub bbox_min: [f64; 3],
    pub bbox_max: [f64; 3],
    pub height_ft: f64,
    pub spread_ft: f64,
    pub point_count: usize,
    pub dominant_color: String,
    pub vertical_profile: String,
    pub density: f64,
}

/// Extract [`FeatureCandidate`]s from DBSCAN clusters.
///
/// `points` is the obstacle point slice that `Cluster.point_indices` indexes into.
pub fn extract_candidates(
    clusters: &[Cluster],
    points: &[Point],
    ground_plane: &Plane,
) -> Vec<FeatureCandidate> {
    clusters
        .iter()
        .map(|c| {
            let height_ft = compute_height_ft(points, &c.point_indices, ground_plane);
            let spread_ft = compute_spread_ft(&c.bbox);
            let density = compute_density(c.point_indices.len(), &c.bbox);
            let dominant_color = classify_color(points, &c.point_indices);
            let vertical_profile =
                classify_vertical_profile(height_ft, spread_ft, points, &c.point_indices, &c.bbox);

            FeatureCandidate {
                cluster_id: c.id as usize,
                centroid: f32_to_f64_3(c.centroid),
                bbox_min: f32_to_f64_3(c.bbox.min),
                bbox_max: f32_to_f64_3(c.bbox.max),
                height_ft,
                spread_ft,
                point_count: c.point_indices.len(),
                dominant_color,
                vertical_profile,
                density,
            }
        })
        .collect()
}

fn f32_to_f64_3(v: [f32; 3]) -> [f64; 3] {
    [f64::from(v[0]), f64::from(v[1]), f64::from(v[2])]
}

/// Max distance above the ground plane among all cluster points, in feet.
fn compute_height_ft(points: &[Point], indices: &[usize], plane: &Plane) -> f64 {
    let max_dist = indices
        .iter()
        .map(|&i| {
            let p = &points[i].position;
            (f64::from(plane.normal[0]) * f64::from(p[0])
                + f64::from(plane.normal[1]) * f64::from(p[1])
                + f64::from(plane.normal[2]) * f64::from(p[2])
                + f64::from(plane.d))
            .abs()
        })
        .fold(0.0_f64, f64::max);
    max_dist * M_TO_FT
}

/// Max horizontal extent (XY) of the bounding box, in feet.
fn compute_spread_ft(bbox: &BoundingBox) -> f64 {
    let dx = f64::from(bbox.max[0] - bbox.min[0]);
    let dy = f64::from(bbox.max[1] - bbox.min[1]);
    dx.max(dy) * M_TO_FT
}

/// Points per cubic meter, with minimum dimension clamping.
fn compute_density(point_count: usize, bbox: &BoundingBox) -> f64 {
    let dx = f64::from(bbox.max[0] - bbox.min[0]).max(MIN_DIM);
    let dy = f64::from(bbox.max[1] - bbox.min[1]).max(MIN_DIM);
    let dz = f64::from(bbox.max[2] - bbox.min[2]).max(MIN_DIM);
    point_count as f64 / (dx * dy * dz)
}

/// Classify dominant color from mean RGB of cluster points.
///
/// Returns one of: "green", "brown", "gray", "white", "mixed", "unknown".
fn classify_color(points: &[Point], indices: &[usize]) -> String {
    let mut sum_r: u64 = 0;
    let mut sum_g: u64 = 0;
    let mut sum_b: u64 = 0;
    let mut count: u64 = 0;

    for &i in indices {
        if let Some(c) = points[i].color {
            sum_r += u64::from(c[0]);
            sum_g += u64::from(c[1]);
            sum_b += u64::from(c[2]);
            count += 1;
        }
    }

    if count == 0 {
        return "unknown".to_string();
    }

    let r = (sum_r / count) as f64;
    let g = (sum_g / count) as f64;
    let b = (sum_b / count) as f64;

    classify_rgb(r, g, b)
}

fn classify_rgb(r: f64, g: f64, b: f64) -> String {
    // White: all channels high
    if r > 200.0 && g > 200.0 && b > 200.0 {
        return "white".to_string();
    }

    // Gray: channels close together, moderate value
    let max_c = r.max(g).max(b);
    let min_c = r.min(g).min(b);
    if max_c - min_c < 30.0 && max_c > 50.0 {
        return "gray".to_string();
    }

    // Green: G dominant
    if g > 80.0 && g > r * 1.2 && g > b * 1.2 {
        return "green".to_string();
    }

    // Brown: warm earth tones
    if r > 80.0 && r > g && g > 40.0 && b < g {
        return "brown".to_string();
    }

    "mixed".to_string()
}

/// Classify vertical profile from aspect ratio and shape analysis.
///
/// Returns one of: "columnar", "flat", "conical", "spreading", "irregular".
fn classify_vertical_profile(
    height_ft: f64,
    spread_ft: f64,
    points: &[Point],
    indices: &[usize],
    bbox: &BoundingBox,
) -> String {
    // Degenerate: no meaningful spread
    if spread_ft < 0.1 || height_ft < 0.1 {
        return "irregular".to_string();
    }

    let ratio = height_ft / spread_ft;

    if ratio > 3.0 {
        return "columnar".to_string();
    }

    if ratio < 0.5 {
        return "flat".to_string();
    }

    // Mid-range: check for conical taper
    if is_conical(points, indices, bbox) {
        return "conical".to_string();
    }

    "spreading".to_string()
}

/// Check if a cluster tapers upward (conical shape).
///
/// Splits the cluster into upper and lower halves by Z, compares XY spread.
/// Conical if upper half spread < 70% of lower half spread.
fn is_conical(points: &[Point], indices: &[usize], bbox: &BoundingBox) -> bool {
    let mid_z = (bbox.min[2] + bbox.max[2]) / 2.0;

    let mut lower_min_x = f32::MAX;
    let mut lower_max_x = f32::MIN;
    let mut lower_min_y = f32::MAX;
    let mut lower_max_y = f32::MIN;
    let mut lower_count = 0_usize;

    let mut upper_min_x = f32::MAX;
    let mut upper_max_x = f32::MIN;
    let mut upper_min_y = f32::MAX;
    let mut upper_max_y = f32::MIN;
    let mut upper_count = 0_usize;

    for &i in indices {
        let p = &points[i].position;
        if p[2] <= mid_z {
            lower_min_x = lower_min_x.min(p[0]);
            lower_max_x = lower_max_x.max(p[0]);
            lower_min_y = lower_min_y.min(p[1]);
            lower_max_y = lower_max_y.max(p[1]);
            lower_count += 1;
        } else {
            upper_min_x = upper_min_x.min(p[0]);
            upper_max_x = upper_max_x.max(p[0]);
            upper_min_y = upper_min_y.min(p[1]);
            upper_max_y = upper_max_y.max(p[1]);
            upper_count += 1;
        }
    }

    // Need points in both halves
    if lower_count < 2 || upper_count < 2 {
        return false;
    }

    let lower_spread = (lower_max_x - lower_min_x).max(lower_max_y - lower_min_y);
    let upper_spread = (upper_max_x - upper_min_x).max(upper_max_y - upper_min_y);

    if lower_spread < 0.01 {
        return false;
    }

    (upper_spread / lower_spread) < 0.7
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::Cluster;
    use pt_test_utils::timed;

    /// Make a cluster from raw points with a given ID.
    fn make_cluster(id: u32, points: &[Point]) -> Cluster {
        let positions: Vec<[f32; 3]> = points.iter().map(|p| p.position).collect();
        let centroid = {
            let mut sum = [0.0_f64; 3];
            for p in &positions {
                sum[0] += f64::from(p[0]);
                sum[1] += f64::from(p[1]);
                sum[2] += f64::from(p[2]);
            }
            let n = positions.len() as f64;
            #[allow(clippy::cast_possible_truncation)]
            [
                (sum[0] / n) as f32,
                (sum[1] / n) as f32,
                (sum[2] / n) as f32,
            ]
        };
        let bbox = BoundingBox::from_positions(&positions).unwrap();
        Cluster {
            id,
            point_indices: (0..points.len()).collect(),
            centroid,
            bbox,
        }
    }

    fn point(x: f32, y: f32, z: f32) -> Point {
        Point {
            position: [x, y, z],
            color: None,
        }
    }

    fn colored_point(x: f32, y: f32, z: f32, r: u8, g: u8, b: u8) -> Point {
        Point {
            position: [x, y, z],
            color: Some([r, g, b]),
        }
    }

    #[test]
    fn test_known_cube_candidate() {
        timed(|| {
            // 8 points at corners of a 1m cube sitting 2m above ground plane z=0
            // bbox: [0,0,2] to [1,1,3]
            let points = vec![
                point(0.0, 0.0, 2.0),
                point(1.0, 0.0, 2.0),
                point(0.0, 1.0, 2.0),
                point(1.0, 1.0, 2.0),
                point(0.0, 0.0, 3.0),
                point(1.0, 0.0, 3.0),
                point(0.0, 1.0, 3.0),
                point(1.0, 1.0, 3.0),
            ];
            let cluster = make_cluster(0, &points);
            let plane = Plane {
                normal: [0.0, 0.0, 1.0],
                d: 0.0,
            };

            let candidates = extract_candidates(&[cluster], &points, &plane);
            assert_eq!(candidates.len(), 1);

            let c = &candidates[0];
            assert_eq!(c.cluster_id, 0);
            assert_eq!(c.point_count, 8);

            // Height: max distance from z=0 plane = 3.0m = 9.84252 ft
            let expected_height = 3.0 * M_TO_FT;
            assert!(
                (c.height_ft - expected_height).abs() < 0.01,
                "height_ft: {}, expected: {}",
                c.height_ft,
                expected_height
            );

            // Spread: max(1.0, 1.0) = 1.0m = 3.28084 ft
            let expected_spread = 1.0 * M_TO_FT;
            assert!(
                (c.spread_ft - expected_spread).abs() < 0.01,
                "spread_ft: {}, expected: {}",
                c.spread_ft,
                expected_spread
            );

            // Density: 8 points / (1.0 * 1.0 * 1.0) = 8.0 pts/m³
            assert!(
                (c.density - 8.0).abs() < 0.01,
                "density: {}, expected: 8.0",
                c.density
            );

            // No color data → "unknown"
            assert_eq!(c.dominant_color, "unknown");

            // height/spread ratio = 3.0/1.0 * (same conversion) = 3.0 → columnar (>3? no, =3)
            // ratio = 9.84/3.28 = 3.0, which is NOT > 3, so it's mid-range → check conical
            // Upper half (z>2.5) and lower half same spread → spreading
            assert_eq!(c.vertical_profile, "spreading");
        });
    }

    #[test]
    fn test_color_classification_green() {
        timed(|| {
            let points = vec![
                colored_point(0.0, 0.0, 1.0, 30, 160, 40),
                colored_point(1.0, 0.0, 1.0, 20, 180, 30),
                colored_point(0.0, 1.0, 1.0, 40, 150, 50),
            ];
            let result = classify_color(&points, &[0, 1, 2]);
            assert_eq!(result, "green");
        });
    }

    #[test]
    fn test_color_classification_brown() {
        timed(|| {
            let points = vec![
                colored_point(0.0, 0.0, 1.0, 140, 90, 50),
                colored_point(1.0, 0.0, 1.0, 150, 100, 60),
                colored_point(0.0, 1.0, 1.0, 130, 80, 40),
            ];
            let result = classify_color(&points, &[0, 1, 2]);
            assert_eq!(result, "brown");
        });
    }

    #[test]
    fn test_color_classification_gray() {
        timed(|| {
            let points = vec![
                colored_point(0.0, 0.0, 1.0, 120, 120, 120),
                colored_point(1.0, 0.0, 1.0, 130, 125, 128),
            ];
            let result = classify_color(&points, &[0, 1]);
            assert_eq!(result, "gray");
        });
    }

    #[test]
    fn test_color_classification_white() {
        timed(|| {
            let points = vec![
                colored_point(0.0, 0.0, 1.0, 240, 235, 230),
                colored_point(1.0, 0.0, 1.0, 220, 210, 215),
            ];
            let result = classify_color(&points, &[0, 1]);
            assert_eq!(result, "white");
        });
    }

    #[test]
    fn test_color_classification_no_color() {
        timed(|| {
            let points = vec![point(0.0, 0.0, 1.0), point(1.0, 0.0, 1.0)];
            let result = classify_color(&points, &[0, 1]);
            assert_eq!(result, "unknown");
        });
    }

    #[test]
    fn test_vertical_profile_columnar() {
        timed(|| {
            // Tall narrow cluster: height >> spread
            // 0.1m wide, 2m tall → ratio = (2*3.28)/(0.1*3.28) = 20 > 3
            let points = vec![
                point(0.0, 0.0, 1.0),
                point(0.1, 0.0, 1.0),
                point(0.0, 0.0, 3.0),
                point(0.1, 0.0, 3.0),
            ];
            let cluster = make_cluster(0, &points);
            let plane = Plane {
                normal: [0.0, 0.0, 1.0],
                d: 0.0,
            };
            let candidates = extract_candidates(&[cluster], &points, &plane);
            assert_eq!(candidates[0].vertical_profile, "columnar");
        });
    }

    #[test]
    fn test_vertical_profile_flat() {
        timed(|| {
            // Wide low cluster: spread >> height
            // 4m wide, 0.3m tall → ratio = (0.3*3.28)/(4*3.28) = 0.075 < 0.5
            let points = vec![
                point(0.0, 0.0, 0.3),
                point(4.0, 0.0, 0.3),
                point(0.0, 4.0, 0.3),
                point(4.0, 4.0, 0.6),
            ];
            let cluster = make_cluster(0, &points);
            let plane = Plane {
                normal: [0.0, 0.0, 1.0],
                d: 0.0,
            };
            let candidates = extract_candidates(&[cluster], &points, &plane);
            assert_eq!(candidates[0].vertical_profile, "flat");
        });
    }

    #[test]
    fn test_vertical_profile_spreading() {
        timed(|| {
            // Moderate ratio: 2m wide, 2m tall → ratio = 1.0
            // Both halves same spread → not conical → spreading
            let points = vec![
                point(0.0, 0.0, 1.0),
                point(2.0, 0.0, 1.0),
                point(0.0, 2.0, 1.0),
                point(2.0, 2.0, 1.0),
                point(0.0, 0.0, 3.0),
                point(2.0, 0.0, 3.0),
                point(0.0, 2.0, 3.0),
                point(2.0, 2.0, 3.0),
            ];
            let cluster = make_cluster(0, &points);
            let plane = Plane {
                normal: [0.0, 0.0, 1.0],
                d: 0.0,
            };
            let candidates = extract_candidates(&[cluster], &points, &plane);
            assert_eq!(candidates[0].vertical_profile, "spreading");
        });
    }

    #[test]
    fn test_vertical_profile_conical() {
        timed(|| {
            // Tapers upward: wide base, narrow top
            // Lower half spread: 3m, upper half spread: 0.5m (ratio 0.17 < 0.7)
            let mut points = Vec::new();
            // Lower half: z=1..2, spread 3m
            for x in 0..10 {
                for y in 0..10 {
                    points.push(point(x as f32 * 0.3, y as f32 * 0.3, 1.5));
                }
            }
            // Upper half: z=2..3, spread 0.5m
            for x in 0..5 {
                for y in 0..5 {
                    points.push(point(1.0 + x as f32 * 0.1, 1.0 + y as f32 * 0.1, 2.5));
                }
            }
            let cluster = make_cluster(0, &points);
            let plane = Plane {
                normal: [0.0, 0.0, 1.0],
                d: 0.0,
            };
            let candidates = extract_candidates(&[cluster], &points, &plane);
            assert_eq!(candidates[0].vertical_profile, "conical");
        });
    }

    #[test]
    fn test_empty_clusters() {
        timed(|| {
            let plane = Plane {
                normal: [0.0, 0.0, 1.0],
                d: 0.0,
            };
            let candidates = extract_candidates(&[], &[], &plane);
            assert!(candidates.is_empty());
        });
    }
}
