//! Baseline builder — orchestrates the satellite pre-population pipeline.

use chrono::NaiveDate;
use pt_solar::{radiance_grid, GridConfig, LatLngBounds, METERS_PER_DEGREE_LAT};

use crate::error::SatelliteError;
use crate::traits::{CanopySource, Geocoder, ParcelSource};
use crate::types::ProjectBaseline;

/// Orchestrates address → geocode → parcel → canopy → solar pipeline.
///
/// Takes trait-based data sources for geocoding, parcel lookup, and canopy
/// detection. Sun exposure is always computed via pt-solar (not a trait —
/// it's pure computation with no I/O).
#[derive(Debug)]
pub struct BaselineBuilder<G, P, C> {
    geocoder: G,
    parcel_source: P,
    canopy_source: C,
}

impl<G, P, C> BaselineBuilder<G, P, C>
where
    G: Geocoder,
    P: ParcelSource,
    C: CanopySource,
{
    /// Create a new builder with the given data sources.
    pub fn new(geocoder: G, parcel_source: P, canopy_source: C) -> Self {
        Self {
            geocoder,
            parcel_source,
            canopy_source,
        }
    }

    /// Build a project baseline from an address.
    ///
    /// Pipeline:
    /// 1. Geocode address → coordinates
    /// 2. Look up lot boundary from parcel data
    /// 3. Compute bounding box from coordinates (padded ~100m each direction)
    /// 4. Detect trees from canopy data within bounds
    /// 5. Compute sun exposure grid over the bounds using pt-solar
    /// 6. Assemble into `ProjectBaseline`
    ///
    /// # Errors
    /// Propagates errors from any pipeline stage.
    pub fn build(&self, address: &str) -> Result<ProjectBaseline, SatelliteError> {
        // 1. Geocode
        let coordinates = self.geocoder.geocode(address)?;

        // 2. Lot boundary
        let lot_boundary = self.parcel_source.lot_boundary(&coordinates)?;

        // 3. Bounding box — pad ~100m around the center point for canopy + solar
        let offset_lat = 100.0 / METERS_PER_DEGREE_LAT;
        let lng_scale = METERS_PER_DEGREE_LAT * coordinates.latitude.to_radians().cos();
        let offset_lng = 100.0 / lng_scale;

        let bounds = LatLngBounds {
            south: coordinates.latitude - offset_lat,
            west: coordinates.longitude - offset_lng,
            north: coordinates.latitude + offset_lat,
            east: coordinates.longitude + offset_lng,
        };

        // 4. Tree detection
        let trees = self.canopy_source.detect_trees(&bounds)?;

        // 5. Sun exposure grid — growing season (March–September), coarse resolution
        let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();
        let config = GridConfig {
            resolution_meters: 50.0,
            sample_days: 6,
        };
        let sun_grid = radiance_grid(&bounds, (start, end), &config);

        Ok(ProjectBaseline {
            coordinates,
            lot_boundary,
            trees,
            sun_grid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EmbeddedSource;
    use pt_test_utils::timed;

    #[test]
    fn build_baseline_for_known_address() {
        timed(|| {
            let source = EmbeddedSource;
            let builder = BaselineBuilder::new(source.clone(), source.clone(), source);
            let baseline = builder.build("1234 Noriega St, San Francisco, CA").unwrap();

            // Coordinates should be near Inner Sunset
            assert!((baseline.coordinates.latitude - 37.7601).abs() < 0.01);
            assert!((baseline.coordinates.longitude - (-122.4862)).abs() < 0.01);

            // Lot should have reasonable area
            assert!(baseline.lot_boundary.area_sqft > 1_000.0);
            assert!(baseline.lot_boundary.area_sqft < 15_000.0);

            // Should have detected trees
            assert!(!baseline.trees.is_empty());
            assert!(baseline.trees.len() <= 10);

            // Sun grid should have cells with valid Bay Area growing-season values
            assert!(!baseline.sun_grid.values.is_empty());
            assert_eq!(
                baseline.sun_grid.values.len(),
                (baseline.sun_grid.width * baseline.sun_grid.height) as usize
            );
            for &val in &baseline.sun_grid.values {
                let h = f64::from(val);
                assert!(
                    (6.0..=18.0).contains(&h),
                    "sun hours {h} outside expected range"
                );
            }
        });
    }

    #[test]
    fn build_baseline_unknown_address_fails() {
        timed(|| {
            let source = EmbeddedSource;
            let builder = BaselineBuilder::new(source.clone(), source.clone(), source);
            let result = builder.build("999 Unknown Ave, Nowhere, CA");
            assert!(result.is_err());
        });
    }
}
