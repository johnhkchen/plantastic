//! Satellite pre-population engine for Plantastic.
//!
//! Given an address, produces a `ProjectBaseline` containing lot boundary,
//! detected trees, and a sun exposure grid. Uses trait-based data sources
//! so embedded test data can be swapped for real APIs.

pub mod builder;
pub mod embedded;
pub mod error;
pub(crate) mod serde_helpers;
pub mod traits;
pub mod types;

pub use builder::BaselineBuilder;
pub use embedded::EmbeddedSource;
pub use error::SatelliteError;
pub use traits::{CanopySource, Geocoder, ParcelSource};
pub use types::*;

// Re-export Coordinates so callers don't need a direct pt-solar dependency.
pub use pt_solar::Coordinates;
