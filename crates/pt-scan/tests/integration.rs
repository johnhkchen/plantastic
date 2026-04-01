//! Integration tests for the pt-scan processing pipeline.

use pt_scan::{ScanConfig, ScanError};
use pt_test_utils::timed;
use std::io::Cursor;

/// Generate a synthetic binary PLY file with known geometry:
/// - `ground_n` points on z ≈ 0 (noise ±0.005m) spread over 10m × 10m
/// - `obstacle_n` points at z ∈ [0.3, 1.0] (box-shaped obstacles)
/// - `outlier_n` points at z > 10.0 (clear outliers)
fn make_synthetic_ply(ground_n: usize, obstacle_n: usize, outlier_n: usize) -> Vec<u8> {
    let total = ground_n + obstacle_n + outlier_n;
    let mut buf = Vec::new();

    let header = format!(
        "ply\n\
         format binary_little_endian 1.0\n\
         element vertex {total}\n\
         property float x\n\
         property float y\n\
         property float z\n\
         property uchar red\n\
         property uchar green\n\
         property uchar blue\n\
         end_header\n"
    );
    buf.extend_from_slice(header.as_bytes());

    // Ground points: z ≈ 0, spread over [0, 10] × [0, 10]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let side = (ground_n as f32).sqrt().ceil() as usize;
    for i in 0..ground_n {
        let x = (i % side) as f32 * (10.0 / side as f32);
        let y = (i / side) as f32 * (10.0 / side as f32);
        // Small deterministic noise: alternating ±0.003
        let z = if i % 2 == 0 { 0.003 } else { -0.003_f32 };
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.extend_from_slice(&z.to_le_bytes());
        buf.extend_from_slice(&[0, 128, 0]); // green for ground
    }

    // Obstacle points: box at x∈[2,3], y∈[2,3], z∈[0.3, 1.0]
    for i in 0..obstacle_n {
        let x = 2.0 + (i % 10) as f32 * 0.1;
        let y = 2.0 + (i / 10) as f32 * 0.1;
        let z = 0.3 + (i as f32 / obstacle_n as f32) * 0.7;
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.extend_from_slice(&z.to_le_bytes());
        buf.extend_from_slice(&[128, 0, 0]); // red for obstacles
    }

    // Outlier points: far away
    for i in 0..outlier_n {
        let x = 50.0 + i as f32;
        let y = 50.0 + i as f32;
        let z = 20.0 + i as f32;
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.extend_from_slice(&z.to_le_bytes());
        buf.extend_from_slice(&[255, 255, 255]); // white for outliers
    }

    buf
}

#[test]
fn test_full_pipeline() {
    timed(|| {
        // 1000 ground + 200 obstacle + 50 outlier = 1250 total
        let ply_data = make_synthetic_ply(1000, 200, 50);

        // Use larger voxels and lower k for test speed
        let config = ScanConfig {
            voxel_size: 0.5,
            outlier_k: 10,
            outlier_threshold: 2.0,
            ransac_iterations: 500,
            ransac_threshold: 0.05,
        };

        let cloud = pt_scan::process_scan(Cursor::new(ply_data), &config).unwrap();

        // Metadata: original count should be 1250
        assert_eq!(cloud.metadata.original_count, 1250);

        // After processing, outlier points (z > 10) should be removed
        // Verify no ground or obstacle point has z > 10
        for p in &cloud.ground {
            assert!(
                p.position[2] < 10.0,
                "ground point has z={}, likely an outlier leaked through",
                p.position[2]
            );
        }
        for p in &cloud.obstacles {
            assert!(
                p.position[2] < 10.0,
                "obstacle point has z={}, likely an outlier leaked through",
                p.position[2]
            );
        }

        // Ground points should have z near 0 (within RANSAC threshold)
        for p in &cloud.ground {
            assert!(
                p.position[2].abs() < 0.1,
                "ground point has z={}, expected near 0",
                p.position[2]
            );
        }

        // We should have some ground and some obstacle points
        assert!(!cloud.ground.is_empty(), "expected ground points but got 0");
        assert!(
            !cloud.obstacles.is_empty(),
            "expected obstacle points but got 0"
        );

        // Metadata counts should be consistent
        assert_eq!(
            cloud.metadata.ground_count,
            cloud.ground.len(),
            "metadata ground_count doesn't match"
        );
        assert_eq!(
            cloud.metadata.obstacle_count,
            cloud.obstacles.len(),
            "metadata obstacle_count doesn't match"
        );
        assert_eq!(
            cloud.metadata.ground_count + cloud.metadata.obstacle_count,
            cloud.metadata.filtered_count,
            "ground + obstacle != filtered"
        );

        // BBox should be within the input range (no outlier at z=20+ should be in bbox)
        assert!(
            cloud.metadata.bbox.max[2] < 10.0,
            "bbox max z={}, outliers should be filtered",
            cloud.metadata.bbox.max[2]
        );
    });
}

#[test]
fn test_default_config_values() {
    timed(|| {
        let config = ScanConfig::default();
        // Verify documented defaults
        assert!((config.voxel_size - 0.02).abs() < f32::EPSILON);
        assert_eq!(config.outlier_k, 30);
        assert!((config.outlier_threshold - 2.0).abs() < f32::EPSILON);
        assert_eq!(config.ransac_iterations, 1000);
        assert!((config.ransac_threshold - 0.02).abs() < f32::EPSILON);
    });
}

#[test]
fn test_terrain_generation_pipeline() {
    timed(|| {
        // Full pipeline: PLY → PointCloud → TerrainOutput
        let ply_data = make_synthetic_ply(500, 100, 30);
        let config = ScanConfig {
            voxel_size: 0.5,
            outlier_k: 10,
            outlier_threshold: 2.0,
            ransac_iterations: 500,
            ransac_threshold: 0.05,
        };

        let cloud = pt_scan::process_scan(Cursor::new(ply_data), &config).unwrap();
        let export_config = pt_scan::ExportConfig::default();
        let output = pt_scan::generate_terrain(&cloud, &export_config).unwrap();

        // All three outputs should be non-empty
        assert!(!output.mesh_glb.is_empty(), "GLB output empty");
        assert!(!output.plan_view_png.is_empty(), "PNG output empty");

        // Metadata should be consistent with actual data
        assert!(output.metadata.decimated_triangle_count > 0);
        assert!(output.metadata.vertex_count > 0);
        assert_eq!(output.metadata.original_point_count, 630);
        // 500 + 100 + 30 = 630 points total, independently computed

        // Elevation range: ground near z=0, no outliers (z>10) survived
        assert!(
            output.metadata.elevation_range[0] >= -0.1,
            "min elevation {} too low",
            output.metadata.elevation_range[0]
        );
        assert!(
            output.metadata.elevation_range[1] <= 1.0,
            "max elevation {} too high (outlier leak?)",
            output.metadata.elevation_range[1]
        );
    });
}

#[test]
fn test_glb_structure() {
    timed(|| {
        let ply_data = make_synthetic_ply(200, 50, 10);
        let config = ScanConfig {
            voxel_size: 0.5,
            outlier_k: 10,
            outlier_threshold: 2.0,
            ransac_iterations: 500,
            ransac_threshold: 0.05,
        };

        let cloud = pt_scan::process_scan(Cursor::new(ply_data), &config).unwrap();
        let output = pt_scan::generate_terrain(&cloud, &pt_scan::ExportConfig::default()).unwrap();
        let glb = &output.mesh_glb;

        // GLB header: 12 bytes
        assert!(glb.len() >= 20, "GLB too short");

        // Magic: 0x46546C67 = "glTF"
        let magic = u32::from_le_bytes([glb[0], glb[1], glb[2], glb[3]]);
        assert_eq!(magic, 0x4654_6C67, "not a valid GLB file");

        // Version 2
        let version = u32::from_le_bytes([glb[4], glb[5], glb[6], glb[7]]);
        assert_eq!(version, 2);

        // Total length matches buffer
        let total = u32::from_le_bytes([glb[8], glb[9], glb[10], glb[11]]) as usize;
        assert_eq!(total, glb.len());

        // First chunk is JSON
        let chunk_type = u32::from_le_bytes([glb[16], glb[17], glb[18], glb[19]]);
        assert_eq!(chunk_type, 0x4E4F_534A, "first chunk should be JSON");

        // JSON is parseable and has required glTF fields
        let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
        let json_bytes = &glb[20..20 + json_len];
        let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();

        assert_eq!(parsed["asset"]["version"], "2.0");
        assert!(parsed["meshes"].as_array().unwrap().len() == 1);

        // Accessors: 4 (POSITION, NORMAL, COLOR_0, indices)
        assert_eq!(parsed["accessors"].as_array().unwrap().len(), 4);
    });
}

#[test]
fn test_plan_view_png_valid() {
    timed(|| {
        let ply_data = make_synthetic_ply(200, 50, 10);
        let config = ScanConfig {
            voxel_size: 0.5,
            outlier_k: 10,
            outlier_threshold: 2.0,
            ransac_iterations: 500,
            ransac_threshold: 0.05,
        };

        let cloud = pt_scan::process_scan(Cursor::new(ply_data), &config).unwrap();
        let output = pt_scan::generate_terrain(&cloud, &pt_scan::ExportConfig::default()).unwrap();

        let png = &output.plan_view_png;

        // PNG signature: 8 bytes
        assert!(png.len() >= 8, "PNG too short");
        // 0x89 P N G \r \n 0x1A \n
        assert_eq!(png[0], 0x89);
        assert_eq!(&png[1..4], b"PNG");

        // Should be decodable as an image
        let img = image::load_from_memory(png).expect("should be a valid PNG");
        let (w, h) = (img.width(), img.height());
        assert!(w > 0 && h > 0, "image should have positive dimensions");
    });
}

#[test]
fn test_insufficient_points_error() {
    timed(|| {
        // PLY with only 2 points → should fail
        let mut buf = Vec::new();
        let header = "ply\n\
                      format binary_little_endian 1.0\n\
                      element vertex 2\n\
                      property float x\n\
                      property float y\n\
                      property float z\n\
                      end_header\n";
        buf.extend_from_slice(header.as_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&1.0_f32.to_le_bytes());
        buf.extend_from_slice(&1.0_f32.to_le_bytes());
        buf.extend_from_slice(&1.0_f32.to_le_bytes());

        let result = pt_scan::process_scan(Cursor::new(buf), &ScanConfig::default());
        assert!(
            matches!(result, Err(ScanError::InsufficientPoints { .. })),
            "expected InsufficientPoints error, got {result:?}"
        );
    });
}
