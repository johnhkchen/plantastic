//! Error types for the quote engine.

use pt_materials::MaterialId;
use pt_project::ZoneId;
use std::fmt;

/// Errors that can occur during quote computation.
#[derive(Debug, Clone, PartialEq)]
pub enum QuoteError {
    /// A material assignment references a material not in the catalog.
    MaterialNotFound {
        material_id: MaterialId,
        zone_id: ZoneId,
    },
    /// A cu_yd material has no depth_inches and no override.
    MissingDepth {
        material_id: MaterialId,
        material_name: String,
    },
}

impl fmt::Display for QuoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaterialNotFound {
                material_id,
                zone_id,
            } => {
                write!(
                    f,
                    "material {material_id} not found in catalog (referenced by zone {zone_id})"
                )
            }
            Self::MissingDepth {
                material_id,
                material_name,
            } => {
                write!(
                    f,
                    "cu_yd material \"{material_name}\" ({material_id}) has no depth_inches and no override"
                )
            }
        }
    }
}

impl std::error::Error for QuoteError {}
