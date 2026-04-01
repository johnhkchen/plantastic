//! Conversion from [`pt_satellite::ProjectBaseline`] to BAML [`SatelliteBaseline`].

use crate::baml_client::types::{SatelliteBaseline, SatelliteTree};

/// Summarize a full project baseline into the compact form the BAML prompt expects.
///
/// Computes `avg_sun_hours` as the arithmetic mean of the exposure grid values.
/// Drops polygon geometry and per-cell grid data — the LLM doesn't need them.
pub fn summarize_baseline(baseline: &pt_satellite::ProjectBaseline) -> SatelliteBaseline {
    let trees = baseline
        .trees
        .iter()
        .map(|t| SatelliteTree {
            height_ft: t.height_ft,
            spread_ft: t.spread_ft,
            confidence: t.confidence,
        })
        .collect();

    let avg_sun_hours = if baseline.sun_grid.values.is_empty() {
        0.0
    } else {
        let sum: f64 = baseline.sun_grid.values.iter().map(|&v| f64::from(v)).sum();
        sum / baseline.sun_grid.values.len() as f64
    };

    SatelliteBaseline {
        lot_area_sqft: baseline.lot_boundary.area_sqft,
        trees,
        avg_sun_hours,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::Polygon;
    use pt_satellite::types::{DataSourceLabel, DetectedTree, LotBoundary};
    use pt_solar::{Coordinates, ExposureGrid, LatLngBounds};
    use pt_test_utils::timed;

    fn test_baseline() -> pt_satellite::ProjectBaseline {
        pt_satellite::ProjectBaseline {
            coordinates: Coordinates {
                latitude: 37.7849,
                longitude: -122.4094,
            },
            lot_boundary: LotBoundary {
                polygon: Polygon::new(geo::LineString::new(vec![]), vec![]),
                area_sqft: 1000.0,
                source: DataSourceLabel::Embedded,
            },
            trees: vec![
                DetectedTree {
                    location: Coordinates {
                        latitude: 37.7849,
                        longitude: -122.4094,
                    },
                    height_ft: 30.0,
                    spread_ft: 15.0,
                    confidence: 0.78,
                },
                DetectedTree {
                    location: Coordinates {
                        latitude: 37.7850,
                        longitude: -122.4095,
                    },
                    height_ft: 28.0,
                    spread_ft: 12.0,
                    confidence: 0.72,
                },
            ],
            sun_grid: ExposureGrid {
                bounds: LatLngBounds {
                    south: 37.784,
                    west: -122.410,
                    north: 37.786,
                    east: -122.408,
                },
                resolution_meters: 50.0,
                width: 2,
                height: 2,
                values: vec![8.0, 9.0, 7.0, 10.0],
                sample_days_used: 6,
            },
        }
    }

    #[test]
    fn summarize_preserves_lot_area() {
        timed(|| {
            let summary = summarize_baseline(&test_baseline());
            // 1000.0 sq ft from test fixture — hand-verified, not system-derived.
            assert!((summary.lot_area_sqft - 1000.0).abs() < f64::EPSILON);
        });
    }

    #[test]
    fn summarize_maps_trees() {
        timed(|| {
            let summary = summarize_baseline(&test_baseline());
            assert_eq!(summary.trees.len(), 2);
            assert!((summary.trees[0].height_ft - 30.0).abs() < f64::EPSILON);
            assert!((summary.trees[1].spread_ft - 12.0).abs() < f64::EPSILON);
        });
    }

    #[test]
    fn summarize_computes_avg_sun_hours() {
        timed(|| {
            let summary = summarize_baseline(&test_baseline());
            // Grid values: [8.0, 9.0, 7.0, 10.0]. Mean = 34.0 / 4 = 8.5.
            assert!((summary.avg_sun_hours - 8.5).abs() < f64::EPSILON);
        });
    }

    #[test]
    fn summarize_empty_grid_returns_zero() {
        timed(|| {
            let mut baseline = test_baseline();
            baseline.sun_grid.values.clear();
            let summary = summarize_baseline(&baseline);
            assert!((summary.avg_sun_hours - 0.0).abs() < f64::EPSILON);
        });
    }
}
