//! Error types for satellite pre-population.

/// Errors that can occur during satellite pre-population.
#[derive(Debug, thiserror::Error)]
pub enum SatelliteError {
    /// The given address could not be geocoded to coordinates.
    #[error("address not found: {0}")]
    AddressNotFound(String),

    /// No parcel data is available for the given coordinates.
    #[error("no parcel data for coordinates ({lat}, {lng})")]
    NoParcelData { lat: f64, lng: f64 },

    /// Canopy height data is unavailable for the requested area.
    #[error("canopy data unavailable for requested bounds")]
    CanopyUnavailable,
}
