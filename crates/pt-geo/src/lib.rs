//! Geometry and spatial math for Plantastic.
//!
//! Thin wrapper around the [`geo`] crate providing domain-specific functions
//! that operate in landscaping units (sq ft, linear ft, cu yd).
//!
//! All functions are pure — no I/O, no side effects.

pub mod area;
pub mod boolean;
pub mod perimeter;
pub mod simplify;
pub mod volume;

// Re-export core geo types so callers don't need a direct geo dependency.
pub use geo::{coord, polygon, Coord, LineString, MultiPolygon, Polygon};
