use crate::registry::{Integration, Scenario, ScenarioOutcome, ValueArea};

pub fn scenarios() -> &'static [Scenario] {
    &SCENARIOS
}

static SCENARIOS: [Scenario; 4] = [
    Scenario {
        id: "S.1.1",
        name: "Scan processing",
        area: ValueArea::SiteAssessment,
        time_savings_minutes: 30.0,
        replaces: "Manual site measurement with tape measure, graph paper, return visits",
        test_fn: s_1_1_scan_processing,
    },
    Scenario {
        id: "S.1.2",
        name: "Satellite pre-population",
        area: ValueArea::SiteAssessment,
        time_savings_minutes: 25.0,
        replaces: "Cold-start research on every new site (lot boundaries, existing features)",
        test_fn: s_1_2_satellite_prepopulation,
    },
    Scenario {
        id: "S.1.3",
        name: "Sun exposure analysis",
        area: ValueArea::SiteAssessment,
        time_savings_minutes: 20.0,
        replaces: "Guessing sun patterns or multiple site visits at different times of day",
        test_fn: s_1_3_sun_exposure_analysis,
    },
    Scenario {
        id: "S.1.4",
        name: "Plant identification",
        area: ValueArea::SiteAssessment,
        time_savings_minutes: 15.0,
        replaces: "Manual plant ID during site visit or follow-up research",
        test_fn: s_1_4_plant_identification,
    },
];

/// S.1.1 — Scan processing
///
/// Validates: PLY file → classified point cloud → terrain mesh + exports.
/// T-015-01 delivered parsing + filtering. T-015-02 adds mesh gen + export.
/// OneStar: full scan-to-artifact pipeline works as pure computation.
fn s_1_1_scan_processing() -> ScenarioOutcome {
    use pt_scan::{ExportConfig, ScanConfig};
    use std::io::Cursor;

    // Build a synthetic PLY: 500 ground (z≈0), 100 obstacles (z=0.5), 30 outliers (z=20)
    let ply_data = {
        let total = 630;
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

        // 500 ground points
        for i in 0..500_u32 {
            let x = (i % 25) as f32 * 0.4;
            let y = (i / 25) as f32 * 0.4;
            let z = if i % 2 == 0 { 0.002_f32 } else { -0.002 };
            buf.extend_from_slice(&x.to_le_bytes());
            buf.extend_from_slice(&y.to_le_bytes());
            buf.extend_from_slice(&z.to_le_bytes());
            buf.extend_from_slice(&[0, 128, 0]);
        }
        // 100 obstacle points
        for i in 0..100_u32 {
            let x = 2.0 + (i % 10) as f32 * 0.1;
            let y = 2.0 + (i / 10) as f32 * 0.1;
            let z = 0.5_f32;
            buf.extend_from_slice(&x.to_le_bytes());
            buf.extend_from_slice(&y.to_le_bytes());
            buf.extend_from_slice(&z.to_le_bytes());
            buf.extend_from_slice(&[128, 0, 0]);
        }
        // 30 outlier points
        for i in 0..30_u32 {
            let v = 20.0 + i as f32;
            buf.extend_from_slice(&v.to_le_bytes());
            buf.extend_from_slice(&v.to_le_bytes());
            buf.extend_from_slice(&v.to_le_bytes());
            buf.extend_from_slice(&[255, 255, 255]);
        }
        buf
    };

    let config = ScanConfig {
        voxel_size: 0.5,
        outlier_k: 10,
        outlier_threshold: 2.0,
        ransac_iterations: 500,
        ransac_threshold: 0.05,
    };

    let cloud = match pt_scan::process_scan(Cursor::new(ply_data), &config) {
        Ok(c) => c,
        Err(e) => return ScenarioOutcome::Fail(format!("process_scan failed: {e}")),
    };

    // 1. Verify original count
    // 500 + 100 + 30 = 630 — computed independently
    if cloud.metadata.original_count != 630 {
        return ScenarioOutcome::Fail(format!(
            "original_count {} != 630",
            cloud.metadata.original_count
        ));
    }

    // 2. No outliers (z > 10) should survive
    for p in cloud.ground.iter().chain(cloud.obstacles.iter()) {
        if p.position[2] > 10.0 {
            return ScenarioOutcome::Fail(format!("outlier leaked through: z={}", p.position[2]));
        }
    }

    // 3. Ground points should have z near 0
    if cloud.ground.is_empty() {
        return ScenarioOutcome::Fail("no ground points classified".to_string());
    }
    for p in &cloud.ground {
        if p.position[2].abs() > 0.1 {
            return ScenarioOutcome::Fail(format!(
                "ground point z={} too far from 0",
                p.position[2]
            ));
        }
    }

    // 4. Should have obstacle points
    if cloud.obstacles.is_empty() {
        return ScenarioOutcome::Fail("no obstacle points classified".to_string());
    }

    // 5. Metadata consistency
    if cloud.metadata.ground_count != cloud.ground.len() {
        return ScenarioOutcome::Fail("ground_count mismatch".to_string());
    }
    if cloud.metadata.obstacle_count != cloud.obstacles.len() {
        return ScenarioOutcome::Fail("obstacle_count mismatch".to_string());
    }

    // 6. Ground plane normal should be roughly vertical (z component dominant)
    let nz = cloud.metadata.ground_plane.normal[2].abs();
    if nz < 0.9 {
        return ScenarioOutcome::Fail(format!(
            "ground plane normal z={nz}, expected near-vertical"
        ));
    }

    // 7. Mesh generation + export (T-015-02)
    let export_config = ExportConfig::default();
    let output = match pt_scan::generate_terrain(&cloud, &export_config) {
        Ok(o) => o,
        Err(e) => return ScenarioOutcome::Fail(format!("generate_terrain failed: {e}")),
    };

    // 8. GLB starts with glTF magic (0x46546C67)
    if output.mesh_glb.len() < 4 {
        return ScenarioOutcome::Fail("GLB output too short".to_string());
    }
    let glb_magic = u32::from_le_bytes([
        output.mesh_glb[0],
        output.mesh_glb[1],
        output.mesh_glb[2],
        output.mesh_glb[3],
    ]);
    if glb_magic != 0x4654_6C67 {
        return ScenarioOutcome::Fail(format!("GLB magic {glb_magic:#x} != 0x46546C67"));
    }

    // 9. PNG starts with PNG signature
    if output.plan_view_png.len() < 4 || &output.plan_view_png[1..4] != b"PNG" {
        return ScenarioOutcome::Fail("plan view PNG signature invalid".to_string());
    }

    // 10. Metadata sanity
    if output.metadata.decimated_triangle_count == 0 {
        return ScenarioOutcome::Fail("no triangles in decimated mesh".to_string());
    }
    if output.metadata.vertex_count == 0 {
        return ScenarioOutcome::Fail("no vertices in decimated mesh".to_string());
    }

    // OneStar: full scan-to-artifact pipeline works as pure computation.
    // No API or UI integration yet — needs T-016-01 (upload API) for TwoStar.
    ScenarioOutcome::Pass(Integration::OneStar)
}

/// S.1.2 — Satellite pre-population
///
/// Given an address, produce a project baseline containing lot boundary,
/// detected trees, and sun exposure grid. Verifies the complete pipeline
/// from address string to structured site data.
fn s_1_2_satellite_prepopulation() -> ScenarioOutcome {
    use pt_satellite::{BaselineBuilder, EmbeddedSource};

    let source = EmbeddedSource;
    let builder = BaselineBuilder::new(source.clone(), source.clone(), source);

    let baseline = match builder.build("1234 Noriega St, San Francisco, CA") {
        Ok(b) => b,
        Err(e) => return ScenarioOutcome::Fail(format!("build failed: {e}")),
    };

    // 1. Coordinates should be in SF Inner Sunset (~37.76, ~-122.49)
    if (baseline.coordinates.latitude - 37.76).abs() > 0.01 {
        return ScenarioOutcome::Fail(format!(
            "latitude {} too far from expected 37.76",
            baseline.coordinates.latitude
        ));
    }
    if (baseline.coordinates.longitude - (-122.486)).abs() > 0.01 {
        return ScenarioOutcome::Fail(format!(
            "longitude {} too far from expected -122.486",
            baseline.coordinates.longitude
        ));
    }

    // 2. Lot boundary: typical SF residential lot is 2,500–10,000 sqft
    if baseline.lot_boundary.area_sqft < 2_000.0 || baseline.lot_boundary.area_sqft > 15_000.0 {
        return ScenarioOutcome::Fail(format!(
            "lot area {} sqft outside plausible range 2,000–15,000",
            baseline.lot_boundary.area_sqft
        ));
    }

    // Polygon should have at least 4 vertices (+ closing point)
    let vertex_count = baseline.lot_boundary.polygon.exterior().0.len();
    if vertex_count < 4 {
        return ScenarioOutcome::Fail(format!(
            "lot polygon has only {vertex_count} vertices, expected ≥4"
        ));
    }

    // 3. Trees: expect 1–10 trees with plausible dimensions
    if baseline.trees.is_empty() {
        return ScenarioOutcome::Fail("no trees detected".to_string());
    }
    if baseline.trees.len() > 10 {
        return ScenarioOutcome::Fail(format!(
            "too many trees: {} (expected ≤10 for residential lot)",
            baseline.trees.len()
        ));
    }
    for (i, tree) in baseline.trees.iter().enumerate() {
        if tree.height_ft < 5.0 || tree.height_ft > 100.0 {
            return ScenarioOutcome::Fail(format!(
                "tree {i} height {} ft outside plausible range 5–100",
                tree.height_ft
            ));
        }
        if tree.spread_ft < 3.0 || tree.spread_ft > 60.0 {
            return ScenarioOutcome::Fail(format!(
                "tree {i} spread {} ft outside plausible range 3–60",
                tree.spread_ft
            ));
        }
        if tree.confidence <= 0.0 || tree.confidence > 1.0 {
            return ScenarioOutcome::Fail(format!(
                "tree {i} confidence {} outside (0, 1]",
                tree.confidence
            ));
        }
    }

    // 4. Sun grid: should have cells with Bay Area growing-season sun hours
    if baseline.sun_grid.values.is_empty() {
        return ScenarioOutcome::Fail("sun grid has no cells".to_string());
    }
    let expected_cells = (baseline.sun_grid.width * baseline.sun_grid.height) as usize;
    if baseline.sun_grid.values.len() != expected_cells {
        return ScenarioOutcome::Fail(format!(
            "sun grid values.len()={} != width*height={}",
            baseline.sun_grid.values.len(),
            expected_cells
        ));
    }
    for (i, &val) in baseline.sun_grid.values.iter().enumerate() {
        let h = f64::from(val);
        if !(6.0..=18.0).contains(&h) {
            return ScenarioOutcome::Fail(format!(
                "sun grid cell {i} hours {h:.1} outside expected range 6–18"
            ));
        }
    }

    // 5. TwoStar: verify JSON round-trip (proves baseline survives JSONB storage)
    let json_val = match serde_json::to_value(&baseline) {
        Ok(v) => v,
        Err(e) => return ScenarioOutcome::Fail(format!("baseline serialization failed: {e}")),
    };
    let round_tripped: pt_satellite::ProjectBaseline = match serde_json::from_value(json_val) {
        Ok(b) => b,
        Err(e) => return ScenarioOutcome::Fail(format!("baseline deserialization failed: {e}")),
    };

    // Verify key fields survive the round-trip
    if (round_tripped.coordinates.latitude - baseline.coordinates.latitude).abs() > f64::EPSILON {
        return ScenarioOutcome::Fail("coordinates.latitude changed after round-trip".to_string());
    }
    if (round_tripped.lot_boundary.area_sqft - baseline.lot_boundary.area_sqft).abs() > f64::EPSILON
    {
        return ScenarioOutcome::Fail(
            "lot_boundary.area_sqft changed after round-trip".to_string(),
        );
    }
    if round_tripped.trees.len() != baseline.trees.len() {
        return ScenarioOutcome::Fail(format!(
            "tree count changed after round-trip: {} → {}",
            baseline.trees.len(),
            round_tripped.trees.len()
        ));
    }
    if round_tripped.sun_grid.values.len() != baseline.sun_grid.values.len() {
        return ScenarioOutcome::Fail(format!(
            "sun grid cell count changed after round-trip: {} → {}",
            baseline.sun_grid.values.len(),
            round_tripped.sun_grid.values.len()
        ));
    }

    ScenarioOutcome::Pass(Integration::TwoStar)
}

/// S.1.3 — Sun exposure analysis
///
/// Given a location and date range, compute a radiance grid and verify
/// that every cell has a valid light classification and sun hours fall
/// within expected ranges for the SF Bay Area growing season.
fn s_1_3_sun_exposure_analysis() -> ScenarioOutcome {
    use chrono::NaiveDate;
    use pt_solar::{
        classify, radiance_grid, GridConfig, LatLngBounds, LightCategory, METERS_PER_DEGREE_LAT,
    };

    // SF Bay Area: ~200m × 200m residential lot
    let center_lat: f64 = 37.7749;
    let center_lng: f64 = -122.4194;
    let offset_lat = 100.0 / METERS_PER_DEGREE_LAT;
    let lng_scale = METERS_PER_DEGREE_LAT * center_lat.to_radians().cos();
    let offset_lng = 100.0 / lng_scale;

    let bounds = LatLngBounds {
        south: center_lat - offset_lat,
        west: center_lng - offset_lng,
        north: center_lat + offset_lat,
        east: center_lng + offset_lng,
    };

    // Growing season: March through September
    let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();

    let config = GridConfig {
        resolution_meters: 50.0,
        sample_days: 6,
    };

    let grid = radiance_grid(&bounds, (start, end), &config);

    // Grid should have cells
    if grid.values.is_empty() {
        return ScenarioOutcome::Fail("grid has no cells".to_string());
    }

    // Every cell should have reasonable Bay Area growing-season sun hours (8-18h average)
    // and a valid light classification
    for (i, &sun_hours) in grid.values.iter().enumerate() {
        let h = f64::from(sun_hours);
        if !(8.0..=18.0).contains(&h) {
            return ScenarioOutcome::Fail(format!(
                "cell {i} sun hours {h:.1} outside expected range 8-18"
            ));
        }

        // Verify classification is deterministic and matches thresholds
        let cat = classify(h);
        match cat {
            LightCategory::FullSun => {
                if h < 6.0 {
                    return ScenarioOutcome::Fail(format!(
                        "cell {i}: classified FullSun but hours={h:.1}"
                    ));
                }
            }
            LightCategory::PartSun => {
                if !(4.0..6.0).contains(&h) {
                    return ScenarioOutcome::Fail(format!(
                        "cell {i}: classified PartSun but hours={h:.1}"
                    ));
                }
            }
            _ => {
                // Bay Area growing season should not produce PartShade or FullShade
                // for unobstructed theoretical sun hours
                return ScenarioOutcome::Fail(format!(
                    "cell {i}: unexpected category {:?} for hours={h:.1}",
                    cat
                ));
            }
        }
    }

    // Grid dimensions should be consistent with values length
    if grid.values.len() != (grid.width * grid.height) as usize {
        return ScenarioOutcome::Fail(format!(
            "values.len()={} != width*height={}",
            grid.values.len(),
            grid.width * grid.height
        ));
    }

    ScenarioOutcome::Pass(Integration::OneStar)
}

fn s_1_4_plant_identification() -> ScenarioOutcome {
    // Validates: photo → species ID → mapped to platform plant database
    // Requires: pt-plants crate + Plant.id integration
    ScenarioOutcome::NotImplemented
}
