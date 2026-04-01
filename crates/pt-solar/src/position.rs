//! Sun position calculation using simplified NOAA solar position formulas.
//!
//! Computes altitude (degrees above horizon) and azimuth (compass bearing)
//! for any geographic location and UTC datetime. The algorithm uses the same
//! approach as SunCalc.js, which itself is based on NOAA's Solar Calculator.

use chrono::NaiveDateTime;

use crate::types::{Coordinates, SolarPosition};

const PI: f64 = std::f64::consts::PI;
const RAD: f64 = PI / 180.0;

/// Returns the sun's position for a specific location and UTC datetime.
pub fn sun_position(coords: &Coordinates, dt: NaiveDateTime) -> SolarPosition {
    let lat_rad = coords.latitude * RAD;
    let lng = coords.longitude;

    // Days since J2000.0 epoch (2000-01-01 12:00 UTC)
    let d = julian_days_since_j2000(dt);

    // Solar coordinates
    let (dec, ra) = solar_coordinates(d);

    // Sidereal time and hour angle
    let lw = -lng * RAD;
    let sidereal = sidereal_time(d, lw);
    let ha = sidereal - ra;

    // Altitude: angle above the horizon
    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();
    let sin_dec = dec.sin();
    let cos_dec = dec.cos();
    let cos_ha = ha.cos();

    let altitude = (sin_lat * sin_dec + cos_lat * cos_dec * cos_ha).asin();

    // Azimuth: compass bearing from north
    let azimuth = ha.sin().atan2(cos_ha * sin_lat - dec.tan() * cos_lat) + PI;

    SolarPosition {
        altitude_degrees: altitude / RAD,
        azimuth_degrees: normalize_angle(azimuth / RAD),
    }
}

/// Days since J2000.0 epoch (2000-01-01 12:00:00 UTC).
fn julian_days_since_j2000(dt: NaiveDateTime) -> f64 {
    // Julian Day Number for 2000-01-01 12:00 UTC = 2451545.0
    let timestamp = dt.and_utc().timestamp() as f64;
    // Unix epoch (1970-01-01) is JD 2440587.5
    // So JD = timestamp / 86400 + 2440587.5
    // Days since J2000 = JD - 2451545.0
    timestamp / 86400.0 - 10957.5
}

/// Solar declination and right ascension (both in radians).
fn solar_coordinates(d: f64) -> (f64, f64) {
    let m = solar_mean_anomaly(d);
    let l = ecliptic_longitude(m);
    let dec = declination(l, 0.0);
    let ra = right_ascension(l, 0.0);
    (dec, ra)
}

/// Solar mean anomaly in radians.
fn solar_mean_anomaly(d: f64) -> f64 {
    (357.5291 + 0.985_600_28 * d) * RAD
}

/// Ecliptic longitude in radians.
fn ecliptic_longitude(m: f64) -> f64 {
    // Equation of the center
    let c = (1.9148 * m.sin() + 0.02 * (2.0 * m).sin() + 0.0003 * (3.0 * m).sin()) * RAD;
    // Perihelion of Earth
    let p = 102.9372 * RAD;
    m + c + p + PI
}

/// Declination from ecliptic longitude and latitude (both radians).
fn declination(l: f64, _b: f64) -> f64 {
    let obliquity = 23.4393 * RAD; // Earth's axial tilt
    (l.sin() * obliquity.sin()).asin()
}

/// Right ascension from ecliptic longitude and latitude (both radians).
fn right_ascension(l: f64, _b: f64) -> f64 {
    let obliquity = 23.4393 * RAD;
    (l.sin() * obliquity.cos()).atan2(l.cos())
}

/// Sidereal time at the observer's longitude (radians).
fn sidereal_time(d: f64, lw: f64) -> f64 {
    (280.16 + 360.985_623_5 * d) * RAD - lw
}

/// Normalize angle to 0..360 degrees.
fn normalize_angle(deg: f64) -> f64 {
    let r = deg % 360.0;
    if r < 0.0 {
        r + 360.0
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use pt_test_utils::timed;

    fn portland() -> Coordinates {
        Coordinates {
            latitude: 45.5152,
            longitude: -122.6784,
        }
    }

    fn singapore() -> Coordinates {
        Coordinates {
            latitude: 1.3521,
            longitude: 103.8198,
        }
    }

    #[test]
    fn summer_solstice_noon_altitude() {
        timed(|| {
            // Portland summer solstice 2024: June 20
            // Approximate solar noon in Portland (UTC-7, ~13:15 local = ~20:15 UTC)
            let dt = NaiveDate::from_ymd_opt(2024, 6, 20)
                .unwrap()
                .and_hms_opt(20, 15, 0)
                .unwrap();
            let pos = sun_position(&portland(), dt);

            // Expected: ~68 degrees (90 - 45.5 + 23.5)
            // Reference: timeanddate.com shows ~68.5 degrees
            assert!(
                pos.altitude_degrees > 65.0,
                "altitude too low: {}",
                pos.altitude_degrees
            );
            assert!(
                pos.altitude_degrees < 72.0,
                "altitude too high: {}",
                pos.altitude_degrees
            );
        });
    }

    #[test]
    fn summer_solstice_noon_azimuth() {
        timed(|| {
            // At solar noon, sun should be roughly due south (180 degrees)
            let dt = NaiveDate::from_ymd_opt(2024, 6, 20)
                .unwrap()
                .and_hms_opt(20, 15, 0)
                .unwrap();
            let pos = sun_position(&portland(), dt);

            assert!(
                pos.azimuth_degrees > 170.0,
                "azimuth too low: {}",
                pos.azimuth_degrees
            );
            assert!(
                pos.azimuth_degrees < 190.0,
                "azimuth too high: {}",
                pos.azimuth_degrees
            );
        });
    }

    #[test]
    fn winter_solstice_noon_altitude() {
        timed(|| {
            // Portland winter solstice: December 21
            // Expected: ~21 degrees (90 - 45.5 - 23.5)
            let dt = NaiveDate::from_ymd_opt(2024, 12, 21)
                .unwrap()
                .and_hms_opt(20, 15, 0)
                .unwrap();
            let pos = sun_position(&portland(), dt);

            assert!(
                pos.altitude_degrees > 18.0,
                "altitude too low: {}",
                pos.altitude_degrees
            );
            assert!(
                pos.altitude_degrees < 25.0,
                "altitude too high: {}",
                pos.altitude_degrees
            );
        });
    }

    #[test]
    fn negative_altitude_at_night() {
        timed(|| {
            // Midnight in Portland (UTC-7, so 07:00 UTC)
            let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
                .unwrap()
                .and_hms_opt(7, 0, 0)
                .unwrap();
            let pos = sun_position(&portland(), dt);

            assert!(
                pos.altitude_degrees < 0.0,
                "expected negative altitude at night: {}",
                pos.altitude_degrees
            );
        });
    }

    #[test]
    fn equatorial_high_noon() {
        timed(|| {
            // Singapore near equinox should have very high noon altitude
            let dt = NaiveDate::from_ymd_opt(2024, 3, 20)
                .unwrap()
                .and_hms_opt(4, 0, 0)
                .unwrap(); // ~noon local (UTC+8)
            let pos = sun_position(&singapore(), dt);

            assert!(
                pos.altitude_degrees > 60.0,
                "equatorial noon altitude too low: {}",
                pos.altitude_degrees
            );
        });
    }
}
