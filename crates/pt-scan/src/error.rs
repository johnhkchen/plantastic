use thiserror::Error;

/// Errors that can occur during scan processing.
#[derive(Debug, Error)]
pub enum ScanError {
    #[error("invalid PLY data: {0}")]
    InvalidPly(String),

    #[error("insufficient points: found {found}, need at least {needed}")]
    InsufficientPoints { found: usize, needed: usize },

    #[error("RANSAC could not find a ground plane")]
    NoGroundPlane,

    #[error("mesh generation failed: {0}")]
    MeshGeneration(String),

    #[error("export failed: {0}")]
    ExportError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
