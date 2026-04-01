//! Core types for satellite pre-population.

use geo::Polygon;
use pt_solar::{Coordinates, ExposureGrid};
use serde::{Deserialize, Serialize};

/// Complete pre-populated baseline for a project site.
///
/// Produced by [`crate::BaselineBuilder::build`] from an address.
/// Contains everything known about the site before a physical visit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBaseline {
    /// Geocoded site coordinates (WGS84).
    pub coordinates: Coordinates,
    /// Lot boundary from parcel data.
    pub lot_boundary: LotBoundary,
    /// Trees detected from canopy height data.
    pub trees: Vec<DetectedTree>,
    /// Sun exposure grid computed over the lot bounds.
    pub sun_grid: ExposureGrid,
}

/// Lot boundary polygon with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotBoundary {
    /// Lot outline in WGS84 coordinates (lat/lng).
    #[serde(with = "crate::serde_helpers::geojson_polygon")]
    pub polygon: Polygon<f64>,
    /// Lot area in square feet.
    pub area_sqft: f64,
    /// Where this data came from.
    pub source: DataSourceLabel,
}

/// A tree detected from canopy height data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DetectedTree {
    /// Tree location in WGS84 coordinates.
    pub location: Coordinates,
    /// Estimated height in feet.
    pub height_ft: f64,
    /// Estimated canopy spread (diameter) in feet.
    pub spread_ft: f64,
    /// Detection confidence (0.0–1.0).
    pub confidence: f64,
}

/// Label indicating the origin of a data element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSourceLabel {
    /// Hardcoded test data embedded in the crate.
    Embedded,
    /// Municipal parcel or government data source.
    Municipal,
    /// Satellite or remote sensing data source.
    Satellite,
}
