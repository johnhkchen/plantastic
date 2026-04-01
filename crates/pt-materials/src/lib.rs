//! Material catalog domain: materials, categories, units, pricing, extrusion behavior.
//!
//! Defines the types that represent what a landscaping company sells and installs.
//! Each material has pricing for quote computation, physical properties for quantity
//! calculation, and extrusion behavior for 3D rendering.

pub mod builder;
pub mod types;

pub use types::{ExtrusionBehavior, Material, MaterialCategory, MaterialId, Unit};
