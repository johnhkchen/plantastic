//! Climate data engine for Plantastic.
//!
//! Provides frost dates, USDA hardiness zones, Sunset Western Garden zones,
//! and growing season computation. Pure computation with embedded lookup
//! tables — no I/O at query time.
//!
//! Ported from the solar-sim TypeScript prototype's climate module.

pub mod frost;
pub mod growing_season;
pub mod hardiness;
pub mod sunset;
pub mod types;

// Re-export key types and functions at crate root.
pub use frost::frost_dates;
pub use growing_season::growing_season;
pub use hardiness::hardiness_zone;
pub use sunset::sunset_zone;
pub use types::*;

/// Build a complete climate profile for a location.
///
/// Convenience function that calls all four climate lookups and assembles
/// the results into a single `ClimateProfile`.
pub fn climate_profile(
    coords: &Coordinates,
    elevation_m: Option<f64>,
    coastal: bool,
) -> ClimateProfile {
    let fd = frost_dates(coords, elevation_m, coastal);
    let hz = hardiness_zone(coords, elevation_m, coastal);
    let sz = sunset_zone(coords);
    let gs = growing_season(&fd);

    ClimateProfile {
        frost_dates: fd,
        hardiness_zone: hz,
        sunset_zone: sz,
        growing_season: gs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    #[test]
    fn climate_profile_sf() {
        timed(|| {
            let sf = Coordinates {
                latitude: 37.7749,
                longitude: -122.4194,
            };
            let profile = climate_profile(&sf, None, true);

            // Frost dates exist
            assert!(profile.frost_dates.last_spring_frost.median > 0);
            assert!(profile.frost_dates.first_fall_frost.median > 0);

            // Hardiness zone is populated
            assert!(profile.hardiness_zone.zone >= 1);
            assert!(profile.hardiness_zone.zone <= 13);

            // SF should have a Sunset zone (17)
            let sz = profile.sunset_zone.expect("SF should have Sunset zone");
            assert_eq!(sz.zone, 17);

            // Growing season is positive
            assert!(profile.growing_season.typical_days > 100);
        });
    }

    #[test]
    fn climate_profile_outside_bay_area() {
        timed(|| {
            let portland = Coordinates {
                latitude: 45.5152,
                longitude: -122.6784,
            };
            let profile = climate_profile(&portland, None, true);

            // Should have frost dates and hardiness zone
            assert!(profile.frost_dates.last_spring_frost.median > 0);
            assert!(profile.hardiness_zone.zone >= 1);

            // No Sunset zone outside Bay Area
            assert!(profile.sunset_zone.is_none());
        });
    }
}
