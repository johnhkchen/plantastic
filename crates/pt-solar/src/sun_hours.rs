//! Daily sun hours integration via 5-minute sampling.
//!
//! Computes total hours of sunlight for a single day by sampling solar
//! altitude at regular intervals and counting intervals where the sun
//! is above the horizon.

use chrono::NaiveDate;

use crate::position::sun_position;
use crate::types::{
    Coordinates, DailySunData, PolarCondition, SAMPLES_PER_DAY, SAMPLING_INTERVAL_MINUTES,
};

/// Computes sun hours for a single day at the specified location.
///
/// Samples solar altitude at 5-minute intervals throughout the 24-hour
/// UTC day. Each interval where altitude > 0 contributes to the total.
pub fn daily_sun_hours(coords: &Coordinates, date: NaiveDate) -> DailySunData {
    let start_of_day = date.and_hms_opt(0, 0, 0).unwrap();
    let interval_secs = i64::from(SAMPLING_INTERVAL_MINUTES) * 60;

    let mut positive_count: u32 = 0;

    for i in 0..SAMPLES_PER_DAY {
        let secs_offset = i64::from(i) * interval_secs;
        let sample_time = start_of_day + chrono::TimeDelta::seconds(secs_offset);
        let pos = sun_position(coords, sample_time);

        if pos.altitude_degrees > 0.0 {
            positive_count += 1;
        }
    }

    let polar_condition = if positive_count == SAMPLES_PER_DAY {
        PolarCondition::MidnightSun
    } else if positive_count == 0 {
        PolarCondition::PolarNight
    } else {
        PolarCondition::Normal
    };

    let sun_hours = f64::from(positive_count) * f64::from(SAMPLING_INTERVAL_MINUTES) / 60.0;

    DailySunData {
        date,
        sun_hours,
        polar_condition,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn tromso() -> Coordinates {
        Coordinates {
            latitude: 69.6492,
            longitude: 18.9553,
        }
    }

    #[test]
    fn portland_summer_solstice() {
        timed(|| {
            // Reference: timeanddate.com Portland June 20, 2024 — day length ~15h 41m
            let date = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
            let result = daily_sun_hours(&portland(), date);

            assert!(result.sun_hours > 15.0, "too low: {}", result.sun_hours);
            assert!(result.sun_hours < 16.5, "too high: {}", result.sun_hours);
            assert_eq!(result.polar_condition, PolarCondition::Normal);
        });
    }

    #[test]
    fn portland_winter_solstice() {
        timed(|| {
            // Reference: timeanddate.com Portland Dec 21, 2024 — day length ~8h 42m
            let date = NaiveDate::from_ymd_opt(2024, 12, 21).unwrap();
            let result = daily_sun_hours(&portland(), date);

            assert!(result.sun_hours > 8.0, "too low: {}", result.sun_hours);
            assert!(result.sun_hours < 9.5, "too high: {}", result.sun_hours);
            assert_eq!(result.polar_condition, PolarCondition::Normal);
        });
    }

    #[test]
    fn portland_spring_equinox() {
        timed(|| {
            // Reference: ~12h 9m
            let date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
            let result = daily_sun_hours(&portland(), date);

            assert!(result.sun_hours > 11.5, "too low: {}", result.sun_hours);
            assert!(result.sun_hours < 12.75, "too high: {}", result.sun_hours);
            assert_eq!(result.polar_condition, PolarCondition::Normal);
        });
    }

    #[test]
    fn portland_fall_equinox() {
        timed(|| {
            // Reference: ~12h 8m
            let date = NaiveDate::from_ymd_opt(2024, 9, 22).unwrap();
            let result = daily_sun_hours(&portland(), date);

            assert!(result.sun_hours > 11.5, "too low: {}", result.sun_hours);
            assert!(result.sun_hours < 12.75, "too high: {}", result.sun_hours);
            assert_eq!(result.polar_condition, PolarCondition::Normal);
        });
    }

    #[test]
    fn singapore_stable_year_round() {
        timed(|| {
            let summer =
                daily_sun_hours(&singapore(), NaiveDate::from_ymd_opt(2024, 6, 20).unwrap());
            let winter =
                daily_sun_hours(&singapore(), NaiveDate::from_ymd_opt(2024, 12, 21).unwrap());

            // Near-equatorial: ~12h year-round
            assert!(
                summer.sun_hours > 11.5 && summer.sun_hours < 12.5,
                "summer: {}",
                summer.sun_hours
            );
            assert!(
                winter.sun_hours > 11.5 && winter.sun_hours < 12.5,
                "winter: {}",
                winter.sun_hours
            );

            // Minimal variation
            assert!(
                (summer.sun_hours - winter.sun_hours).abs() < 0.5,
                "variation too large: {} vs {}",
                summer.sun_hours,
                winter.sun_hours
            );
        });
    }

    #[test]
    fn tromso_midnight_sun() {
        timed(|| {
            let date = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
            let result = daily_sun_hours(&tromso(), date);

            assert_eq!(result.sun_hours, 24.0);
            assert_eq!(result.polar_condition, PolarCondition::MidnightSun);
        });
    }

    #[test]
    fn tromso_polar_night() {
        timed(|| {
            let date = NaiveDate::from_ymd_opt(2024, 12, 21).unwrap();
            let result = daily_sun_hours(&tromso(), date);

            assert_eq!(result.sun_hours, 0.0);
            assert_eq!(result.polar_condition, PolarCondition::PolarNight);
        });
    }
}
