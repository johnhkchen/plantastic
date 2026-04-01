//! Shared types and constants for the climate engine.

use serde::{Deserialize, Serialize};

/// Elevation adjustment: frost date shifts by this many days per 300m of elevation.
pub const ELEVATION_DAYS_PER_300M: f64 = 4.0;

/// USDA zone system constants.
pub const ZONE_BASE_TEMP_F: f64 = -60.0;
pub const ZONE_RANGE_F: f64 = 10.0;
pub const SUBZONE_RANGE_F: f64 = 5.0;
pub const MIN_ZONE: u8 = 1;
pub const MAX_ZONE: u8 = 13;

/// Temperature lapse rate for elevation: degrees F per 300m.
pub const ELEVATION_LAPSE_RATE_F_PER_300M: f64 = 3.5;

/// Geographic coordinates (WGS84).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinates {
    /// Latitude in degrees. Positive north, negative south.
    pub latitude: f64,
    /// Longitude in degrees. Positive east, negative west.
    pub longitude: f64,
}

/// A range of day-of-year values capturing year-to-year variance.
///
/// Day-of-year is 1-based (Jan 1 = 1, Dec 31 = 365 or 366).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayOfYearRange {
    /// Earliest typical occurrence (10th percentile).
    pub early: u16,
    /// Median occurrence.
    pub median: u16,
    /// Latest typical occurrence (90th percentile).
    pub late: u16,
}

/// Confidence level for climate data lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Location is within well-characterized climate region.
    High,
    /// Standard lookup-table interpolation.
    Medium,
    /// Extreme latitude, elevation adjustment, or sparse data.
    Low,
}

/// Frost date information for a location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrostDates {
    /// Last spring frost (day-of-year range).
    pub last_spring_frost: DayOfYearRange,
    /// First fall frost (day-of-year range).
    pub first_fall_frost: DayOfYearRange,
    /// Data quality indicator.
    pub confidence: Confidence,
}

/// USDA Plant Hardiness Zone.
///
/// Zones 1-13, each spanning 10F of average annual minimum winter temperature.
/// Subzones 'a' (colder half) and 'b' (warmer half) span 5F each.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HardinessZone {
    /// Zone number (1-13).
    pub zone: u8,
    /// Subzone: 'a' (colder) or 'b' (warmer).
    pub subzone: char,
    /// Lower bound of this subzone's temperature range (F).
    pub min_temp_f: f64,
    /// Upper bound of this subzone's temperature range (F).
    pub max_temp_f: f64,
}

impl HardinessZone {
    /// Formats as "10b", "9a", etc.
    pub fn label(&self) -> String {
        format!("{}{}", self.zone, self.subzone)
    }
}

/// Sunset Western Garden zone (western US, primarily California).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SunsetZone {
    /// Zone number (14-17 for Bay Area).
    pub zone: u8,
    /// Brief description of the zone's climate character.
    pub description: &'static str,
}

/// Growing season computed from frost dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrowingSeason {
    /// Days between median last spring frost and median first fall frost.
    pub typical_days: u16,
    /// Conservative estimate: late spring to early fall.
    pub short_days: u16,
    /// Optimistic estimate: early spring to late fall.
    pub long_days: u16,
    /// Day-of-year when frost-free period typically starts (median spring frost).
    pub frost_free_start: u16,
    /// Day-of-year when frost-free period typically ends (median fall frost).
    pub frost_free_end: u16,
}

/// Complete climate profile for a location.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ClimateProfile {
    pub frost_dates: FrostDates,
    pub hardiness_zone: HardinessZone,
    /// None if location is outside Sunset zone coverage (i.e., not western US).
    pub sunset_zone: Option<SunsetZone>,
    pub growing_season: GrowingSeason,
}

/// Internal lookup table entry for frost dates by latitude band.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FrostLookupEntry {
    pub lat_min: f64,
    pub lat_max: f64,
    /// Days to shift frost dates for coastal locations (negative = earlier spring).
    pub coastal_modifier_days: i16,
    /// Median last spring frost (day-of-year).
    pub last_spring_doy: u16,
    /// Median first fall frost (day-of-year).
    pub first_fall_doy: u16,
    /// Variance in days (applied symmetrically for early/late bounds).
    pub variance_days: u16,
}

/// Internal lookup table entry for minimum winter temperature by latitude band.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MinTempEntry {
    pub lat_min: f64,
    pub lat_max: f64,
    /// Base minimum winter temp (F) for continental interior at sea level.
    pub min_temp_f: f64,
    /// Degrees F to add for coastal locations (warmer).
    pub coastal_modifier: f64,
}

/// Internal bounding box for Sunset zone lookup.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SunsetZoneEntry {
    pub lat_min: f64,
    pub lat_max: f64,
    pub lng_min: f64,
    pub lng_max: f64,
    pub zone: u8,
    pub description: &'static str,
}
