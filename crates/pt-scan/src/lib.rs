//! pt-scan: PLY parsing and point cloud filtering for Plantastic.
//!
//! Reads PLY files (binary and ASCII), filters point clouds (outlier removal,
//! voxel downsampling), fits ground planes (RANSAC), and separates ground
//! from obstacle points.

pub mod cluster;
pub mod error;
pub mod export;
pub mod filter;
pub mod mesh;
pub mod parser;
pub mod ransac;
pub mod report;
pub mod types;

pub use cluster::{Cluster, ClusterConfig, ClusterResult};
pub use error::ScanError;
pub use export::{generate_terrain, ExportConfig, TerrainMetadata, TerrainOutput};
pub use mesh::{MeshConfig, TerrainMesh};
pub use report::{
    GroundInfo, InputInfo, ObstacleInfo, OutputInfo, ProcessingInfo, ScanReport, StageTiming,
};
pub use types::{BoundingBox, Plane, Point, PointCloud, ScanConfig, ScanMetadata};

/// Process a PLY scan with per-stage timing and a structured metadata report.
///
/// Same pipeline as [`process_scan`] but additionally returns a [`ScanReport`]
/// with processing statistics, classification details, and per-stage durations.
///
/// The caller should fill `report.input.filename`, `report.input.file_size_bytes`,
/// `report.output`, and `report.timing.terrain_export_ms` after export.
///
/// # Errors
///
/// Returns `ScanError` if the PLY data is invalid, there are too few points
/// for processing, or RANSAC cannot find a ground plane.
#[allow(clippy::cast_possible_truncation)]
pub fn process_scan_timed(
    reader: impl std::io::Read,
    config: &ScanConfig,
) -> Result<(PointCloud, ScanReport), ScanError> {
    let total_start = std::time::Instant::now();

    // Stage 1: Parse PLY
    let stage_start = std::time::Instant::now();
    let raw_points = parser::parse_ply(reader)?;
    let parse_ms = stage_start.elapsed().as_millis() as u64;
    let original_count = raw_points.len();

    if original_count < 3 {
        return Err(ScanError::InsufficientPoints {
            found: original_count,
            needed: 3,
        });
    }

    // Stage 2: Voxel downsample
    let stage_start = std::time::Instant::now();
    let downsampled = filter::voxel_downsample(&raw_points, config.voxel_size);
    let downsample_ms = stage_start.elapsed().as_millis() as u64;
    let downsampled_count = downsampled.len();

    // Stage 3: Outlier removal
    let stage_start = std::time::Instant::now();
    let filtered =
        filter::remove_outliers(&downsampled, config.outlier_k, config.outlier_threshold);
    let outlier_removal_ms = stage_start.elapsed().as_millis() as u64;
    let filtered_count = filtered.len();
    let outliers_removed = downsampled_count.saturating_sub(filtered_count);

    if filtered_count < 3 {
        return Err(ScanError::InsufficientPoints {
            found: filtered_count,
            needed: 3,
        });
    }

    // Stage 4: RANSAC ground fitting
    let stage_start = std::time::Instant::now();
    let classification =
        ransac::fit_ground_plane(&filtered, config.ransac_iterations, config.ransac_threshold)?;
    let ransac_ms = stage_start.elapsed().as_millis() as u64;

    let best_iteration = classification.best_iteration;

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

    let total_processing_ms = total_start.elapsed().as_millis() as u64;

    // Compute ground area estimate (XY bounding box of ground points)
    let ground_area = if let Some(ground_bbox) = BoundingBox::from_points(&ground) {
        (ground_bbox.max[0] - ground_bbox.min[0]) * (ground_bbox.max[1] - ground_bbox.min[1])
    } else {
        0.0
    };

    // Compute obstacle height range (distance above ground plane)
    let plane = &classification.plane;
    let obstacle_heights: Vec<f32> = obstacles
        .iter()
        .map(|p| {
            (plane.normal[0] * p.position[0]
                + plane.normal[1] * p.position[1]
                + plane.normal[2] * p.position[2]
                + plane.d)
                .abs()
        })
        .collect();

    let height_range = if obstacle_heights.is_empty() {
        None
    } else {
        let min_h = obstacle_heights
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_h = obstacle_heights
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        Some([min_h, max_h])
    };

    let obstacle_bbox = BoundingBox::from_points(&obstacles);

    let report = ScanReport {
        input: InputInfo {
            filename: None,
            file_size_bytes: None,
            format: "ply".to_string(),
            original_vertex_count: original_count,
        },
        processing: ProcessingInfo {
            downsample_ratio: filtered_count as f32 / original_count as f32,
            downsampled_count,
            outliers_removed,
            ransac_iterations_config: config.ransac_iterations,
            ransac_best_iteration: best_iteration,
        },
        ground: GroundInfo {
            plane: classification.plane,
            point_count: ground.len(),
            area_estimate_sqm: ground_area,
        },
        obstacles: ObstacleInfo {
            count: obstacles.len(),
            height_range,
            bbox: obstacle_bbox,
        },
        timing: StageTiming {
            parse_ms,
            downsample_ms,
            outlier_removal_ms,
            ransac_ms,
            total_processing_ms,
            terrain_export_ms: None,
        },
        output: None,
    };

    let metadata = ScanMetadata {
        bbox,
        original_count,
        filtered_count,
        ground_count: ground.len(),
        obstacle_count: obstacles.len(),
        ground_plane: report.ground.plane.clone(),
    };

    let cloud = PointCloud {
        ground,
        obstacles,
        metadata,
    };

    Ok((cloud, report))
}

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
