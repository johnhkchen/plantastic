//! Solar radiance engine for Plantastic.
//!
//! Computes sun position, daily/seasonal sun hours, light category
//! classification, and spatial radiance grids. Pure computation — no I/O.
//!
//! Ported from the solar-sim TypeScript prototype. Uses simplified NOAA
//! solar position formulas (same approach as SunCalc.js).

pub mod classification;
pub mod grid;
pub mod position;
pub mod seasonal;
pub mod sun_hours;
pub mod types;

// Re-export key types and functions at crate root.
pub use classification::{classify, LightCategory};
pub use grid::radiance_grid;
pub use position::sun_position;
pub use seasonal::annual_sun_hours;
pub use sun_hours::daily_sun_hours;
pub use types::*;
