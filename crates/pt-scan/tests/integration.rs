//! Integration tests for the pt-scan processing pipeline.

use pt_scan::{ScanConfig, ScanError, ScanReport};
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

#[test]
fn test_scan_report_round_trip() {
    timed(|| {
        let ply_data = make_synthetic_ply(500, 100, 30);
        let config = ScanConfig {
            voxel_size: 0.5,
            outlier_k: 10,
            outlier_threshold: 2.0,
            ransac_iterations: 500,
            ransac_threshold: 0.05,
        };

        let (_cloud, report) = pt_scan::process_scan_timed(Cursor::new(ply_data), &config).unwrap();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&report).expect("report should serialize");

        // Deserialize back
        let deserialized: ScanReport =
            serde_json::from_str(&json).expect("report should deserialize");

        // Round-trip equality
        assert_eq!(report, deserialized, "report round-trip failed");
    });
}

#[test]
fn test_scan_report_fields_populated() {
    timed(|| {
        // 500 ground + 100 obstacle + 30 outlier = 630 total
        let ply_data = make_synthetic_ply(500, 100, 30);
        let config = ScanConfig {
            voxel_size: 0.5,
            outlier_k: 10,
            outlier_threshold: 2.0,
            ransac_iterations: 500,
            ransac_threshold: 0.05,
        };

        let (cloud, report) = pt_scan::process_scan_timed(Cursor::new(ply_data), &config).unwrap();

        // Input: 630 vertices, format = "ply", caller fields unset
        assert_eq!(report.input.original_vertex_count, 630);
        assert_eq!(report.input.format, "ply");
        assert!(report.input.filename.is_none());
        assert!(report.input.file_size_bytes.is_none());

        // Processing: downsample ratio should be < 1.0 (we used 0.5m voxels on 10m scan)
        assert!(
            report.processing.downsample_ratio > 0.0 && report.processing.downsample_ratio <= 1.0,
            "downsample_ratio={} should be in (0, 1]",
            report.processing.downsample_ratio,
        );
        assert!(report.processing.downsampled_count <= 630);
        assert_eq!(report.processing.ransac_iterations_config, 500);
        assert!(report.processing.ransac_best_iteration < 500);

        // Ground: should have some points, area > 0
        assert!(report.ground.point_count > 0);
        assert!(
            report.ground.area_estimate_sqm > 0.0,
            "ground area should be positive"
        );

        // Obstacles: should have some, with height range
        assert!(report.obstacles.count > 0);
        let [min_h, max_h] = report
            .obstacles
            .height_range
            .expect("should have height range");
        // Obstacles are at z ∈ [0.3, 1.0], ground at z ≈ 0, so heights should be ~0.3–1.0
        assert!(min_h >= 0.0, "min obstacle height should be >= 0");
        assert!(max_h > min_h, "max should exceed min");
        assert!(report.obstacles.bbox.is_some());

        // Timing: total should be positive
        assert!(
            report.timing.total_processing_ms < 30_000,
            "processing should finish in reasonable time"
        );
        assert!(report.timing.terrain_export_ms.is_none());

        // Output: not set (no terrain export run)
        assert!(report.output.is_none());

        // Report counts should match cloud metadata
        assert_eq!(report.ground.point_count, cloud.metadata.ground_count);
        assert_eq!(report.obstacles.count, cloud.metadata.obstacle_count);
        assert_eq!(
            report.input.original_vertex_count,
            cloud.metadata.original_count
        );
    });
}

#[test]
fn test_terrain_glb_is_y_up() {
    timed(|| {
        // Synthetic cloud: ground at z ≈ 0 spread over XY, obstacles at z > 0.3
        // After Z-up → Y-up transform: [x, y, z] → [x, z, -y]
        // Ground z ≈ 0 → GLB Y ≈ 0 (flat ground plane)
        // XY spread → GLB XZ spread (horizontal plane)
        let ply_data = make_synthetic_ply(500, 100, 30);
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

        // Parse JSON chunk to read position accessor min/max
        let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
        let json_bytes = &glb[20..20 + json_len];
        let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();

        // Position accessor is index 0
        let pos_accessor = &parsed["accessors"][0];
        let pos_min: Vec<f64> = pos_accessor["min"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let pos_max: Vec<f64> = pos_accessor["max"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();

        // In Y-up: Y is vertical, X and Z are horizontal.
        // Ground was at scan-z ≈ 0, which maps to GLB Y ≈ 0.
        // The terrain mesh is ground-only, so Y range should be near 0.
        assert!(
            pos_max[1].abs() < 0.5,
            "GLB Y-max={:.3}, expected near 0 for ground plane (Y-up)",
            pos_max[1],
        );
        assert!(
            pos_min[1].abs() < 0.5,
            "GLB Y-min={:.3}, expected near 0 for ground plane (Y-up)",
            pos_min[1],
        );

        // X should span horizontally (ground was spread over X in [0, 10])
        assert!(
            pos_max[0] - pos_min[0] > 1.0,
            "GLB X range too small: {:.3} to {:.3}",
            pos_min[0],
            pos_max[0],
        );

        // Z should span horizontally (old Y mapped to -Z)
        assert!(
            pos_max[2] - pos_min[2] > 1.0,
            "GLB Z range too small: {:.3} to {:.3}",
            pos_min[2],
            pos_max[2],
        );

        // Node name should be "terrain"
        assert_eq!(
            parsed["nodes"][0]["name"].as_str().unwrap(),
            "terrain",
            "terrain node should be named 'terrain'"
        );
    });
}

#[test]
fn test_terrain_glb_has_vertex_colors() {
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

        // Parse JSON chunk
        let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
        let json_bytes = &glb[20..20 + json_len];
        let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();

        // The mesh primitive should have a COLOR_0 attribute
        let attrs = &parsed["meshes"][0]["primitives"][0]["attributes"];
        assert!(
            attrs["COLOR_0"].is_number(),
            "mesh should have COLOR_0 vertex attribute"
        );

        // COLOR_0 accessor should be VEC4 UNSIGNED_BYTE normalized
        #[allow(clippy::cast_possible_truncation)]
        let color_accessor_idx = attrs["COLOR_0"].as_u64().unwrap() as usize;
        let color_accessor = &parsed["accessors"][color_accessor_idx];
        assert_eq!(
            color_accessor["componentType"].as_u64().unwrap(),
            5121, // UNSIGNED_BYTE
            "COLOR_0 should be UNSIGNED_BYTE"
        );
        assert_eq!(
            color_accessor["type"].as_str().unwrap(),
            "VEC4",
            "COLOR_0 should be VEC4"
        );
        assert!(
            color_accessor["normalized"].as_bool().unwrap(),
            "COLOR_0 should be normalized"
        );
    });
}

#[test]
fn test_powell_market_two_clusters() {
    use pt_scan::cluster::{cluster_obstacles, ClusterConfig};
    use std::fs::File;
    use std::io::BufReader;

    let scan_path = "../../assets/scans/samples/Scan at 09.23.ply";
    let file = match File::open(scan_path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Skipping Powell & Market test: scan file not found at {scan_path}");
            return;
        }
    };

    let config = ScanConfig {
        voxel_size: 0.05,
        outlier_k: 20,
        outlier_threshold: 2.0,
        ransac_iterations: 1000,
        ransac_threshold: 0.05,
    };

    let cloud = pt_scan::process_scan(BufReader::new(file), &config)
        .expect("Powell & Market scan should process successfully");

    eprintln!(
        "Powell & Market: {} ground, {} obstacle points",
        cloud.ground.len(),
        cloud.obstacles.len()
    );

    let cluster_config = ClusterConfig {
        epsilon: 0.3,
        min_points: 50,
    };
    let result = cluster_obstacles(&cloud.obstacles, &cluster_config);

    eprintln!(
        "Clusters: {}, noise points: {}",
        result.clusters.len(),
        result.noise_indices.len()
    );
    for c in &result.clusters {
        eprintln!(
            "  Cluster {}: {} points, centroid [{:.2}, {:.2}, {:.2}]",
            c.id,
            c.point_indices.len(),
            c.centroid[0],
            c.centroid[1],
            c.centroid[2],
        );
    }

    assert_eq!(
        result.clusters.len(),
        2,
        "Powell & Market should produce exactly 2 clusters (two tree trunks), got {}",
        result.clusters.len()
    );
}
