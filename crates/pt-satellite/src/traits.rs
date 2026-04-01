//! Data source traits for satellite pre-population.
//!
//! These traits define the contracts for external data retrieval.
//! The [`crate::EmbeddedSource`] implements all three for testing.
//! Real implementations (geocoding API, parcel API, canopy dataset)
//! will be added in follow-on tickets.

use pt_solar::{Coordinates, LatLngBounds};

use crate::error::SatelliteError;
use crate::types::{DetectedTree, LotBoundary};

/// Converts a street address to geographic coordinates.
pub trait Geocoder {
    /// Geocode an address to WGS84 coordinates.
    ///
    /// # Errors
    /// Returns [`SatelliteError::AddressNotFound`] if the address cannot be resolved.
    fn geocode(&self, address: &str) -> Result<Coordinates, SatelliteError>;
}

/// Retrieves lot boundary polygons from parcel data.
pub trait ParcelSource {
    /// Look up the lot boundary for a location.
    ///
    /// # Errors
    /// Returns [`SatelliteError::NoParcelData`] if no parcel covers the coordinates.
    fn lot_boundary(&self, coords: &Coordinates) -> Result<LotBoundary, SatelliteError>;
}

/// Detects trees from canopy height data within a bounding box.
pub trait CanopySource {
    /// Detect trees within the given bounds.
    ///
    /// # Errors
    /// Returns [`SatelliteError::CanopyUnavailable`] if canopy data is not available.
    fn detect_trees(&self, bounds: &LatLngBounds) -> Result<Vec<DetectedTree>, SatelliteError>;
}
