//! Light category classification for horticultural use.
//!
//! Translates sun hours into standard gardening light categories:
//! full sun, part sun, part shade, full shade.

use serde::{Deserialize, Serialize};

/// Standard horticultural light categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LightCategory {
    /// 6+ hours of direct sun.
    FullSun,
    /// 4-6 hours of direct sun.
    PartSun,
    /// 2-4 hours of direct sun.
    PartShade,
    /// Less than 2 hours of direct sun.
    FullShade,
}

impl LightCategory {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::FullSun => "Full Sun",
            Self::PartSun => "Part Sun",
            Self::PartShade => "Part Shade",
            Self::FullShade => "Full Shade",
        }
    }

    /// Sun hours range description.
    pub fn sun_hours_range(self) -> &'static str {
        match self {
            Self::FullSun => "6+ hours",
            Self::PartSun => "4-6 hours",
            Self::PartShade => "2-4 hours",
            Self::FullShade => "<2 hours",
        }
    }
}

/// Classifies sun hours into a light category.
///
/// Thresholds: full sun (6+), part sun (4-6), part shade (2-4), full shade (<2).
pub fn classify(sun_hours: f64) -> LightCategory {
    if sun_hours >= 6.0 {
        LightCategory::FullSun
    } else if sun_hours >= 4.0 {
        LightCategory::PartSun
    } else if sun_hours >= 2.0 {
        LightCategory::PartShade
    } else {
        LightCategory::FullShade
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    #[test]
    fn full_sun_at_boundary() {
        timed(|| {
            assert_eq!(classify(6.0), LightCategory::FullSun);
            assert_eq!(classify(10.0), LightCategory::FullSun);
            assert_eq!(classify(24.0), LightCategory::FullSun);
        });
    }

    #[test]
    fn part_sun_range() {
        timed(|| {
            assert_eq!(classify(5.99), LightCategory::PartSun);
            assert_eq!(classify(4.0), LightCategory::PartSun);
            assert_eq!(classify(5.0), LightCategory::PartSun);
        });
    }

    #[test]
    fn part_shade_range() {
        timed(|| {
            assert_eq!(classify(3.99), LightCategory::PartShade);
            assert_eq!(classify(2.0), LightCategory::PartShade);
            assert_eq!(classify(3.0), LightCategory::PartShade);
        });
    }

    #[test]
    fn full_shade_below_two() {
        timed(|| {
            assert_eq!(classify(1.99), LightCategory::FullShade);
            assert_eq!(classify(0.0), LightCategory::FullShade);
            assert_eq!(classify(1.0), LightCategory::FullShade);
        });
    }

    #[test]
    fn labels_and_ranges() {
        timed(|| {
            assert_eq!(LightCategory::FullSun.label(), "Full Sun");
            assert_eq!(LightCategory::FullShade.sun_hours_range(), "<2 hours");
        });
    }
}
