//! Scene generation errors.

use pt_materials::MaterialId;
use pt_project::ZoneId;

/// Errors that can occur during scene generation.
#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    #[error("material {material_id} not found for zone {zone_id}")]
    MissingMaterial {
        zone_id: ZoneId,
        material_id: MaterialId,
    },

    #[error("triangulation failed: {0}")]
    Triangulation(String),

    #[error("glTF export failed: {0}")]
    Export(String),
}
