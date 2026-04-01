//! Shared types and constants for the solar engine.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Sampling interval in minutes for sun-hour integration.
/// 5 minutes provides sub-1% accuracy at classification boundaries
/// while keeping computation under 2ms per year at a single point.
pub const SAMPLING_INTERVAL_MINUTES: u32 = 5;

/// Number of samples per 24-hour day at 5-minute intervals.
pub const SAMPLES_PER_DAY: u32 = 24 * 60 / SAMPLING_INTERVAL_MINUTES;

/// Meters per degree of latitude (approximately constant).
pub const METERS_PER_DEGREE_LAT: f64 = 111_320.0;

/// Geographic coordinates (WGS84).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinates {
    /// Latitude in degrees. Positive north, negative south. Range: -90 to 90.
    pub latitude: f64,
    /// Longitude in degrees. Positive east, negative west. Range: -180 to 180.
    pub longitude: f64,
}

/// Sun position at a specific moment.
#[derive(Debug, Clone, Copy)]
pub struct SolarPosition {
    /// Degrees above the horizon. Negative when below.
    pub altitude_degrees: f64,
    /// Compass bearing in degrees. 0 = north, 90 = east, 180 = south, 270 = west.
    pub azimuth_degrees: f64,
}

/// Whether a day has normal sunrise/sunset or a polar extreme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolarCondition {
    /// Normal day with sunrise and sunset.
    Normal,
    /// Sun never sets (Arctic/Antarctic summer).
    MidnightSun,
    /// Sun never rises (Arctic/Antarctic winter).
    PolarNight,
}

/// Sun data for a single day at a location.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DailySunData {
    pub date: NaiveDate,
    /// Total hours the sun is above the horizon.
    pub sun_hours: f64,
    pub polar_condition: PolarCondition,
}

/// Summary statistics across a date range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalSummary {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub average_sun_hours: f64,
    pub min_sun_hours: f64,
    pub max_sun_hours: f64,
    pub total_days: u32,
    pub days_of_midnight_sun: u32,
    pub days_of_polar_night: u32,
    pub daily_data: Vec<DailySunData>,
}

/// Geographic bounding box for grid calculations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LatLngBounds {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

/// Configuration for radiance grid computation.
#[derive(Debug, Clone, Copy)]
pub struct GridConfig {
    /// Cell size in meters.
    pub resolution_meters: f64,
    /// Number of representative days to sample (default: 12).
    pub sample_days: u32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            resolution_meters: 2.0,
            sample_days: 12,
        }
    }
}

/// Result of a grid-based sun exposure calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureGrid {
    pub bounds: LatLngBounds,
    pub resolution_meters: f64,
    /// Number of columns (west to east).
    pub width: u32,
    /// Number of rows (south to north).
    pub height: u32,
    /// Average sun hours per cell, row-major order (south to north, west to east).
    pub values: Vec<f32>,
    pub sample_days_used: u32,
}
