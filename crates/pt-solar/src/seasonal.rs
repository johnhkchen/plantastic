//! Date range aggregation of sun hours.
//!
//! Computes summary statistics (average, min, max) across arbitrary
//! date ranges by calling daily_sun_hours for each day.

use chrono::NaiveDate;

use crate::sun_hours::daily_sun_hours;
use crate::types::{Coordinates, PolarCondition, SeasonalSummary};

/// Computes sun hours for every day in a date range (inclusive) and returns
/// aggregate statistics.
pub fn annual_sun_hours(coords: &Coordinates, start: NaiveDate, end: NaiveDate) -> SeasonalSummary {
    let mut daily_data = Vec::new();
    let mut current = start;

    while current <= end {
        daily_data.push(daily_sun_hours(coords, current));
        current = current.succ_opt().unwrap();
    }

    #[allow(clippy::cast_possible_truncation)] // date ranges won't exceed u32
    let total_days = daily_data.len() as u32;

    if daily_data.is_empty() {
        return SeasonalSummary {
            start,
            end,
            average_sun_hours: 0.0,
            min_sun_hours: 0.0,
            max_sun_hours: 0.0,
            total_days: 0,
            days_of_midnight_sun: 0,
            days_of_polar_night: 0,
            daily_data,
        };
    }

    let mut sum = 0.0;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut midnight_sun_count = 0u32;
    let mut polar_night_count = 0u32;

    for day in &daily_data {
        sum += day.sun_hours;
        if day.sun_hours < min {
            min = day.sun_hours;
        }
        if day.sun_hours > max {
            max = day.sun_hours;
        }
        if day.polar_condition == PolarCondition::MidnightSun {
            midnight_sun_count += 1;
        }
        if day.polar_condition == PolarCondition::PolarNight {
            polar_night_count += 1;
        }
    }

    SeasonalSummary {
        start,
        end,
        average_sun_hours: sum / f64::from(total_days),
        min_sun_hours: min,
        max_sun_hours: max,
        total_days,
        days_of_midnight_sun: midnight_sun_count,
        days_of_polar_night: polar_night_count,
        daily_data,
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

    #[test]
    fn portland_growing_season() {
        timed(|| {
            // March through September — typical growing season
            let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            let end = NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();
            let summary = annual_sun_hours(&portland(), start, end);

            // Average should be well above 12 hours in growing season
            assert!(
                summary.average_sun_hours > 12.0,
                "average too low: {}",
                summary.average_sun_hours
            );
            assert!(
                summary.average_sun_hours < 16.0,
                "average too high: {}",
                summary.average_sun_hours
            );

            // Max should be near summer solstice (~15.5h)
            assert!(
                summary.max_sun_hours > 15.0,
                "max too low: {}",
                summary.max_sun_hours
            );

            // No polar conditions in Portland
            assert_eq!(summary.days_of_midnight_sun, 0);
            assert_eq!(summary.days_of_polar_night, 0);
        });
    }

    #[test]
    fn short_range() {
        timed(|| {
            let date = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
            let summary = annual_sun_hours(&portland(), date, date);

            assert_eq!(summary.total_days, 1);
            assert_eq!(summary.daily_data.len(), 1);
            // Single day: min == max == average
            assert!((summary.min_sun_hours - summary.max_sun_hours).abs() < f64::EPSILON);
        });
    }
}
