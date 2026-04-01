//! Growing season computation from frost dates.
//!
//! The growing season is the frost-free period between last spring frost
//! and first fall frost. It determines which annual crops and vegetables
//! can reach maturity at a given location.

use crate::types::{FrostDates, GrowingSeason};

/// Compute the growing season from frost date information.
///
/// Returns typical (median-to-median), short (conservative), and long
/// (optimistic) estimates of the frost-free period in days.
pub fn growing_season(frost_dates: &FrostDates) -> GrowingSeason {
    let spring = &frost_dates.last_spring_frost;
    let fall = &frost_dates.first_fall_frost;

    GrowingSeason {
        typical_days: season_days(spring.median, fall.median),
        short_days: season_days(spring.late, fall.early),
        long_days: season_days(spring.early, fall.late),
        frost_free_start: spring.median,
        frost_free_end: fall.median,
    }
}

/// Calculate days between spring end and fall start, handling southern hemisphere
/// wraparound where fall DOY < spring DOY.
fn season_days(spring_doy: u16, fall_doy: u16) -> u16 {
    if fall_doy >= spring_doy {
        fall_doy - spring_doy
    } else {
        // Southern hemisphere: fall frost in March (DOY ~80), spring frost ends in October (DOY ~280)
        (366 - spring_doy) + fall_doy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, DayOfYearRange};
    use pt_test_utils::timed;

    fn bay_area_frost() -> FrostDates {
        // SF coastal: spring DOY 100, fall DOY 285
        FrostDates {
            last_spring_frost: DayOfYearRange {
                early: 86,
                median: 100,
                late: 114,
            },
            first_fall_frost: DayOfYearRange {
                early: 271,
                median: 285,
                late: 299,
            },
            confidence: Confidence::Medium,
        }
    }

    #[test]
    fn bay_area_growing_season() {
        timed(|| {
            let gs = growing_season(&bay_area_frost());

            // 285 - 100 = 185 typical days
            assert_eq!(gs.typical_days, 185);
            // 271 - 114 = 157 short days
            assert_eq!(gs.short_days, 157);
            // 299 - 86 = 213 long days
            assert_eq!(gs.long_days, 213);
            assert_eq!(gs.frost_free_start, 100);
            assert_eq!(gs.frost_free_end, 285);
        });
    }

    #[test]
    fn southern_hemisphere_wraparound() {
        timed(|| {
            // Sydney-like: spring frost ends Oct (DOY ~280), fall frost starts May (DOY ~130)
            let frost = FrostDates {
                last_spring_frost: DayOfYearRange {
                    early: 266,
                    median: 280,
                    late: 294,
                },
                first_fall_frost: DayOfYearRange {
                    early: 116,
                    median: 130,
                    late: 144,
                },
                confidence: Confidence::Medium,
            };

            let gs = growing_season(&frost);

            // (366 - 280) + 130 = 86 + 130 = 216 typical days
            assert_eq!(gs.typical_days, 216);
        });
    }

    #[test]
    fn tropical_very_long_season() {
        timed(|| {
            let frost = FrostDates {
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

            let gs = growing_season(&frost);

            // 350 - 15 = 335 typical days
            assert_eq!(gs.typical_days, 335);
            assert!(gs.typical_days > 300);
        });
    }

    #[test]
    fn short_season_northern_latitude() {
        timed(|| {
            // ~50N: spring DOY 150, fall DOY 240
            let frost = FrostDates {
                last_spring_frost: DayOfYearRange {
                    early: 136,
                    median: 150,
                    late: 164,
                },
                first_fall_frost: DayOfYearRange {
                    early: 226,
                    median: 240,
                    late: 254,
                },
                confidence: Confidence::Medium,
            };

            let gs = growing_season(&frost);

            // 240 - 150 = 90 typical days
            assert_eq!(gs.typical_days, 90);
        });
    }
}
