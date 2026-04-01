//! Embedded data source for known test addresses.
//!
//! [`EmbeddedSource`] implements [`Geocoder`], [`ParcelSource`], and [`CanopySource`]
//! using hardcoded data for SF Bay Area test addresses. This allows the satellite
//! pre-population pipeline to work as pure computation at OneStar integration.

use geo::{coord, LineString, Polygon};
use pt_solar::{Coordinates, LatLngBounds};

use crate::error::SatelliteError;
use crate::traits::{CanopySource, Geocoder, ParcelSource};
use crate::types::{DataSourceLabel, DetectedTree, LotBoundary};

/// Data source backed by hardcoded SF Bay Area test data.
///
/// Supports a single test address: "1234 Noriega St, San Francisco, CA".
/// Address matching is case-insensitive with whitespace normalization.
#[derive(Debug, Clone, Default)]
pub struct EmbeddedSource;

/// The canonical test address.
const TEST_ADDRESS: &str = "1234 noriega st, san francisco, ca";

/// Coordinates for 1234 Noriega St — Inner Sunset, SF.
const TEST_COORDS: Coordinates = Coordinates {
    latitude: 37.7601,
    longitude: -122.4862,
};

fn normalize_address(address: &str) -> String {
    address
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build the test lot polygon for 1234 Noriega St.
///
/// Represents a typical Inner Sunset rectangular lot: ~25 ft × 120 ft = ~3,000 sqft.
/// Coordinates are WGS84 (lng, lat) offset from the center point.
/// Approximately 25 ft wide (east-west) and 120 ft deep (north-south).
fn test_lot_polygon() -> Polygon<f64> {
    // 1 foot ≈ 0.0000027° latitude, ≈ 0.0000034° longitude at SF latitude
    let ft_to_lat = 1.0 / 364_567.0; // ~0.00000274
    let ft_to_lng = 1.0 / 290_854.0; // ~0.00000344 (cos(37.76°) factor)

    let center_lat = TEST_COORDS.latitude;
    let center_lng = TEST_COORDS.longitude;

    // 25 ft wide × 120 ft deep rectangular lot
    let half_w = 12.5 * ft_to_lng;
    let half_d = 60.0 * ft_to_lat;

    Polygon::new(
        LineString::from(vec![
            coord! { x: center_lng - half_w, y: center_lat - half_d },
            coord! { x: center_lng + half_w, y: center_lat - half_d },
            coord! { x: center_lng + half_w, y: center_lat + half_d },
            coord! { x: center_lng - half_w, y: center_lat + half_d },
            coord! { x: center_lng - half_w, y: center_lat - half_d }, // close
        ]),
        vec![],
    )
}

/// Test trees for 1234 Noriega St.
///
/// Three trees typical for SF Inner Sunset residential lots:
/// - Mature Monterey cypress in back yard
/// - Medium Victorian box hedge/tree along side
/// - Small ornamental plum in front yard
fn test_trees() -> Vec<DetectedTree> {
    vec![
        DetectedTree {
            location: Coordinates {
                latitude: 37.7602,
                longitude: -122.4862,
            },
            height_ft: 45.0,
            spread_ft: 25.0,
            confidence: 0.85,
        },
        DetectedTree {
            location: Coordinates {
                latitude: 37.7601,
                longitude: -122.4861,
            },
            height_ft: 20.0,
            spread_ft: 12.0,
            confidence: 0.72,
        },
        DetectedTree {
            location: Coordinates {
                latitude: 37.7600,
                longitude: -122.4862,
            },
            height_ft: 15.0,
            spread_ft: 10.0,
            confidence: 0.68,
        },
    ]
}

impl Geocoder for EmbeddedSource {
    fn geocode(&self, address: &str) -> Result<Coordinates, SatelliteError> {
        let normalized = normalize_address(address);
        if normalized == TEST_ADDRESS {
            Ok(TEST_COORDS)
        } else {
            Err(SatelliteError::AddressNotFound(address.to_string()))
        }
    }
}

impl ParcelSource for EmbeddedSource {
    fn lot_boundary(&self, coords: &Coordinates) -> Result<LotBoundary, SatelliteError> {
        // Accept any coordinates near the test address (within ~500m)
        let lat_diff = (coords.latitude - TEST_COORDS.latitude).abs();
        let lng_diff = (coords.longitude - TEST_COORDS.longitude).abs();
        if lat_diff < 0.005 && lng_diff < 0.005 {
            let polygon = test_lot_polygon();

            // 25 ft × 120 ft = 3,000 sqft
            let area_sqft = 3_000.0;

            Ok(LotBoundary {
                polygon,
                area_sqft,
                source: DataSourceLabel::Embedded,
            })
        } else {
            Err(SatelliteError::NoParcelData {
                lat: coords.latitude,
                lng: coords.longitude,
            })
        }
    }
}

impl CanopySource for EmbeddedSource {
    fn detect_trees(&self, bounds: &LatLngBounds) -> Result<Vec<DetectedTree>, SatelliteError> {
        // Return test trees if bounds overlap with the test location
        let overlaps = bounds.south < TEST_COORDS.latitude + 0.005
            && bounds.north > TEST_COORDS.latitude - 0.005
            && bounds.west < TEST_COORDS.longitude + 0.005
            && bounds.east > TEST_COORDS.longitude - 0.005;

        if overlaps {
            Ok(test_trees())
        } else {
            Err(SatelliteError::CanopyUnavailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    #[test]
    fn geocode_known_address() {
        timed(|| {
            let src = EmbeddedSource;
            let coords = src.geocode("1234 Noriega St, San Francisco, CA").unwrap();
            assert!((coords.latitude - 37.7601).abs() < 0.001);
            assert!((coords.longitude - (-122.4862)).abs() < 0.001);
        });
    }

    #[test]
    fn geocode_case_insensitive() {
        timed(|| {
            let src = EmbeddedSource;
            let coords = src
                .geocode("1234  NORIEGA  ST,  San  Francisco,  CA")
                .unwrap();
            assert!((coords.latitude - 37.7601).abs() < 0.001);
        });
    }

    #[test]
    fn geocode_unknown_address_fails() {
        timed(|| {
            let src = EmbeddedSource;
            let result = src.geocode("999 Unknown Ave, Nowhere, CA");
            assert!(result.is_err());
        });
    }

    #[test]
    fn lot_boundary_near_test_coords() {
        timed(|| {
            let src = EmbeddedSource;
            let boundary = src.lot_boundary(&TEST_COORDS).unwrap();

            // Lot area should be ~3,000 sqft (25 × 120)
            assert!((boundary.area_sqft - 3_000.0).abs() < 1.0);
            assert_eq!(boundary.source, DataSourceLabel::Embedded);

            // Polygon should have 5 points (4 corners + closing)
            let exterior = boundary.polygon.exterior();
            assert_eq!(exterior.0.len(), 5);
        });
    }

    #[test]
    fn lot_boundary_far_coords_fails() {
        timed(|| {
            let src = EmbeddedSource;
            let far = Coordinates {
                latitude: 40.0,
                longitude: -74.0,
            };
            assert!(src.lot_boundary(&far).is_err());
        });
    }

    #[test]
    fn detect_trees_in_test_area() {
        timed(|| {
            let src = EmbeddedSource;
            let bounds = LatLngBounds {
                south: 37.759,
                west: -122.487,
                north: 37.761,
                east: -122.485,
            };
            let trees = src.detect_trees(&bounds).unwrap();
            assert_eq!(trees.len(), 3);

            // All heights should be plausible (10-80 ft)
            for tree in &trees {
                assert!(tree.height_ft >= 10.0 && tree.height_ft <= 80.0);
                assert!(tree.spread_ft >= 5.0 && tree.spread_ft <= 40.0);
                assert!(tree.confidence > 0.0 && tree.confidence <= 1.0);
            }
        });
    }
}
