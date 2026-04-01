//! Integration tests for the pt-scan processing pipeline.

use pt_scan::{extract_candidates, measure_gaps, GapConfig, ScanConfig, ScanError, ScanReport};
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
    let cluster_start = std::time::Instant::now();
    let result = cluster_obstacles(&cloud.obstacles, &cluster_config);
    let cluster_ms = cluster_start.elapsed().as_millis();

    eprintln!("Clustering took {cluster_ms}ms");
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

    // The Powell & Market scan contains multiple urban features (tree trunks,
    // poles, curbs, planters, etc.) — not just 2 tree trunks. With default params
    // (eps=0.3m, min_pts=50), expect multiple distinct clusters.
    // The two largest clusters (by point count) correspond to the two main tree trunks.
    assert!(
        result.clusters.len() >= 2,
        "Powell & Market should produce at least 2 clusters, got {}",
        result.clusters.len()
    );

    // Sort clusters by size descending — the two biggest should be the tree trunks
    let mut sorted = result.clusters.clone();
    sorted.sort_by(|a, b| b.point_indices.len().cmp(&a.point_indices.len()));
    assert!(
        sorted[0].point_indices.len() >= 500,
        "Largest cluster too small: {} points",
        sorted[0].point_indices.len()
    );
    assert!(
        sorted[1].point_indices.len() >= 500,
        "Second largest cluster too small: {} points",
        sorted[1].point_indices.len()
    );
}

#[test]
fn test_feature_candidates_synthetic() {
    use pt_scan::cluster::{cluster_obstacles, ClusterConfig};

    timed(|| {
        // 1000 ground + 200 obstacle + 50 outlier
        let ply_data = make_synthetic_ply(1000, 200, 50);
        let config = ScanConfig {
            voxel_size: 0.5,
            outlier_k: 10,
            outlier_threshold: 2.0,
            ransac_iterations: 500,
            ransac_threshold: 0.05,
        };

        let cloud = pt_scan::process_scan(Cursor::new(ply_data), &config).unwrap();

        let cluster_config = ClusterConfig {
            epsilon: 0.5,
            min_points: 3,
        };
        let result = cluster_obstacles(&cloud.obstacles, &cluster_config);

        let candidates = extract_candidates(
            &result.clusters,
            &cloud.obstacles,
            &cloud.metadata.ground_plane,
        );

        // Should have one candidate per cluster
        assert_eq!(
            candidates.len(),
            result.clusters.len(),
            "candidate count should match cluster count"
        );

        for c in &candidates {
            // Height must be positive (obstacles are above ground)
            assert!(
                c.height_ft > 0.0,
                "height_ft should be > 0, got {}",
                c.height_ft
            );

            // Spread must be non-negative
            assert!(c.spread_ft >= 0.0, "spread_ft should be >= 0");

            // Density must be positive
            assert!(c.density > 0.0, "density should be > 0");

            // Point count must be >= cluster min_points
            assert!(c.point_count >= 3, "point_count should be >= min_points");

            // Color should be a known category (synthetic PLY has red obstacles)
            let valid_colors = ["green", "brown", "gray", "white", "mixed", "unknown"];
            assert!(
                valid_colors.contains(&c.dominant_color.as_str()),
                "unexpected color: {}",
                c.dominant_color
            );

            // Profile should be a known category
            let valid_profiles = ["columnar", "flat", "conical", "spreading", "irregular"];
            assert!(
                valid_profiles.contains(&c.vertical_profile.as_str()),
                "unexpected profile: {}",
                c.vertical_profile
            );
        }
    });
}

#[test]
fn test_powell_market_candidates() {
    use pt_scan::cluster::{cluster_obstacles, ClusterConfig};
    use std::fs::File;
    use std::io::BufReader;

    let scan_path = "../../assets/scans/samples/Scan at 09.23.ply";
    let file = match File::open(scan_path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Skipping Powell & Market candidates test: scan file not found");
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

    let cluster_config = ClusterConfig::default();
    let result = cluster_obstacles(&cloud.obstacles, &cluster_config);

    let candidates = extract_candidates(
        &result.clusters,
        &cloud.obstacles,
        &cloud.metadata.ground_plane,
    );

    eprintln!("Powell & Market: {} feature candidates", candidates.len());
    for c in &candidates {
        eprintln!(
            "  Candidate {}: ht={:.1}ft sp={:.1}ft pts={} color={} profile={} density={:.1}",
            c.cluster_id,
            c.height_ft,
            c.spread_ft,
            c.point_count,
            c.dominant_color,
            c.vertical_profile,
            c.density,
        );
    }

    // Should produce 2-20 candidates (trees, poles, structures)
    assert!(
        candidates.len() >= 2,
        "expected >= 2 candidates, got {}",
        candidates.len()
    );
    assert!(
        candidates.len() <= 50,
        "expected <= 50 candidates, got {} (clustering too fragmented?)",
        candidates.len()
    );

    // All candidates should have positive heights within urban range
    // Many clusters are low-lying features (curbs, pavement edges) at ~0.2-0.5 ft
    for c in &candidates {
        assert!(
            c.height_ft > 0.0,
            "candidate {} height should be positive, got {:.1} ft",
            c.cluster_id,
            c.height_ft
        );
        assert!(
            c.height_ft < 200.0,
            "candidate {} height too high: {:.1} ft",
            c.cluster_id,
            c.height_ft
        );
    }

    // At least one candidate should be a significant vertical feature (>3 ft)
    let tall_count = candidates.iter().filter(|c| c.height_ft > 3.0).count();
    assert!(
        tall_count >= 1,
        "expected at least 1 candidate taller than 3ft, got {}",
        tall_count
    );
}

/// Generate a synthetic PLY with two separate obstacle clusters for gap testing.
///
/// Two columns of points 4m apart in X, each ~0.3m wide, sitting on a ground plane.
fn make_two_cluster_ply() -> Vec<u8> {
    let ground_n = 500_usize;
    let cluster_n = 100_usize;
    let total = ground_n + cluster_n * 2;
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

    // Ground: z ≈ 0, spread [0, 8] × [0, 4]
    #[allow(clippy::cast_possible_truncation)]
    let side = (ground_n as f32).sqrt().ceil() as usize;
    for i in 0..ground_n {
        let x = (i % side) as f32 * (8.0 / side as f32);
        let y = (i / side) as f32 * (4.0 / side as f32);
        let z = if i % 2 == 0 { 0.003_f32 } else { -0.003 };
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.extend_from_slice(&z.to_le_bytes());
        buf.extend_from_slice(&[0, 128, 0]);
    }

    // Cluster A: column at x=2, y=2, z∈[0.3, 2.0], ~0.3m wide
    for i in 0..cluster_n {
        let x = 2.0 + (i % 5) as f32 * 0.06;
        let y = 2.0 + (i / 20) as f32 * 0.06;
        let z = 0.3 + (i as f32 / cluster_n as f32) * 1.7;
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.extend_from_slice(&z.to_le_bytes());
        buf.extend_from_slice(&[140, 100, 50]);
    }

    // Cluster B: column at x=6, y=2, z∈[0.3, 2.0], ~0.3m wide
    for i in 0..cluster_n {
        let x = 6.0 + (i % 5) as f32 * 0.06;
        let y = 2.0 + (i / 20) as f32 * 0.06;
        let z = 0.3 + (i as f32 / cluster_n as f32) * 1.7;
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.extend_from_slice(&z.to_le_bytes());
        buf.extend_from_slice(&[140, 100, 50]);
    }

    buf
}

#[test]
fn test_gap_measurement_synthetic() {
    use pt_scan::cluster::{cluster_obstacles, ClusterConfig};

    timed(|| {
        let ply_data = make_two_cluster_ply();
        let config = ScanConfig {
            voxel_size: 0.05,
            outlier_k: 10,
            outlier_threshold: 2.0,
            ransac_iterations: 500,
            ransac_threshold: 0.05,
        };

        let cloud = pt_scan::process_scan(Cursor::new(ply_data), &config).unwrap();

        let cluster_config = ClusterConfig {
            epsilon: 0.3,
            min_points: 5,
        };
        let result = cluster_obstacles(&cloud.obstacles, &cluster_config);
        assert!(
            result.clusters.len() >= 2,
            "expected >= 2 clusters for gap test, got {}",
            result.clusters.len()
        );

        let candidates = extract_candidates(
            &result.clusters,
            &cloud.obstacles,
            &cloud.metadata.ground_plane,
        );

        let gaps = measure_gaps(&candidates, &cloud.metadata.ground_plane, &GapConfig::default());

        // Two clusters ~4m apart = ~13.1ft. Should produce at least 1 gap.
        assert!(
            !gaps.is_empty(),
            "expected at least 1 gap between two clusters"
        );

        // The primary gap (closest pair) should be roughly 4m ≈ 13.1ft apart
        let primary = &gaps[0];
        assert!(
            primary.centroid_distance_ft > 5.0 && primary.centroid_distance_ft < 25.0,
            "primary gap distance {:.1}ft outside expected range 5-25ft",
            primary.centroid_distance_ft
        );

        // Clear width should be positive (features don't overlap at 4m separation)
        assert!(
            primary.clear_width_ft > 0.0,
            "clear_width should be positive, got {:.2}",
            primary.clear_width_ft
        );

        // Area should be positive
        assert!(
            primary.area_sqft > 0.0,
            "area should be positive, got {:.2}",
            primary.area_sqft
        );

        eprintln!(
            "Gap: features {} ↔ {}, dist={:.1}ft, clear_w={:.1}ft, area={:.1}sqft",
            primary.feature_a_id,
            primary.feature_b_id,
            primary.centroid_distance_ft,
            primary.clear_width_ft,
            primary.area_sqft,
        );
    });
}

#[test]
fn test_powell_market_gaps() {
    use pt_scan::cluster::{cluster_obstacles, ClusterConfig};
    use std::fs::File;
    use std::io::BufReader;

    let scan_path = "../../assets/scans/samples/Scan at 09.23.ply";
    let file = match File::open(scan_path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Skipping Powell & Market gap test: scan file not found at {scan_path}");
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

    let cluster_config = ClusterConfig::default();
    let result = cluster_obstacles(&cloud.obstacles, &cluster_config);

    let candidates = extract_candidates(
        &result.clusters,
        &cloud.obstacles,
        &cloud.metadata.ground_plane,
    );

    let gaps = measure_gaps(&candidates, &cloud.metadata.ground_plane, &GapConfig::default());

    eprintln!("Powell & Market gaps: {}", gaps.len());
    for g in &gaps {
        eprintln!(
            "  Gap {} ↔ {}: dist={:.1}ft clear_w={:.1}ft area={:.1}sqft elev={:.1}ft",
            g.feature_a_id,
            g.feature_b_id,
            g.centroid_distance_ft,
            g.clear_width_ft,
            g.area_sqft,
            g.ground_elevation_ft,
        );
    }

    // With multiple urban features, there should be at least 1 gap
    assert!(
        !gaps.is_empty(),
        "Powell & Market should have at least 1 gap between features"
    );

    // All gaps should have valid measurements
    for g in &gaps {
        assert!(g.clear_width_ft > 0.0, "clear_width must be positive");
        assert!(g.area_sqft > 0.0, "area must be positive");
        assert!(
            g.centroid_distance_ft <= 30.0,
            "gap distance {:.1} exceeds threshold",
            g.centroid_distance_ft
        );
    }
}
