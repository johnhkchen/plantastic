//! Debug segmentation with Rerun 3D viewer.
//!
//! Runs a PLY file through the scan pipeline, computes eigenvalue features,
//! clusters with HDBSCAN, and logs each stage to Rerun for visual inspection.
//!
//! Usage:
//!     cargo run -p pt-scan --example debug_segmentation --release -- [path.ply]
//!     just debug-scan [path]
//!
//! The Rerun viewer opens automatically. Use the timeline scrubber to step
//! through stages: raw → features → clustered → candidates.

use std::env;
use std::fs;
use std::io::BufReader;
use std::path::Path;

use pt_scan::cluster::HdbscanConfig;
use pt_scan::eigenvalue::compute_point_features;
use pt_scan::types::Point;
use pt_scan::{extract_candidates, hdbscan_cluster, ScanConfig};

const DEFAULT_PLY: &str = "assets/scans/samples/powell-market-downsampled.ply";

const VOXEL_SIZE: f32 = 0.05;
const OUTLIER_K: usize = 20;
const OUTLIER_THRESHOLD: f32 = 2.0;
const RANSAC_ITERATIONS: usize = 1000;
const RANSAC_THRESHOLD: f32 = 0.05;
const EIGENVALUE_K: usize = 30;

fn main() {
    let ply_path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PLY.to_string());
    let path = Path::new(&ply_path);

    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        eprintln!(
            "Usage: cargo run -p pt-scan --example debug_segmentation --release -- [path.ply]"
        );
        std::process::exit(1);
    }

    println!("── Debug Segmentation ───────────────────────────────");
    println!("Input: {}", path.display());
    println!();

    // Process scan
    let config = ScanConfig {
        voxel_size: VOXEL_SIZE,
        outlier_k: OUTLIER_K,
        outlier_threshold: OUTLIER_THRESHOLD,
        ransac_iterations: RANSAC_ITERATIONS,
        ransac_threshold: RANSAC_THRESHOLD,
    };

    let file = fs::File::open(path).expect("cannot open PLY file");
    let reader = BufReader::new(file);
    let (cloud, report) =
        pt_scan::process_scan_timed(reader, &config).expect("scan processing failed");

    println!(
        "Processed: {} ground, {} obstacles",
        cloud.ground.len(),
        cloud.obstacles.len()
    );

    // Initialize Rerun
    let rec = rerun::RecordingStreamBuilder::new("debug_segmentation")
        .spawn()
        .expect("failed to spawn Rerun viewer");

    // Stage 0: Raw points
    rec.set_time_sequence("stage", 0);

    log_points(&rec, "ground/points", &cloud.ground, false);
    log_points(&rec, "obstacles/points", &cloud.obstacles, false);
    println!("[stage 0] Logged raw points");

    // Stage 1: Eigenvalue features
    rec.set_time_sequence("stage", 1);

    let features = compute_point_features(&cloud.obstacles, EIGENVALUE_K);
    println!("[stage 1] Computed {} eigenvalue features", features.len());

    let positions: Vec<[f32; 3]> = cloud.obstacles.iter().map(|p| p.position).collect();

    // Planarity heatmap
    let planarity_colors: Vec<[u8; 3]> = features
        .iter()
        .map(|f| value_to_color(f.planarity))
        .collect();
    rec.log(
        "features/planarity",
        &rerun::Points3D::new(&positions)
            .with_colors(planarity_colors)
            .with_radii([0.02]),
    )
    .ok();

    // Linearity heatmap
    let linearity_colors: Vec<[u8; 3]> = features
        .iter()
        .map(|f| value_to_color(f.linearity))
        .collect();
    rec.log(
        "features/linearity",
        &rerun::Points3D::new(&positions)
            .with_colors(linearity_colors)
            .with_radii([0.02]),
    )
    .ok();

    // Sphericity heatmap
    let sphericity_colors: Vec<[u8; 3]> = features
        .iter()
        .map(|f| value_to_color(f.sphericity))
        .collect();
    rec.log(
        "features/sphericity",
        &rerun::Points3D::new(&positions)
            .with_colors(sphericity_colors)
            .with_radii([0.02]),
    )
    .ok();

    println!("[stage 1] Logged feature heatmaps");

    // Stage 2: HDBSCAN clustering
    rec.set_time_sequence("stage", 2);

    let hdbscan_config = HdbscanConfig::default();
    let cluster_result = hdbscan_cluster(&cloud.obstacles, &features, &hdbscan_config);

    // Clustered points — colored by cluster ID
    let mut cluster_labels = vec![-1i32; cloud.obstacles.len()];
    for cluster in &cluster_result.clusters {
        for &idx in &cluster.point_indices {
            cluster_labels[idx] = cluster.id as i32;
        }
    }

    let assigned: Vec<usize> = (0..cloud.obstacles.len())
        .filter(|i| cluster_labels[*i] >= 0)
        .collect();
    let assigned_positions: Vec<[f32; 3]> = assigned
        .iter()
        .map(|&i| cloud.obstacles[i].position)
        .collect();
    let assigned_colors: Vec<[u8; 3]> = assigned
        .iter()
        .map(|&i| {
            #[allow(clippy::cast_sign_loss)]
            cluster_color(cluster_labels[i] as u32)
        })
        .collect();

    rec.log(
        "clusters/assigned",
        &rerun::Points3D::new(&assigned_positions)
            .with_colors(assigned_colors)
            .with_radii([0.02]),
    )
    .ok();

    // Noise points — gray
    let noise_positions: Vec<[f32; 3]> = cluster_result
        .noise_indices
        .iter()
        .map(|&i| cloud.obstacles[i].position)
        .collect();

    if !noise_positions.is_empty() {
        rec.log(
            "clusters/noise",
            &rerun::Points3D::new(&noise_positions)
                .with_colors([[128u8, 128, 128]])
                .with_radii([0.01]),
        )
        .ok();
    }

    println!(
        "[stage 2] Logged {} clusters, {} noise points",
        cluster_result.clusters.len(),
        cluster_result.noise_indices.len()
    );

    // Stage 3: Feature candidates as bounding boxes
    rec.set_time_sequence("stage", 3);

    let candidates = extract_candidates(
        &cluster_result.clusters,
        &cloud.obstacles,
        &cloud.metadata.ground_plane,
    );

    #[allow(clippy::cast_possible_truncation)]
    if !candidates.is_empty() {
        let centers: Vec<[f32; 3]> = candidates
            .iter()
            .map(|c| {
                [
                    (c.bbox_min[0] + c.bbox_max[0]) as f32 / 2.0,
                    (c.bbox_min[1] + c.bbox_max[1]) as f32 / 2.0,
                    (c.bbox_min[2] + c.bbox_max[2]) as f32 / 2.0,
                ]
            })
            .collect();

        let half_sizes: Vec<[f32; 3]> = candidates
            .iter()
            .map(|c| {
                [
                    (c.bbox_max[0] - c.bbox_min[0]) as f32 / 2.0,
                    (c.bbox_max[1] - c.bbox_min[1]) as f32 / 2.0,
                    (c.bbox_max[2] - c.bbox_min[2]) as f32 / 2.0,
                ]
            })
            .collect();

        let labels: Vec<String> = candidates
            .iter()
            .map(|c| {
                format!(
                    "#{} h={:.1}ft s={:.1}ft n={}",
                    c.cluster_id, c.height_ft, c.spread_ft, c.point_count
                )
            })
            .collect();

        let box_colors: Vec<[u8; 3]> = candidates
            .iter()
            .map(|c| {
                #[allow(clippy::cast_possible_truncation)]
                cluster_color(c.cluster_id as u32)
            })
            .collect();

        rec.log(
            "candidates/boxes",
            &rerun::Boxes3D::from_centers_and_half_sizes(centers, half_sizes)
                .with_labels(labels)
                .with_colors(box_colors),
        )
        .ok();
    }

    println!(
        "[stage 3] Logged {} candidate bounding boxes",
        candidates.len()
    );

    // Summary
    println!();
    println!("── Summary ──────────────────────────────────────────");
    println!("Processing: {}ms", report.timing.total_processing_ms);
    println!(
        "Ground: {} pts | Obstacles: {} pts",
        cloud.ground.len(),
        cloud.obstacles.len()
    );
    println!(
        "Clusters: {} | Noise: {} pts",
        cluster_result.clusters.len(),
        cluster_result.noise_indices.len()
    );
    println!("Candidates: {}", candidates.len());
    println!();
    println!("Rerun viewer should be open. Use the timeline to step through stages.");
    println!("  stage 0: raw points (ground + obstacles)");
    println!("  stage 1: eigenvalue feature heatmaps");
    println!("  stage 2: HDBSCAN cluster assignments");
    println!("  stage 3: feature candidate bounding boxes");
}

/// Map a [0, 1] value to a blue→red color gradient.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn value_to_color(v: f32) -> [u8; 3] {
    let v = v.clamp(0.0, 1.0);
    let r = (v * 255.0) as u8;
    let b = ((1.0 - v) * 255.0) as u8;
    [r, 0, b]
}

/// Generate a distinct color for a cluster ID using golden ratio hue spacing.
fn cluster_color(id: u32) -> [u8; 3] {
    let hue = ((id as f32) * 0.618_034) % 1.0;
    hsv_to_rgb(hue, 0.85, 0.95)
}

/// Convert HSV [0,1] to RGB [0,255].
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let i = (h * 6.0) as u32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
}

/// Log points to a Rerun entity path with their RGB colors.
fn log_points(rec: &rerun::RecordingStream, entity_path: &str, points: &[Point], small: bool) {
    let positions: Vec<[f32; 3]> = points.iter().map(|p| p.position).collect();
    let colors: Vec<[u8; 3]> = points
        .iter()
        .map(|p| p.color.unwrap_or([255, 255, 255]))
        .collect();
    let radius = if small { 0.01 } else { 0.02 };

    rec.log(
        entity_path,
        &rerun::Points3D::new(&positions)
            .with_colors(colors)
            .with_radii([radius]),
    )
    .ok();
}
