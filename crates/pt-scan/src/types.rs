use serde::{Deserialize, Serialize};

/// A single 3D point with optional color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    /// Position in meters: [x, y, z].
    pub position: [f32; 3],
    /// Optional RGB color.
    pub color: Option<[u8; 3]>,
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BoundingBox {
    /// Compute bounding box from a set of points. Returns None if empty.
    pub fn from_points(points: &[Point]) -> Option<Self> {
        let first = points.first()?;
        let mut min = first.position;
        let mut max = first.position;
        for p in &points[1..] {
            for i in 0..3 {
                if p.position[i] < min[i] {
                    min[i] = p.position[i];
                }
                if p.position[i] > max[i] {
                    max[i] = p.position[i];
                }
            }
        }
        Some(Self { min, max })
    }
}

/// Plane equation: normal · point + d = 0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plane {
    /// Unit normal vector [a, b, c].
    pub normal: [f32; 3],
    /// Distance component d.
    pub d: f32,
}

/// Result of RANSAC ground plane fitting.
#[derive(Debug, Clone)]
pub struct GroundClassification {
    pub ground_indices: Vec<usize>,
    pub obstacle_indices: Vec<usize>,
    pub plane: Plane,
}

/// Metadata about the processed scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMetadata {
    pub bbox: BoundingBox,
    pub original_count: usize,
    pub filtered_count: usize,
    pub ground_count: usize,
    pub obstacle_count: usize,
    pub ground_plane: Plane,
}

/// Processed point cloud with ground/obstacle separation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloud {
    pub ground: Vec<Point>,
    pub obstacles: Vec<Point>,
    pub metadata: ScanMetadata,
}

/// Configuration for the scan processing pipeline.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Voxel grid cell size in meters (default: 0.02 = 2cm).
    pub voxel_size: f32,
    /// Number of neighbors for outlier detection (default: 30).
    pub outlier_k: usize,
    /// Standard deviation multiplier for outlier threshold (default: 2.0).
    pub outlier_threshold: f32,
    /// Number of RANSAC iterations (default: 1000).
    pub ransac_iterations: usize,
    /// RANSAC inlier distance threshold in meters (default: 0.02 = 2cm).
    pub ransac_threshold: f32,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            voxel_size: 0.02,
            outlier_k: 30,
            outlier_threshold: 2.0,
            ransac_iterations: 1000,
            ransac_threshold: 0.02,
        }
    }
}
