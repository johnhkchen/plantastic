//! Gap measurement between feature candidates.
//!
//! Computes pairwise distances and clear widths between [`FeatureCandidate`]s.
//! The gap between two features (e.g. tree trunks) defines the plantable zone.

use serde::Serialize;

use crate::feature::FeatureCandidate;
use crate::types::Plane;

/// Meters-to-feet conversion factor.
const M_TO_FT: f64 = 3.28084;

/// Measured gap between two feature candidates.
#[derive(Debug, Clone, Serialize)]
pub struct Gap {
    /// Cluster ID of the first feature.
    pub feature_a_id: usize,
    /// Cluster ID of the second feature.
    pub feature_b_id: usize,
    /// Center-to-center distance in the XY plane (feet).
    pub centroid_distance_ft: f64,
    /// Plantable width: centroid distance minus feature radii (feet).
    pub clear_width_ft: f64,
    /// Perpendicular extent of the gap (feet).
    pub clear_length_ft: f64,
    /// Rectangular approximation of plantable area (sq ft).
    pub area_sqft: f64,
    /// Ground plane elevation at the gap midpoint (feet).
    pub ground_elevation_ft: f64,
    /// XY midpoint of the gap in meters.
    pub midpoint: [f64; 2],
}

/// Configuration for gap measurement.
#[derive(Debug, Clone)]
pub struct GapConfig {
    /// Maximum centroid distance (feet) to consider a pair as a gap.
    pub max_distance_ft: f64,
}

impl Default for GapConfig {
    fn default() -> Self {
        Self {
            max_distance_ft: 30.0,
        }
    }
}

/// Measure gaps between feature candidates.
///
/// Computes pairwise gaps for all candidate pairs within `config.max_distance_ft`.
/// Pairs with overlapping envelopes (clear_width ≤ 0) are excluded.
/// Results are sorted by centroid distance ascending.
pub fn measure_gaps(
    candidates: &[FeatureCandidate],
    ground_plane: &Plane,
    config: &GapConfig,
) -> Vec<Gap> {
    let mut gaps = Vec::new();

    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let a = &candidates[i];
            let b = &candidates[j];

            let dist_ft = centroid_distance_2d_ft(a, b);
            if dist_ft > config.max_distance_ft {
                continue;
            }

            let clear_width = dist_ft - (a.spread_ft / 2.0 + b.spread_ft / 2.0);
            if clear_width <= 0.0 {
                continue;
            }

            let clear_length = a.spread_ft.min(b.spread_ft);
            let area = clear_width * clear_length;

            let mx = (a.centroid[0] + b.centroid[0]) / 2.0;
            let my = (a.centroid[1] + b.centroid[1]) / 2.0;
            let elevation = ground_elevation_ft(mx, my, ground_plane);

            gaps.push(Gap {
                feature_a_id: a.cluster_id,
                feature_b_id: b.cluster_id,
                centroid_distance_ft: dist_ft,
                clear_width_ft: clear_width,
                clear_length_ft: clear_length,
                area_sqft: area,
                ground_elevation_ft: elevation,
                midpoint: [mx, my],
            });
        }
    }

    gaps.sort_by(|a, b| {
        a.centroid_distance_ft
            .partial_cmp(&b.centroid_distance_ft)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    gaps
}

/// 2D Euclidean distance between centroids in the XY plane, in feet.
fn centroid_distance_2d_ft(a: &FeatureCandidate, b: &FeatureCandidate) -> f64 {
    let dx = a.centroid[0] - b.centroid[0];
    let dy = a.centroid[1] - b.centroid[1];
    (dx * dx + dy * dy).sqrt() * M_TO_FT
}

/// Ground plane elevation at an XY point, in feet.
///
/// For plane equation n·p + d = 0 with n = [a, b, c]:
///   z = -(a*x + b*y + d) / c
fn ground_elevation_ft(x: f64, y: f64, plane: &Plane) -> f64 {
    let c = f64::from(plane.normal[2]);
    if c.abs() < 1e-6 {
        return 0.0;
    }
    let a = f64::from(plane.normal[0]);
    let b = f64::from(plane.normal[1]);
    let d = f64::from(plane.d);
    let z = -(a * x + b * y + d) / c;
    z * M_TO_FT
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    /// Build a minimal FeatureCandidate for gap testing.
    fn candidate(id: usize, cx: f64, cy: f64, spread_ft: f64) -> FeatureCandidate {
        FeatureCandidate {
            cluster_id: id,
            centroid: [cx, cy, 0.0],
            bbox_min: [cx - 0.5, cy - 0.5, 0.0],
            bbox_max: [cx + 0.5, cy + 0.5, 1.0],
            height_ft: 10.0,
            spread_ft,
            point_count: 100,
            dominant_color: "brown".to_string(),
            vertical_profile: "columnar".to_string(),
            density: 500.0,
        }
    }

    fn flat_ground() -> Plane {
        Plane {
            normal: [0.0, 0.0, 1.0],
            d: 0.0,
        }
    }

    #[test]
    fn test_two_features_known_gap() {
        timed(|| {
            // Two features 3.048m apart in X (≈10ft), each with spread_ft=2.0
            // centroid_distance = 3.048 * 3.28084 ≈ 10.0 ft
            // clear_width = 10.0 - (1.0 + 1.0) = 8.0 ft
            // clear_length = min(2.0, 2.0) = 2.0 ft
            // area = 8.0 * 2.0 = 16.0 sqft
            let a = candidate(0, 0.0, 0.0, 2.0);
            let b = candidate(1, 3.048, 0.0, 2.0);

            let gaps = measure_gaps(&[a, b], &flat_ground(), &GapConfig::default());

            assert_eq!(gaps.len(), 1);
            let g = &gaps[0];
            assert_eq!(g.feature_a_id, 0);
            assert_eq!(g.feature_b_id, 1);

            // 3.048m * 3.28084 = 9.9999... ≈ 10.0
            assert!(
                (g.centroid_distance_ft - 10.0).abs() < 0.01,
                "centroid_distance_ft: {}, expected ≈10.0",
                g.centroid_distance_ft
            );
            // 10.0 - (1.0 + 1.0) = 8.0
            assert!(
                (g.clear_width_ft - 8.0).abs() < 0.01,
                "clear_width_ft: {}, expected ≈8.0",
                g.clear_width_ft
            );
            // min(2.0, 2.0) = 2.0
            assert!(
                (g.clear_length_ft - 2.0).abs() < 0.01,
                "clear_length_ft: {}, expected ≈2.0",
                g.clear_length_ft
            );
            // 8.0 * 2.0 = 16.0
            assert!(
                (g.area_sqft - 16.0).abs() < 0.1,
                "area_sqft: {}, expected ≈16.0",
                g.area_sqft
            );

            // Midpoint: (1.524, 0.0) meters
            assert!((g.midpoint[0] - 1.524).abs() < 0.001);
            assert!(g.midpoint[1].abs() < 0.001);
        });
    }

    #[test]
    fn test_overlapping_features() {
        timed(|| {
            // Centroids 0.3m apart, spread=4.0ft each
            // dist = 0.3 * 3.28084 ≈ 0.984ft
            // clear_width = 0.984 - (2.0 + 2.0) = -3.016 → filtered
            let a = candidate(0, 0.0, 0.0, 4.0);
            let b = candidate(1, 0.3, 0.0, 4.0);

            let gaps = measure_gaps(&[a, b], &flat_ground(), &GapConfig::default());
            assert!(gaps.is_empty(), "overlapping features should produce no gaps");
        });
    }

    #[test]
    fn test_beyond_threshold() {
        timed(|| {
            // 20m apart ≈ 65.6ft, threshold is 30ft
            let a = candidate(0, 0.0, 0.0, 2.0);
            let b = candidate(1, 20.0, 0.0, 2.0);

            let gaps = measure_gaps(&[a, b], &flat_ground(), &GapConfig::default());
            assert!(gaps.is_empty(), "distant features should be filtered out");
        });
    }

    #[test]
    fn test_empty_candidates() {
        timed(|| {
            let gaps = measure_gaps(&[], &flat_ground(), &GapConfig::default());
            assert!(gaps.is_empty());
        });
    }

    #[test]
    fn test_single_candidate() {
        timed(|| {
            let a = candidate(0, 0.0, 0.0, 2.0);
            let gaps = measure_gaps(&[a], &flat_ground(), &GapConfig::default());
            assert!(gaps.is_empty());
        });
    }

    #[test]
    fn test_ground_elevation() {
        timed(|| {
            // Ground plane at z = 1.0m: normal=[0,0,1], d=-1.0
            // Elevation at any XY = -(0 + 0 + (-1.0)) / 1.0 = 1.0m = 3.28084ft
            let plane = Plane {
                normal: [0.0, 0.0, 1.0],
                d: -1.0,
            };
            let a = candidate(0, 0.0, 0.0, 2.0);
            let b = candidate(1, 3.048, 0.0, 2.0);

            let gaps = measure_gaps(&[a, b], &plane, &GapConfig::default());
            assert_eq!(gaps.len(), 1);

            // 1.0m * 3.28084 = 3.28084ft
            let expected_elev = 1.0 * M_TO_FT;
            assert!(
                (gaps[0].ground_elevation_ft - expected_elev).abs() < 0.01,
                "elevation: {}, expected: {}",
                gaps[0].ground_elevation_ft,
                expected_elev
            );
        });
    }

    #[test]
    fn test_three_features_pairwise() {
        timed(|| {
            // Three features in a line: 0m, 3.048m, 6.096m (≈0ft, 10ft, 20ft)
            let a = candidate(0, 0.0, 0.0, 2.0);
            let b = candidate(1, 3.048, 0.0, 2.0);
            let c = candidate(2, 6.096, 0.0, 2.0);

            let gaps = measure_gaps(&[a, b, c], &flat_ground(), &GapConfig::default());

            // 3 pairs: (0,1)≈10ft, (1,2)≈10ft, (0,2)≈20ft — all within 30ft
            assert_eq!(gaps.len(), 3, "expected 3 gaps, got {}", gaps.len());

            // Sorted by distance: two ~10ft gaps first, then ~20ft gap
            assert!(gaps[0].centroid_distance_ft < 11.0);
            assert!(gaps[1].centroid_distance_ft < 11.0);
            assert!(gaps[2].centroid_distance_ft > 19.0);

            // The 20ft gap has clear_width = 20 - (1+1) = 18ft
            let far_gap = &gaps[2];
            assert!(
                (far_gap.clear_width_ft - 18.0).abs() < 0.1,
                "far gap clear_width: {}, expected ≈18.0",
                far_gap.clear_width_ft
            );
        });
    }
}
