//! Scan-to-quote demo: full pipeline from PLY scan to three-tier landscaping quote.
//!
//! Runs the entire Plantastic product loop on a single PLY file:
//! scan → detect → classify → measure → design → quote.
//!
//! Usage:
//!     cargo run -p pt-scan --example scan_to_quote --release -- [path.ply]
//!     just scan-to-quote [path]
//!
//! Default input: assets/scans/samples/powell-market-downsampled.ply

use std::env;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

use geo::polygon;
use rust_decimal::Decimal;

use pt_materials::{ExtrusionBehavior, Material, MaterialCategory, Unit};
use pt_project::{MaterialAssignment, Tier, TierLevel, Zone, ZoneId, ZoneType};
use pt_quote::compute_quote;
use pt_scan::cluster::{cluster_obstacles, ClusterConfig};
use pt_scan::export::ExportConfig;
use pt_scan::{extract_candidates, measure_gaps, FeatureCandidate, Gap, GapConfig, ScanConfig};

const DEFAULT_PLY: &str = "assets/scans/samples/powell-market-downsampled.ply";

// Outdoor urban scan config (matching process_sample.rs)
const VOXEL_SIZE: f32 = 0.05;
const OUTLIER_K: usize = 20;
const OUTLIER_THRESHOLD: f32 = 2.0;
const RANSAC_ITERATIONS: usize = 1000;
const RANSAC_THRESHOLD: f32 = 0.05;

fn main() {
    let args: Vec<String> = env::args().collect();
    let live = args.iter().any(|a| a == "--live");
    let ply_path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .map(String::as_str)
        .unwrap_or(DEFAULT_PLY);
    let path = Path::new(ply_path);

    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        eprintln!(
            "Usage: cargo run -p pt-scan --example scan_to_quote --release -- [path.ply] [--live]"
        );
        std::process::exit(1);
    }

    let stem = path.file_stem().expect("no file stem").to_string_lossy();
    let parent = path.parent().expect("no parent directory");

    println!();
    println!("POWELL & MARKET PLANTER — SCAN ANALYSIS");
    println!("========================================");
    println!();

    // ── Stage 1: Process scan ─────────────────────────────────
    let stage_start = Instant::now();
    let config = ScanConfig {
        voxel_size: VOXEL_SIZE,
        outlier_k: OUTLIER_K,
        outlier_threshold: OUTLIER_THRESHOLD,
        ransac_iterations: RANSAC_ITERATIONS,
        ransac_threshold: RANSAC_THRESHOLD,
    };

    let file = fs::File::open(path).expect("cannot open PLY file");
    let reader = BufReader::new(file);
    let (cloud, _report) =
        pt_scan::process_scan_timed(reader, &config).expect("scan processing failed");
    let scan_ms = stage_start.elapsed().as_millis();
    println!(
        "[1/7] Scan processing ...... {scan_ms}ms   {} ground, {} obstacles",
        cloud.ground.len(),
        cloud.obstacles.len(),
    );

    // ── Stage 2: Cluster obstacles ────────────────────────────
    let stage_start = Instant::now();
    let cluster_config = ClusterConfig::default();
    let cluster_result = cluster_obstacles(&cloud.obstacles, &cluster_config);
    let candidates = extract_candidates(
        &cluster_result.clusters,
        &cloud.obstacles,
        &cloud.metadata.ground_plane,
    );
    let cluster_ms = stage_start.elapsed().as_millis();
    println!(
        "[2/7] Clustering ........... {cluster_ms}ms   {} features detected",
        candidates.len(),
    );

    // ── Stage 3: Classify features ────────────────────────────
    let stage_start = Instant::now();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let classified = if live {
        let classifier = pt_features::ClaudeCliClassifier;
        rt.block_on(pt_features::FeatureClassifier::classify(
            &classifier,
            &candidates,
            "Powell & Market, San Francisco, CA",
            "USDA 10b",
        ))
        .expect("live classification failed")
    } else {
        let classifier = pt_features::MockFeatureClassifier;
        rt.block_on(pt_features::FeatureClassifier::classify(
            &classifier,
            &candidates,
            "Powell & Market, San Francisco, CA",
            "USDA 10b",
        ))
        .expect("mock classification failed")
    };
    let classify_ms = stage_start.elapsed().as_millis();

    let tree_count = classified.iter().filter(|c| c.category == "tree").count();
    println!(
        "[3/7] Classification ....... {classify_ms}ms   {} trees, {} total ({})",
        tree_count,
        classified.len(),
        if live { "live" } else { "mock" },
    );

    for cls in &classified {
        println!(
            "      [{:>9}] {} ({:.0}%)",
            cls.category,
            cls.label,
            cls.confidence * 100.0
        );
    }

    // ── Stage 4: Measure gaps ─────────────────────────────────
    let stage_start = Instant::now();
    let gaps = measure_gaps(
        &candidates,
        &cloud.metadata.ground_plane,
        &GapConfig::default(),
    );
    let gap_ms = stage_start.elapsed().as_millis();
    println!(
        "[4/7] Gap measurement ...... {gap_ms}ms   {} gaps",
        gaps.len()
    );

    if gaps.is_empty() {
        println!();
        println!("No plantable gaps found between features. Need at least 2 features.");
        std::process::exit(0);
    }

    // Use the gap with the largest plantable area
    let gap = gaps
        .iter()
        .max_by(|a, b| a.area_sqft.partial_cmp(&b.area_sqft).unwrap())
        .expect("gaps not empty");
    println!(
        "      Best gap: {:.1} ft wide × {:.1} ft long = {:.1} sqft",
        gap.clear_width_ft, gap.clear_length_ft, gap.area_sqft,
    );

    // ── Stage 5: Build zones + materials + tiers ──────────────
    let stage_start = Instant::now();
    let zone = gap_to_zone(gap, &candidates);
    let area_sqft = gap.area_sqft;

    let (good_mats, better_mats, best_mats) = build_materials();
    let good_tier = build_tier(TierLevel::Good, &zone, &good_mats);
    let better_tier = build_tier(TierLevel::Better, &zone, &better_mats);
    let best_tier = build_tier(TierLevel::Best, &zone, &best_mats);

    let zones = std::slice::from_ref(&zone);
    let good_quote = compute_quote(zones, &good_tier, &good_mats, None).expect("good quote failed");
    let better_quote =
        compute_quote(zones, &better_tier, &better_mats, None).expect("better quote failed");
    let best_quote = compute_quote(zones, &best_tier, &best_mats, None).expect("best quote failed");
    let quote_ms = stage_start.elapsed().as_millis();
    println!("[5/7] Quote computation .... {quote_ms}ms   3 tiers");

    // ── Stage 6: Terrain GLB ──────────────────────────────────
    let stage_start = Instant::now();
    let terrain =
        pt_scan::generate_terrain(&cloud, &ExportConfig::default()).expect("terrain gen failed");
    let terrain_ms = stage_start.elapsed().as_millis();

    let glb_path = parent.join(format!("{stem}-terrain.glb"));
    let png_path = parent.join(format!("{stem}-planview.png"));
    fs::write(&glb_path, &terrain.mesh_glb).expect("failed to write GLB");
    fs::write(&png_path, &terrain.plan_view_png).expect("failed to write PNG");
    println!(
        "[6/7] Terrain export ....... {terrain_ms}ms   {} triangles",
        terrain.metadata.decimated_triangle_count,
    );

    // ── Stage 7: Print summary ────────────────────────────────
    println!("[7/7] Summary");
    println!();
    println!("──────────────────────────────────────────────────────");
    println!(
        "Site: {} features detected ({} trees), {:.1} ft gap, {:.0} sqft plantable area",
        classified.len(),
        tree_count,
        gap.clear_width_ft,
        area_sqft,
    );
    println!();

    print_tier_quote("GOOD", "Low-Maintenance Succulents", &good_quote, area_sqft);
    print_tier_quote("BETTER", "Ornamental Grasses", &better_quote, area_sqft);
    print_tier_quote("BEST", "Seasonal Color Display", &best_quote, area_sqft);

    println!("──────────────────────────────────────────────────────");
    println!("Terrain: {}", glb_path.display());
    println!("Plan view: {}", png_path.display());
    println!();
}

/// Convert a measured gap into a Zone polygon (rectangle in feet).
fn gap_to_zone(gap: &Gap, candidates: &[FeatureCandidate]) -> Zone {
    // Find the two features forming this gap to get orientation
    let a = candidates
        .iter()
        .find(|c| c.cluster_id == gap.feature_a_id)
        .expect("feature_a not found");
    let b = candidates
        .iter()
        .find(|c| c.cluster_id == gap.feature_b_id)
        .expect("feature_b not found");

    // Midpoint in feet (gap stores midpoint in meters)
    let mx = gap.midpoint[0] * 3.28084;
    let my = gap.midpoint[1] * 3.28084;

    // Direction vector between features
    let dx = b.centroid[0] - a.centroid[0];
    let dy = b.centroid[1] - a.centroid[1];
    let len = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = if len > 1e-6 {
        (dx / len, dy / len)
    } else {
        (1.0, 0.0)
    };

    // Perpendicular
    let (px, py) = (-uy, ux);

    let hw = gap.clear_width_ft / 2.0;
    let hl = gap.clear_length_ft / 2.0;

    // Rectangle corners: along gap direction (width) and perpendicular (length)
    let corners = [
        (mx + ux * hw + px * hl, my + uy * hw + py * hl),
        (mx - ux * hw + px * hl, my - uy * hw + py * hl),
        (mx - ux * hw - px * hl, my - uy * hw - py * hl),
        (mx + ux * hw - px * hl, my + uy * hw - py * hl),
    ];

    Zone {
        id: ZoneId::new(),
        geometry: polygon![
            (x: corners[0].0, y: corners[0].1),
            (x: corners[1].0, y: corners[1].1),
            (x: corners[2].0, y: corners[2].1),
            (x: corners[3].0, y: corners[3].1),
        ],
        zone_type: ZoneType::Bed,
        label: Some("Planter".to_string()),
    }
}

/// Build the material catalog for each tier.
///
/// Plant pricing uses SqFt so pt-quote computes: area × (price_per_plant / spacing²) = total.
/// This means the unit_price = per-plant price ÷ (spacing_ft × spacing_ft).
fn build_materials() -> (Vec<Material>, Vec<Material>, Vec<Material>) {
    // ── Good: Low-Maintenance Succulents ──────────────────────
    // Echeveria 'Lola' at 8" center-to-center spacing (0.667 ft), $5/plant
    // price_per_sqft = 5.0 / (0.667 × 0.667) = $11.24/sqft
    let spacing_good = 8.0 / 12.0;
    let good_plants = Material::builder(
        "Echeveria 'Lola' (8\" spacing)",
        MaterialCategory::Softscape,
    )
    .unit(Unit::SqFt)
    .price_per_unit(
        Decimal::from_str("5.00").expect("decimal") / decimal_f64(spacing_good * spacing_good),
    )
    .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 4.0 })
    .build();
    let good_soil = Material::builder("Planting soil", MaterialCategory::Fill)
        .unit(Unit::CuYd)
        .price_per_unit(Decimal::from_str("45.00").expect("decimal"))
        .depth_inches(6.0)
        .extrusion(ExtrusionBehavior::Fills { flush: true })
        .build();

    // ── Better: Ornamental Grasses ────────────────────────────
    // Carex praegracilis at 12" center-to-center spacing (1.0 ft), $8/plant
    // price_per_sqft = $8.00 / 1.0² = $8.00/sqft
    let better_plants = Material::builder(
        "Carex praegracilis (12\" spacing)",
        MaterialCategory::Softscape,
    )
    .unit(Unit::SqFt)
    .price_per_unit(Decimal::from_str("8.00").expect("decimal"))
    .extrusion(ExtrusionBehavior::SitsOnTop {
        height_inches: 12.0,
    })
    .build();
    let better_soil = Material::builder("Planting soil", MaterialCategory::Fill)
        .unit(Unit::CuYd)
        .price_per_unit(Decimal::from_str("45.00").expect("decimal"))
        .depth_inches(6.0)
        .extrusion(ExtrusionBehavior::Fills { flush: true })
        .build();

    // ── Best: Seasonal Color Display ──────────────────────────
    // Mixed annuals at 6" center-to-center spacing (0.5 ft), $3/plant
    // price_per_sqft = 3.0 / (0.5 × 0.5) = $12.00/sqft
    let spacing_best = 6.0 / 12.0;
    let best_plants = Material::builder("Mixed annuals (6\" spacing)", MaterialCategory::Softscape)
        .unit(Unit::SqFt)
        .price_per_unit(
            Decimal::from_str("3.00").expect("decimal") / decimal_f64(spacing_best * spacing_best),
        )
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 8.0 })
        .build();
    let best_soil = Material::builder("Planting soil", MaterialCategory::Fill)
        .unit(Unit::CuYd)
        .price_per_unit(Decimal::from_str("45.00").expect("decimal"))
        .depth_inches(6.0)
        .extrusion(ExtrusionBehavior::Fills { flush: true })
        .build();
    let best_edging = Material::builder("Steel edging", MaterialCategory::Edging)
        .unit(Unit::LinearFt)
        .price_per_unit(Decimal::from_str("3.25").expect("decimal"))
        .extrusion(ExtrusionBehavior::BuildsUp { height_inches: 4.0 })
        .build();

    (
        vec![good_plants, good_soil],
        vec![better_plants, better_soil],
        vec![best_plants, best_soil, best_edging],
    )
}

/// Build a tier that assigns all materials to the given zone.
fn build_tier(level: TierLevel, zone: &Zone, materials: &[Material]) -> Tier {
    Tier {
        level,
        assignments: materials
            .iter()
            .map(|m| MaterialAssignment {
                zone_id: zone.id,
                material_id: m.id,
                overrides: None,
            })
            .collect(),
    }
}

/// Print a single tier's quote in the investor-ready format.
fn print_tier_quote(tier_label: &str, style_name: &str, quote: &pt_quote::Quote, area_sqft: f64) {
    println!("{tier_label} — {style_name}");

    for li in &quote.line_items {
        match li.unit {
            Unit::SqFt => {
                // Display as plant count: area / spacing² (spacing² = price_per_plant / unit_price)
                // We know unit_price = price_per_plant / spacing², and line_total = area × unit_price
                // Plant count = area / spacing² — but we don't have spacing directly.
                // Instead: line_total / price_per_plant. But we don't have price_per_plant either.
                // Simplest: display the sqft quantity and line total.
                // Actually, compute approximate plant count from area and known spacings.
                let plant_count = plant_count_from_name(&li.material_name, area_sqft);
                println!(
                    "  {:>3} × {:<36} ${:.2}",
                    plant_count, li.material_name, li.line_total,
                );
            }
            Unit::CuYd => {
                println!(
                    "  {:.1} cu yd {:<33} ${:.2}",
                    li.quantity, li.material_name, li.line_total,
                );
            }
            Unit::LinearFt => {
                println!(
                    "  {}, {:.0} linear ft{} ${:.2}",
                    li.material_name,
                    li.quantity,
                    " ".repeat(21_usize.saturating_sub(li.material_name.len())),
                    li.line_total,
                );
            }
            Unit::Each => {
                println!(
                    "  {} × {:<36} ${:.2}",
                    li.quantity, li.material_name, li.line_total,
                );
            }
        }
    }

    println!("  {:<42} ${:.2}", "Total:", quote.total);
    println!();
}

/// Derive plant count from material name and area.
///
/// Parses spacing from the material name (e.g. `4" spacing` → 4 inches)
/// and computes: area_sqft / (spacing_ft × spacing_ft).
fn plant_count_from_name(name: &str, area_sqft: f64) -> u32 {
    // Look for pattern: N" spacing
    let spacing_inches = if let Some(pos) = name.find("\" spacing") {
        // Walk backwards to find the number
        let before = &name[..pos];
        let start = before.rfind('(').map(|p| p + 1).unwrap_or(0);
        before[start..].trim().parse::<f64>().unwrap_or(6.0)
    } else {
        6.0 // default 6" spacing
    };

    let spacing_ft = spacing_inches / 12.0;
    let count = area_sqft / (spacing_ft * spacing_ft);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let result = count.round().max(0.0) as u32;
    result
}

/// Convert an f64 to Decimal for arithmetic.
fn decimal_f64(v: f64) -> Decimal {
    Decimal::from_str(&format!("{v:.6}")).unwrap_or(Decimal::ONE)
}
