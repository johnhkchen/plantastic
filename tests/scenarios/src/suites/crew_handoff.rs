use crate::registry::{Integration, Polish, Scenario, ScenarioOutcome, ValueArea};

pub fn scenarios() -> &'static [Scenario] {
    &SCENARIOS
}

static SCENARIOS: [Scenario; 3] = [
    Scenario {
        id: "S.4.1",
        name: "3D viewer on tablet",
        area: ValueArea::CrewHandoff,
        time_savings_minutes: 15.0,
        replaces: "Verbal walkthrough of what goes where, printed sketches",
        test_fn: s_4_1_tablet_viewer,
    },
    Scenario {
        id: "S.4.2",
        name: "DXF export",
        area: ValueArea::CrewHandoff,
        time_savings_minutes: 10.0,
        replaces: "Redrawing the design in CAD for crew reference",
        test_fn: s_4_2_dxf_export,
    },
    Scenario {
        id: "S.4.3",
        name: "Material callouts with supplier info",
        area: ValueArea::CrewHandoff,
        time_savings_minutes: 5.0,
        replaces: "Separate material list with SKUs and install specs",
        test_fn: s_4_3_material_callouts,
    },
];

fn s_4_1_tablet_viewer() -> ScenarioOutcome {
    // Validates: approved project → glTF loads in viewer → zones tappable →
    //           material info + dimensions displayed → works on iPad Safari
    // Requires: pt-scene, Bevy viewer, full stack
    ScenarioOutcome::NotImplemented
}

fn s_4_2_dxf_export() -> ScenarioOutcome {
    // Validates: approved project → DXF bytes → DXF contains correct layers
    //           per zone type, LWPOLYLINE entities match zone geometry,
    //           dimension annotations present, material labels in TEXT entities
    // Requires: pt-dxf, pt-project
    ScenarioOutcome::NotImplemented
}

/// S.4.3 — Material callouts with supplier info
///
/// Validates that zone-material assignments carry the metadata a crew foreman
/// needs: material name, supplier SKU, install depth, product photo, and
/// extrusion behavior. Constructs materials with known callout fields, assigns
/// them to zones via a tier, then verifies each field is present and correct.
///
/// OneStar: pure domain-model test — no API or UI.
/// Path to TwoStar: verify callout data via GET /materials API response.
fn s_4_3_material_callouts() -> ScenarioOutcome {
    use geo::polygon;
    use pt_materials::{ExtrusionBehavior, Material, MaterialCategory, Unit};
    use pt_project::{MaterialAssignment, Tier, TierLevel, Zone, ZoneId, ZoneType};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // ── 1. Build materials with known callout fields ────────────

    let travertine = Material::builder("Travertine Pavers", MaterialCategory::Hardscape)
        .unit(Unit::SqFt)
        .price_per_unit(Decimal::from_str("8.50").unwrap())
        .depth_inches(1.0)
        .photo_ref("photos/trav.jpg")
        .supplier_sku("TRAV-12x12-NAT")
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 1.0 })
        .build();

    let mulch = Material::builder("Premium Mulch", MaterialCategory::Softscape)
        .unit(Unit::CuYd)
        .price_per_unit(Decimal::from_str("45.00").unwrap())
        .depth_inches(3.0)
        .supplier_sku("MULCH-PREM-BRN")
        .extrusion(ExtrusionBehavior::Fills { flush: true })
        .build();

    let edging = Material::builder("Steel Edging", MaterialCategory::Edging)
        .unit(Unit::LinearFt)
        .price_per_unit(Decimal::from_str("3.25").unwrap())
        .supplier_sku("EDGE-STL-4IN")
        .extrusion(ExtrusionBehavior::BuildsUp { height_inches: 4.0 })
        .build();

    let catalog = vec![travertine.clone(), mulch.clone(), edging.clone()];

    // ── 2. Build zones with known geometry ──────────────────────

    let patio_id = ZoneId::new();
    let bed_id = ZoneId::new();
    let edging_id = ZoneId::new();

    let _zones = [
        Zone {
            id: patio_id,
            geometry: polygon![
                (x: 0.0, y: 0.0),
                (x: 12.0, y: 0.0),
                (x: 12.0, y: 15.0),
                (x: 0.0, y: 15.0),
            ],
            zone_type: ZoneType::Patio,
            label: Some("Back patio".into()),
        },
        Zone {
            id: bed_id,
            geometry: polygon![
                (x: 0.0, y: 0.0),
                (x: 8.0, y: 0.0),
                (x: 8.0, y: 20.0),
                (x: 0.0, y: 20.0),
            ],
            zone_type: ZoneType::Bed,
            label: Some("Side bed".into()),
        },
        Zone {
            id: edging_id,
            geometry: polygon![
                (x: 0.0, y: 0.0),
                (x: 10.0, y: 0.0),
                (x: 10.0, y: 10.0),
                (x: 0.0, y: 10.0),
            ],
            zone_type: ZoneType::Edging,
            label: Some("Border edging".into()),
        },
    ];

    // ── 3. Build tier assignments ───────────────────────────────

    let tier = Tier {
        level: TierLevel::Good,
        assignments: vec![
            MaterialAssignment {
                zone_id: patio_id,
                material_id: travertine.id,
                overrides: None,
            },
            MaterialAssignment {
                zone_id: bed_id,
                material_id: mulch.id,
                overrides: None,
            },
            MaterialAssignment {
                zone_id: edging_id,
                material_id: edging.id,
                overrides: None,
            },
        ],
    };

    // ── 4. Verify callout data per zone-material pair ───────────
    //
    // Expected values are specified independently here — not extracted from
    // the Material objects. The test proves the builder sets the fields
    // correctly and they survive lookup by ID.

    struct ExpectedCallout {
        zone_id: ZoneId,
        material_name: &'static str,
        supplier_sku: Option<&'static str>,
        depth_inches: Option<f64>,
        photo_ref: Option<&'static str>,
        extrusion: ExtrusionBehavior,
    }

    let expected = vec![
        ExpectedCallout {
            zone_id: patio_id,
            material_name: "Travertine Pavers",
            supplier_sku: Some("TRAV-12x12-NAT"),
            depth_inches: Some(1.0),
            photo_ref: Some("photos/trav.jpg"),
            extrusion: ExtrusionBehavior::SitsOnTop { height_inches: 1.0 },
        },
        ExpectedCallout {
            zone_id: bed_id,
            material_name: "Premium Mulch",
            supplier_sku: Some("MULCH-PREM-BRN"),
            depth_inches: Some(3.0),
            photo_ref: None,
            extrusion: ExtrusionBehavior::Fills { flush: true },
        },
        ExpectedCallout {
            zone_id: edging_id,
            material_name: "Steel Edging",
            supplier_sku: Some("EDGE-STL-4IN"),
            depth_inches: None,
            photo_ref: None,
            extrusion: ExtrusionBehavior::BuildsUp { height_inches: 4.0 },
        },
    ];

    for exp in &expected {
        // Find the assignment for this zone.
        let assignment = match tier.assignments.iter().find(|a| a.zone_id == exp.zone_id) {
            Some(a) => a,
            None => {
                return ScenarioOutcome::Fail(format!(
                    "no assignment for zone {} (expected {})",
                    exp.zone_id, exp.material_name
                ))
            }
        };

        // Resolve material from catalog by ID.
        let material = match catalog.iter().find(|m| m.id == assignment.material_id) {
            Some(m) => m,
            None => {
                return ScenarioOutcome::Fail(format!(
                    "material {} not found in catalog for zone {}",
                    assignment.material_id, exp.zone_id
                ))
            }
        };

        // Verify each callout field independently.
        if material.name != exp.material_name {
            return ScenarioOutcome::Fail(format!(
                "zone {}: name '{}' != expected '{}'",
                exp.zone_id, material.name, exp.material_name
            ));
        }

        if material.supplier_sku.as_deref() != exp.supplier_sku {
            return ScenarioOutcome::Fail(format!(
                "zone {}: supplier_sku {:?} != expected {:?}",
                exp.zone_id, material.supplier_sku, exp.supplier_sku
            ));
        }

        if material.depth_inches != exp.depth_inches {
            return ScenarioOutcome::Fail(format!(
                "zone {}: depth_inches {:?} != expected {:?}",
                exp.zone_id, material.depth_inches, exp.depth_inches
            ));
        }

        if material.photo_ref.as_deref() != exp.photo_ref {
            return ScenarioOutcome::Fail(format!(
                "zone {}: photo_ref {:?} != expected {:?}",
                exp.zone_id, material.photo_ref, exp.photo_ref
            ));
        }

        if material.extrusion != exp.extrusion {
            return ScenarioOutcome::Fail(format!(
                "zone {}: extrusion {:?} != expected {:?}",
                exp.zone_id, material.extrusion, exp.extrusion
            ));
        }
    }

    // ── 5. JSON round-trip: callout fields survive serialization ─

    for material in &catalog {
        let json = match serde_json::to_value(material) {
            Ok(v) => v,
            Err(e) => {
                return ScenarioOutcome::Fail(format!(
                    "JSON serialize failed for {}: {e}",
                    material.name
                ))
            }
        };

        let back: Material = match serde_json::from_value(json.clone()) {
            Ok(m) => m,
            Err(e) => {
                return ScenarioOutcome::Fail(format!(
                    "JSON deserialize failed for {}: {e}",
                    material.name
                ))
            }
        };

        if back.supplier_sku != material.supplier_sku {
            return ScenarioOutcome::Fail(format!(
                "{}: supplier_sku lost in JSON round-trip",
                material.name
            ));
        }
        if back.depth_inches != material.depth_inches {
            return ScenarioOutcome::Fail(format!(
                "{}: depth_inches lost in JSON round-trip",
                material.name
            ));
        }
        if back.photo_ref != material.photo_ref {
            return ScenarioOutcome::Fail(format!(
                "{}: photo_ref lost in JSON round-trip",
                material.name
            ));
        }
        if back.extrusion != material.extrusion {
            return ScenarioOutcome::Fail(format!(
                "{}: extrusion lost in JSON round-trip",
                material.name
            ));
        }
    }

    // OneStar/OneStar: domain model carries all callout data a crew foreman
    // needs. Path to TwoStar: verify via GET /materials API response.
    ScenarioOutcome::Pass(Integration::OneStar, Polish::OneStar)
}
