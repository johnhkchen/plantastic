//! Scan processing CLI example.
//!
//! Runs a PLY file through the full pt-scan pipeline with per-stage timing
//! and writes a structured JSON report alongside the output files.
//!
//! Usage:
//!     cargo run -p pt-scan --example process_sample --release -- [path.ply]
//!
//! Default input: assets/scans/samples/Scan at 09.23.ply

use std::env;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use pt_scan::cluster::{cluster_obstacles, ClusterConfig};
use pt_scan::export::ExportConfig;
use pt_scan::{extract_candidates, measure_gaps, GapConfig, OutputInfo, ScanConfig};

const DEFAULT_PLY: &str = "assets/scans/samples/Scan at 09.23.ply";

// Outdoor urban scan config (per ticket spec)
const VOXEL_SIZE: f32 = 0.05; // 5cm
const OUTLIER_K: usize = 20;
const OUTLIER_THRESHOLD: f32 = 2.0;
const RANSAC_ITERATIONS: usize = 1000;
const RANSAC_THRESHOLD: f32 = 0.05; // 5cm

fn main() {
    let ply_path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PLY.to_string());
    let path = Path::new(&ply_path);

    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        eprintln!("Usage: cargo run -p pt-scan --example process_sample --release -- [path.ply]");
        std::process::exit(1);
    }

    let file_size = fs::metadata(path).expect("cannot read file metadata").len();
    let stem = path.file_stem().expect("no file stem").to_string_lossy();
    let parent = path.parent().expect("no parent directory");

    println!("── Scan Processing Pipeline ──────────────────────────");
    println!("Input: {} ({})", path.display(), format_bytes(file_size));
    println!();

    let config = ScanConfig {
        voxel_size: VOXEL_SIZE,
        outlier_k: OUTLIER_K,
        outlier_threshold: OUTLIER_THRESHOLD,
        ransac_iterations: RANSAC_ITERATIONS,
        ransac_threshold: RANSAC_THRESHOLD,
    };

    let file = fs::File::open(path).expect("cannot open PLY file");
    let reader = BufReader::new(file);
    let (cloud, mut report) =
        pt_scan::process_scan_timed(reader, &config).expect("scan processing failed");

    // Fill caller-side input metadata
    report.input.filename = Some(
        path.file_name()
            .expect("no filename")
            .to_string_lossy()
            .into_owned(),
    );
    report.input.file_size_bytes = Some(file_size);

    // Print processing summary
    println!(
        "[1/5] Parse PLY .............. {}ms   {} points",
        report.timing.parse_ms,
        format_count(report.input.original_vertex_count),
    );
    println!(
        "[2/5] Voxel downsample ...... {}ms   {} points ({:.0}cm voxels)",
        report.timing.downsample_ms,
        format_count(report.processing.downsampled_count),
        VOXEL_SIZE * 100.0,
    );
    println!(
        "[3/5] Outlier removal ....... {}ms   {} removed",
        report.timing.outlier_removal_ms, report.processing.outliers_removed,
    );
    println!(
        "[4/5] RANSAC ground fit ..... {}ms   ground: {} | obstacles: {} (best iter: {})",
        report.timing.ransac_ms,
        format_count(report.ground.point_count),
        format_count(report.obstacles.count),
        report.processing.ransac_best_iteration,
    );

    // Stage 5: Terrain generation (mesh + GLB + PNG)
    let stage_start = Instant::now();
    let terrain =
        pt_scan::generate_terrain(&cloud, &ExportConfig::default()).expect("terrain gen failed");
    #[allow(clippy::cast_possible_truncation)]
    let terrain_ms = stage_start.elapsed().as_millis() as u64;

    // Fill output metadata
    report.output = Some(OutputInfo::from_terrain(&terrain));
    report.timing.terrain_export_ms = Some(terrain_ms);

    println!(
        "[5/5] Terrain generation .... {}ms   {} triangles",
        terrain_ms,
        format_count(terrain.metadata.decimated_triangle_count),
    );

    // Write output files
    let glb_path = parent.join(format!("{stem}-terrain.glb"));
    let png_path = parent.join(format!("{stem}-planview.png"));
    let report_path = parent.join(format!("{stem}-report.json"));

    fs::write(&glb_path, &terrain.mesh_glb).expect("failed to write GLB");
    fs::write(&png_path, &terrain.plan_view_png).expect("failed to write PNG");

    let report_json = serde_json::to_string_pretty(&report).expect("failed to serialize report");
    fs::write(&report_path, &report_json).expect("failed to write report JSON");

    // Stage 6: Clustering + feature candidate extraction
    let stage_start = Instant::now();
    let cluster_config = ClusterConfig::default();
    let cluster_result = cluster_obstacles(&cloud.obstacles, &cluster_config);
    let candidates = extract_candidates(
        &cluster_result.clusters,
        &cloud.obstacles,
        &cloud.metadata.ground_plane,
    );
    #[allow(clippy::cast_possible_truncation)]
    let cluster_ms = stage_start.elapsed().as_millis() as u64;

    println!(
        "[6/6] Feature extraction .... {cluster_ms}ms   {} clusters, {} noise pts",
        candidates.len(),
        cluster_result.noise_indices.len(),
    );

    // Feature candidates table
    if !candidates.is_empty() {
        println!();
        println!("── Feature Candidates ────────────────────────────────");
        println!(
            "{:>3}  {:>8}  {:>8}  {:>6}  {:>9}  {:>10}  {:>10}",
            "ID", "Ht(ft)", "Sp(ft)", "Pts", "Density", "Color", "Profile"
        );
        for c in &candidates {
            println!(
                "{:>3}  {:>8.1}  {:>8.1}  {:>6}  {:>9.1}  {:>10}  {:>10}",
                c.cluster_id,
                c.height_ft,
                c.spread_ft,
                c.point_count,
                c.density,
                c.dominant_color,
                c.vertical_profile,
            );
        }
    }

    // Stage 7: Gap measurement
    let stage_start = Instant::now();
    let gaps = measure_gaps(
        &candidates,
        &cloud.metadata.ground_plane,
        &GapConfig::default(),
    );
    #[allow(clippy::cast_possible_truncation)]
    let gap_ms = stage_start.elapsed().as_millis() as u64;

    println!(
        "[7/7] Gap measurement ...... {gap_ms}ms   {} gaps",
        gaps.len(),
    );

    if !gaps.is_empty() {
        println!();
        println!("── Gaps ──────────────────────────────────────────────");
        println!(
            "{:>3} {:>3}  {:>9}  {:>9}  {:>9}  {:>9}  {:>8}",
            "A", "B", "Dist(ft)", "Width(ft)", "Len(ft)", "Area(sf)", "Elev(ft)"
        );
        for g in &gaps {
            println!(
                "{:>3} {:>3}  {:>9.1}  {:>9.1}  {:>9.1}  {:>9.1}  {:>8.1}",
                g.feature_a_id,
                g.feature_b_id,
                g.centroid_distance_ft,
                g.clear_width_ft,
                g.clear_length_ft,
                g.area_sqft,
                g.ground_elevation_ft,
            );
        }
    }

    // Metadata summary
    println!();
    println!("── Metadata ──────────────────────────────────────────");
    println!(
        "Ground plane: normal=[{:.3}, {:.3}, {:.3}] d={:.3}",
        report.ground.plane.normal[0],
        report.ground.plane.normal[1],
        report.ground.plane.normal[2],
        report.ground.plane.d,
    );
    println!(
        "Ground area estimate: {:.1} m²",
        report.ground.area_estimate_sqm,
    );

    if let Some([min_h, max_h]) = report.obstacles.height_range {
        println!("Obstacle height range: {min_h:.2} – {max_h:.2} m");
    }

    println!();
    println!("── Output ────────────────────────────────────────────");
    println!(
        "Terrain: {} ({})",
        glb_path.display(),
        format_bytes(terrain.mesh_glb.len() as u64),
    );
    println!(
        "Plan view: {} ({})",
        png_path.display(),
        format_bytes(terrain.plan_view_png.len() as u64),
    );
    println!(
        "Report: {} ({})",
        report_path.display(),
        format_bytes(report_json.len() as u64),
    );
    println!(
        "Total: {}ms",
        report.timing.total_processing_ms + terrain_ms,
    );
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.0} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{bytes} B")
    }
}

fn format_count(n: usize) -> String {
    if n >= 1_000_000 {
        format!(
            "{},{:03},{:03}",
            n / 1_000_000,
            (n / 1_000) % 1_000,
            n % 1_000
        )
    } else if n >= 1_000 {
        format!("{},{:03}", n / 1_000, n % 1_000)
    } else {
        n.to_string()
    }
}
