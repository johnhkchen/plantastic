//! pt-scan: PLY parsing and point cloud filtering for Plantastic.
//!
//! Reads PLY files (binary and ASCII), filters point clouds (outlier removal,
//! voxel downsampling), fits ground planes (RANSAC), and separates ground
//! from obstacle points.

pub mod error;
pub mod export;
pub mod filter;
pub mod mesh;
pub mod parser;
pub mod ransac;
pub mod types;

pub use error::ScanError;
pub use export::{generate_terrain, ExportConfig, TerrainMetadata, TerrainOutput};
pub use mesh::{MeshConfig, TerrainMesh};
pub use types::{BoundingBox, Plane, Point, PointCloud, ScanConfig, ScanMetadata};

/// Process a PLY scan through the full pipeline.
///
/// Pipeline: parse → voxel downsample → outlier removal → RANSAC ground fitting.
///
/// # Errors
///
/// Returns `ScanError` if the PLY data is invalid, there are too few points
/// for processing, or RANSAC cannot find a ground plane.
pub fn process_scan(
    reader: impl std::io::Read,
    config: &ScanConfig,
) -> Result<PointCloud, ScanError> {
    let raw_points = parser::parse_ply(reader)?;
    let original_count = raw_points.len();

    if original_count < 3 {
        return Err(ScanError::InsufficientPoints {
            found: original_count,
            needed: 3,
        });
    }

    // Downsample first to reduce point count for faster k-NN
    let downsampled = filter::voxel_downsample(&raw_points, config.voxel_size);

    // Remove statistical outliers
    let filtered =
        filter::remove_outliers(&downsampled, config.outlier_k, config.outlier_threshold);

    let filtered_count = filtered.len();

    if filtered_count < 3 {
        return Err(ScanError::InsufficientPoints {
            found: filtered_count,
            needed: 3,
        });
    }

    // Fit ground plane and classify points
    let classification =
        ransac::fit_ground_plane(&filtered, config.ransac_iterations, config.ransac_threshold)?;

    let ground: Vec<Point> = classification
        .ground_indices
        .iter()
        .map(|&i| filtered[i].clone())
        .collect();

    let obstacles: Vec<Point> = classification
        .obstacle_indices
        .iter()
        .map(|&i| filtered[i].clone())
        .collect();

    let mut all_points = Vec::with_capacity(ground.len() + obstacles.len());
    all_points.extend_from_slice(&ground);
    all_points.extend_from_slice(&obstacles);

    let bbox = BoundingBox::from_points(&all_points)
        .expect("filtered_count >= 3, so at least 3 points exist");

    let metadata = ScanMetadata {
        bbox,
        original_count,
        filtered_count,
        ground_count: ground.len(),
        obstacle_count: obstacles.len(),
        ground_plane: classification.plane,
    };

    Ok(PointCloud {
        ground,
        obstacles,
        metadata,
    })
}
