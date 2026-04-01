//! Project domain model: zones, tiers, material assignments, GeoJSON serialization.
//!
//! The Project is the single source of truth for a landscaping design. Every
//! component — quote engine, 3D renderer, PDF generator, frontend editor —
//! reads from or writes to this model.

pub mod error;
pub mod geojson_conv;
pub mod project;
pub mod serde_helpers;
pub mod types;

pub use error::ProjectError;
pub use project::Project;
pub use types::{
    AssignmentOverrides, MaterialAssignment, ProjectStatus, Tier, TierLevel, Zone, ZoneId, ZoneType,
};
