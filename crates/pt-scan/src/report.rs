//! Structured scan processing report for downstream consumption (BAML, API).

use serde::{Deserialize, Serialize};

use crate::export::TerrainOutput;
use crate::types::{BoundingBox, Plane};

/// Complete metadata report for a scan processing run.
///
/// Captures input, processing, classification, timing, and output details.
/// Serializable to JSON for BAML function context and API responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanReport {
    pub input: InputInfo,
    pub processing: ProcessingInfo,
    pub ground: GroundInfo,
    pub obstacles: ObstacleInfo,
    pub timing: StageTiming,
    pub output: Option<OutputInfo>,
}

/// Input file metadata. Populated by the caller (CLI/API), not the pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputInfo {
    pub filename: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub format: String,
    pub original_vertex_count: usize,
}

/// Processing pipeline statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessingInfo {
    /// Ratio of points retained after downsampling + outlier removal.
    pub downsample_ratio: f32,
    /// Point count after voxel downsampling (before outlier removal).
    pub downsampled_count: usize,
    /// Number of points removed as outliers.
    pub outliers_removed: usize,
    /// Configured RANSAC iteration count.
    pub ransac_iterations_config: usize,
    /// The iteration (0-indexed) that found the best ground plane.
    pub ransac_best_iteration: usize,
}

/// Ground plane classification details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundInfo {
    pub plane: Plane,
    pub point_count: usize,
    /// Estimated ground area in square meters (bounding box of ground points on XY).
    pub area_estimate_sqm: f32,
}

/// Obstacle classification details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObstacleInfo {
    pub count: usize,
    /// [min, max] height above the ground plane in meters. None if no obstacles.
    pub height_range: Option<[f32; 2]>,
    /// Axis-aligned bounding box of obstacle points. None if no obstacles.
    pub bbox: Option<BoundingBox>,
}

/// Per-stage processing durations in milliseconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageTiming {
    pub parse_ms: u64,
    pub downsample_ms: u64,
    pub outlier_removal_ms: u64,
    pub ransac_ms: u64,
    pub total_processing_ms: u64,
    /// Terrain export duration. None if export was not run.
    pub terrain_export_ms: Option<u64>,
}

/// Output artifact sizes and counts. Populated after terrain export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputInfo {
    pub glb_size_bytes: usize,
    pub png_size_bytes: usize,
    pub triangle_count: usize,
    pub vertex_count: usize,
}

impl OutputInfo {
    /// Build from a completed terrain export.
    pub fn from_terrain(output: &TerrainOutput) -> Self {
        Self {
            glb_size_bytes: output.mesh_glb.len(),
            png_size_bytes: output.plan_view_png.len(),
            triangle_count: output.metadata.decimated_triangle_count,
            vertex_count: output.metadata.vertex_count,
        }
    }
}
