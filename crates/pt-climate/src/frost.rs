//! Frost date lookup by latitude band with elevation and coastal modifiers.
//!
//! Ported from solar-sim TypeScript prototype. Uses embedded lookup tables
//! derived from NOAA climate normals.

use crate::types::{
    Confidence, DayOfYearRange, FrostDates, FrostLookupEntry, ELEVATION_DAYS_PER_300M,
};
use crate::Coordinates;

/// Northern hemisphere frost date table (2.5-degree latitude bands, 20N-70N).
/// Values represent median frost dates for continental interior locations at sea level.
const NORTHERN_FROST_TABLE: &[FrostLookupEntry] = &[
    // Tropical/subtropical — minimal frost risk
    FrostLookupEntry {
        lat_min: 20.0,
        lat_max: 25.0,
        coastal_modifier_days: -5,
        last_spring_doy: 32,
        first_fall_doy: 350,
        variance_days: 14,
    },
    // Deep South — very mild winters
    FrostLookupEntry {
        lat_min: 25.0,
        lat_max: 27.5,
        coastal_modifier_days: -7,
        last_spring_doy: 45,
        first_fall_doy: 335,
        variance_days: 14,
    },
    // Gulf states
    FrostLookupEntry {
        lat_min: 27.5,
        lat_max: 30.0,
        coastal_modifier_days: -7,
        last_spring_doy: 60,
        first_fall_doy: 320,
        variance_days: 14,
    },
    // Lower South
    FrostLookupEntry {
        lat_min: 30.0,
        lat_max: 32.5,
        coastal_modifier_days: -7,
        last_spring_doy: 75,
        first_fall_doy: 305,
        variance_days: 14,
    },
    // Upper South
    FrostLookupEntry {
        lat_min: 32.5,
        lat_max: 35.0,
        coastal_modifier_days: -10,
        last_spring_doy: 90,
        first_fall_doy: 295,
        variance_days: 14,
    },
    // Mid-Atlantic / Border states
    FrostLookupEntry {
        lat_min: 35.0,
        lat_max: 37.5,
        coastal_modifier_days: -10,
        last_spring_doy: 100,
        first_fall_doy: 285,
        variance_days: 14,
    },
    // Central states — SF Bay Area falls in this band (37.5-40N)
    FrostLookupEntry {
        lat_min: 37.5,
        lat_max: 40.0,
        coastal_modifier_days: -10,
        last_spring_doy: 110,
        first_fall_doy: 275,
        variance_days: 14,
    },
    // Northern tier lower
    FrostLookupEntry {
        lat_min: 40.0,
        lat_max: 42.5,
        coastal_modifier_days: -10,
        last_spring_doy: 120,
        first_fall_doy: 265,
        variance_days: 14,
    },
    // Northern tier upper
    FrostLookupEntry {
        lat_min: 42.5,
        lat_max: 45.0,
        coastal_modifier_days: -10,
        last_spring_doy: 130,
        first_fall_doy: 255,
        variance_days: 14,
    },
    // Northern plains / New England
    FrostLookupEntry {
        lat_min: 45.0,
        lat_max: 47.5,
        coastal_modifier_days: -10,
        last_spring_doy: 140,
        first_fall_doy: 250,
        variance_days: 14,
    },
    // Upper northern tier
    FrostLookupEntry {
        lat_min: 47.5,
        lat_max: 50.0,
        coastal_modifier_days: -10,
        last_spring_doy: 145,
        first_fall_doy: 245,
        variance_days: 14,
    },
    // Southern Canada
    FrostLookupEntry {
        lat_min: 50.0,
        lat_max: 52.5,
        coastal_modifier_days: -10,
        last_spring_doy: 150,
        first_fall_doy: 240,
        variance_days: 14,
    },
    // Central Canada
    FrostLookupEntry {
        lat_min: 52.5,
        lat_max: 55.0,
        coastal_modifier_days: -10,
        last_spring_doy: 155,
        first_fall_doy: 235,
        variance_days: 14,
    },
    // Northern Canada / Alaska
    FrostLookupEntry {
        lat_min: 55.0,
        lat_max: 60.0,
        coastal_modifier_days: -7,
        last_spring_doy: 160,
        first_fall_doy: 230,
        variance_days: 14,
    },
    // Far northern
    FrostLookupEntry {
        lat_min: 60.0,
        lat_max: 70.0,
        coastal_modifier_days: -5,
        last_spring_doy: 170,
        first_fall_doy: 220,
        variance_days: 14,
    },
];

/// Southern hemisphere frost table (seasons inverted).
const SOUTHERN_FROST_TABLE: &[FrostLookupEntry] = &[
    // Subtropical southern hemisphere
    FrostLookupEntry {
        lat_min: -25.0,
        lat_max: -20.0,
        coastal_modifier_days: -5,
        last_spring_doy: 244,
        first_fall_doy: 166,
        variance_days: 14,
    },
    // Temperate
    FrostLookupEntry {
        lat_min: -30.0,
        lat_max: -25.0,
        coastal_modifier_days: -7,
        last_spring_doy: 258,
        first_fall_doy: 152,
        variance_days: 14,
    },
    // Cool temperate
    FrostLookupEntry {
        lat_min: -35.0,
        lat_max: -30.0,
        coastal_modifier_days: -7,
        last_spring_doy: 273,
        first_fall_doy: 135,
        variance_days: 14,
    },
    // Cold temperate
    FrostLookupEntry {
        lat_min: -40.0,
        lat_max: -35.0,
        coastal_modifier_days: -10,
        last_spring_doy: 288,
        first_fall_doy: 121,
        variance_days: 14,
    },
    // Sub-Antarctic
    FrostLookupEntry {
        lat_min: -50.0,
        lat_max: -40.0,
        coastal_modifier_days: -10,
        last_spring_doy: 305,
        first_fall_doy: 105,
        variance_days: 14,
    },
    // Antarctic fringe
    FrostLookupEntry {
        lat_min: -60.0,
        lat_max: -50.0,
        coastal_modifier_days: -5,
        last_spring_doy: 320,
        first_fall_doy: 91,
        variance_days: 14,
    },
];

/// Look up frost dates for a geographic location.
///
/// Uses latitude-band tables with elevation and coastal adjustments.
/// Returns day-of-year ranges for last spring frost and first fall frost.
///
/// - `elevation_m`: meters above sea level (higher = later spring, earlier fall frost)
/// - `coastal`: whether the location is near the coast (moderates frost risk)
pub fn frost_dates(coords: &Coordinates, elevation_m: Option<f64>, coastal: bool) -> FrostDates {
    let abs_lat = coords.latitude.abs();

    // Tropical: minimal frost
    if abs_lat < 20.0 {
        return FrostDates {
            last_spring_frost: DayOfYearRange {
                early: 1,
                median: 15,
                late: 32,
            },
            first_fall_frost: DayOfYearRange {
                early: 335,
                median: 350,
                late: 366,
            },
            confidence: Confidence::Low,
        };
    }

    let entry = find_entry(coords.latitude);

    let mut spring_doy: i32 = i32::from(entry.last_spring_doy);
    let mut fall_doy: i32 = i32::from(entry.first_fall_doy);

    // Elevation adjustment: later spring, earlier fall
    if let Some(elev) = elevation_m {
        if elev > 0.0 {
            #[allow(clippy::cast_possible_truncation)] // elev/300 * 4 is always small
            let adj = ((elev / 300.0) * ELEVATION_DAYS_PER_300M).round() as i32;
            spring_doy += adj;
            fall_doy -= adj;
        }
    }

    // Coastal modifier: earlier spring, later fall (modifier is negative)
    if coastal {
        spring_doy += i32::from(entry.coastal_modifier_days);
        fall_doy -= i32::from(entry.coastal_modifier_days);
    }

    // Clamp to valid range — values are guaranteed 1..=366 after clamp
    spring_doy = spring_doy.clamp(1, 366);
    fall_doy = fall_doy.clamp(1, 366);

    let variance = entry.variance_days;

    let confidence = if !(25.0..=55.0).contains(&abs_lat) {
        Confidence::Low
    } else {
        Confidence::Medium
    };

    // Safe: clamped to 1..=366 which fits in u16
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let spring = spring_doy as u16;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let fall = fall_doy as u16;

    FrostDates {
        last_spring_frost: day_range(spring, variance),
        first_fall_frost: day_range(fall, variance),
        confidence,
    }
}

fn find_entry(latitude: f64) -> FrostLookupEntry {
    let abs_lat = latitude.abs();
    let table = if latitude >= 0.0 {
        NORTHERN_FROST_TABLE
    } else {
        SOUTHERN_FROST_TABLE
    };

    for entry in table {
        let entry_min = entry.lat_min.abs().min(entry.lat_max.abs());
        let entry_max = entry.lat_min.abs().max(entry.lat_max.abs());
        if abs_lat >= entry_min && abs_lat < entry_max {
            return *entry;
        }
    }

    // Beyond table range — use the extreme entry
    *table.last().expect("frost table is non-empty")
}

fn day_range(median: u16, variance: u16) -> DayOfYearRange {
    DayOfYearRange {
        early: median.saturating_sub(variance).max(1),
        median,
        late: (median + variance).min(366),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    // SF: 37.7749N, coastal. Falls in 37.5-40 band.
    // Base: spring DOY 110, fall DOY 275.
    // Coastal modifier: -10 → spring 100, fall 285.
    #[test]
    fn sf_frost_dates_coastal() {
        timed(|| {
            let sf = Coordinates {
                latitude: 37.7749,
                longitude: -122.4194,
            };
            let fd = frost_dates(&sf, None, true);

            // Median spring frost ~ DOY 100 (early April)
            assert_eq!(fd.last_spring_frost.median, 100);
            // Median fall frost ~ DOY 285 (mid October)
            assert_eq!(fd.first_fall_frost.median, 285);
            assert_eq!(fd.confidence, Confidence::Medium);
        });
    }

    // San Jose: 37.3382N, inland. Falls in 35-37.5 band.
    // Base: spring DOY 100, fall DOY 285. Not coastal.
    #[test]
    fn san_jose_frost_dates_inland() {
        timed(|| {
            let sj = Coordinates {
                latitude: 37.3382,
                longitude: -121.8863,
            };
            let fd = frost_dates(&sj, None, false);

            assert_eq!(fd.last_spring_frost.median, 100);
            assert_eq!(fd.first_fall_frost.median, 285);
            assert_eq!(fd.confidence, Confidence::Medium);
        });
    }

    // Oakland: 37.8044N, coastal. Same band as SF (37.5-40).
    #[test]
    fn oakland_frost_dates_coastal() {
        timed(|| {
            let oak = Coordinates {
                latitude: 37.8044,
                longitude: -122.2712,
            };
            let fd = frost_dates(&oak, None, true);

            assert_eq!(fd.last_spring_frost.median, 100);
            assert_eq!(fd.first_fall_frost.median, 285);
        });
    }

    // Elevation adjustment: 600m = 2 × 300m = +8 days spring, -8 days fall
    #[test]
    fn elevation_shifts_frost_dates() {
        timed(|| {
            let loc = Coordinates {
                latitude: 37.7749,
                longitude: -122.4194,
            };
            let fd_low = frost_dates(&loc, None, false);
            let fd_high = frost_dates(&loc, Some(600.0), false);

            assert_eq!(
                fd_high.last_spring_frost.median,
                fd_low.last_spring_frost.median + 8
            );
            assert_eq!(
                fd_high.first_fall_frost.median,
                fd_low.first_fall_frost.median - 8
            );
        });
    }

    // Tropical: low confidence, near year-round frost-free
    #[test]
    fn tropical_returns_minimal_frost() {
        timed(|| {
            let equatorial = Coordinates {
                latitude: 5.0,
                longitude: 0.0,
            };
            let fd = frost_dates(&equatorial, None, false);

            assert_eq!(fd.confidence, Confidence::Low);
            assert!(fd.last_spring_frost.median < 30);
            assert!(fd.first_fall_frost.median > 330);
        });
    }

    // Variance creates early/late bounds
    #[test]
    fn variance_creates_range() {
        timed(|| {
            let sf = Coordinates {
                latitude: 37.7749,
                longitude: -122.4194,
            };
            let fd = frost_dates(&sf, None, true);

            assert_eq!(fd.last_spring_frost.early, fd.last_spring_frost.median - 14);
            assert_eq!(fd.last_spring_frost.late, fd.last_spring_frost.median + 14);
        });
    }
}
