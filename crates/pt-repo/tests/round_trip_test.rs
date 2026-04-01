#![cfg(feature = "integration")]
//! Full round-trip integration test: create project with zones, assign materials
//! to tiers, fetch everything, verify geometry matches.

use geo::polygon;
use pt_materials::{ExtrusionBehavior, MaterialCategory, Unit};
use pt_project::{TierLevel, ZoneType};
use pt_repo::material::CreateMaterial;
use pt_repo::project::CreateProject;
use pt_repo::tier_assignment::{self, SetAssignment};
use pt_repo::zone::{self, CreateZone};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test(migrations = "../../migrations")]
async fn full_project_round_trip(pool: PgPool) {
    // 1. Create tenant
    let tenant_id = pt_repo::tenant::create(&pool, "Round Trip Co")
        .await
        .unwrap();

    // 2. Create project
    let project_id = pt_repo::project::create(
        &pool,
        &CreateProject {
            tenant_id,
            client_name: Some("Alice".into()),
            client_email: Some("alice@example.com".into()),
            address: Some("123 Main St, San Francisco, CA".into()),
        },
    )
    .await
    .unwrap();

    // 3. Add zones with known polygons
    let patio_poly = polygon![
        (x: -122.4194, y: 37.7749),
        (x: -122.4180, y: 37.7749),
        (x: -122.4180, y: 37.7760),
        (x: -122.4194, y: 37.7760),
    ];
    let bed_poly = polygon![
        (x: -122.4170, y: 37.7740),
        (x: -122.4155, y: 37.7740),
        (x: -122.4155, y: 37.7755),
        (x: -122.4170, y: 37.7755),
    ];
    let edging_poly = polygon![
        (x: -122.4150, y: 37.7730),
        (x: -122.4140, y: 37.7730),
        (x: -122.4140, y: 37.7740),
        (x: -122.4150, y: 37.7740),
    ];

    let zone_ids = zone::bulk_upsert(
        &pool,
        project_id,
        &[
            CreateZone {
                project_id,
                geometry: patio_poly.clone(),
                zone_type: ZoneType::Patio,
                label: Some("Front Patio".into()),
                sort_order: 0,
            },
            CreateZone {
                project_id,
                geometry: bed_poly.clone(),
                zone_type: ZoneType::Bed,
                label: Some("Flower Bed".into()),
                sort_order: 1,
            },
            CreateZone {
                project_id,
                geometry: edging_poly.clone(),
                zone_type: ZoneType::Edging,
                label: Some("Border Edging".into()),
                sort_order: 2,
            },
        ],
    )
    .await
    .unwrap();
    assert_eq!(zone_ids.len(), 3);

    // 4. Create materials (Good tier)
    let pavers_id = pt_repo::material::create(
        &pool,
        &CreateMaterial {
            tenant_id,
            name: "Travertine Pavers".into(),
            category: MaterialCategory::Hardscape,
            unit: Unit::SqFt,
            price_per_unit: Decimal::from_str("8.50").unwrap(),
            depth_inches: None,
            extrusion: ExtrusionBehavior::SitsOnTop { height_inches: 1.0 },
            texture_key: None,
            photo_key: None,
            supplier_sku: None,
        },
    )
    .await
    .unwrap();

    let mulch_id = pt_repo::material::create(
        &pool,
        &CreateMaterial {
            tenant_id,
            name: "Bark Mulch".into(),
            category: MaterialCategory::Softscape,
            unit: Unit::CuYd,
            price_per_unit: Decimal::from_str("45.00").unwrap(),
            depth_inches: Some(4.0),
            extrusion: ExtrusionBehavior::Fills { flush: true },
            texture_key: None,
            photo_key: None,
            supplier_sku: None,
        },
    )
    .await
    .unwrap();

    let edging_mat_id = pt_repo::material::create(
        &pool,
        &CreateMaterial {
            tenant_id,
            name: "Steel Edging".into(),
            category: MaterialCategory::Edging,
            unit: Unit::LinearFt,
            price_per_unit: Decimal::from_str("3.25").unwrap(),
            depth_inches: None,
            extrusion: ExtrusionBehavior::BuildsUp { height_inches: 4.0 },
            texture_key: None,
            photo_key: None,
            supplier_sku: None,
        },
    )
    .await
    .unwrap();

    // 5. Assign materials to Good tier
    tier_assignment::set_assignments(
        &pool,
        project_id,
        TierLevel::Good,
        &[
            SetAssignment {
                zone_id: zone_ids[0],
                material_id: pavers_id,
                overrides: None,
            },
            SetAssignment {
                zone_id: zone_ids[1],
                material_id: mulch_id,
                overrides: None,
            },
            SetAssignment {
                zone_id: zone_ids[2],
                material_id: edging_mat_id,
                overrides: None,
            },
        ],
    )
    .await
    .unwrap();

    // 6. Fetch everything back and verify

    // Project
    let project = pt_repo::project::get_by_id(&pool, project_id)
        .await
        .unwrap();
    assert_eq!(project.client_name.as_deref(), Some("Alice"));
    assert_eq!(project.tenant_id, tenant_id);

    // Zones — verify geometry coordinates match
    let zones = zone::list_by_project(&pool, project_id).await.unwrap();
    assert_eq!(zones.len(), 3);

    let original_polys = [&patio_poly, &bed_poly, &edging_poly];
    for (zone, orig) in zones.iter().zip(original_polys.iter()) {
        let orig_pts: Vec<_> = orig.exterior().points().collect();
        let back_pts: Vec<_> = zone.geometry.exterior().points().collect();
        assert_eq!(orig_pts.len(), back_pts.len());
        for (a, b) in orig_pts.iter().zip(back_pts.iter()) {
            assert!(
                (a.x() - b.x()).abs() < 1e-10,
                "x mismatch in zone {:?}: {} vs {}",
                zone.label,
                a.x(),
                b.x()
            );
            assert!(
                (a.y() - b.y()).abs() < 1e-10,
                "y mismatch in zone {:?}: {} vs {}",
                zone.label,
                a.y(),
                b.y()
            );
        }
    }

    // Zone types
    assert_eq!(zones[0].zone_type, ZoneType::Patio);
    assert_eq!(zones[1].zone_type, ZoneType::Bed);
    assert_eq!(zones[2].zone_type, ZoneType::Edging);

    // Tier assignments
    let good_assignments =
        tier_assignment::get_by_project_and_tier(&pool, project_id, TierLevel::Good)
            .await
            .unwrap();
    assert_eq!(good_assignments.len(), 3);

    // Verify each zone has the right material
    let patio_assignment = good_assignments
        .iter()
        .find(|a| a.zone_id == zone_ids[0])
        .expect("patio assignment missing");
    assert_eq!(patio_assignment.material_id, pavers_id);

    let bed_assignment = good_assignments
        .iter()
        .find(|a| a.zone_id == zone_ids[1])
        .expect("bed assignment missing");
    assert_eq!(bed_assignment.material_id, mulch_id);

    let edging_assignment = good_assignments
        .iter()
        .find(|a| a.zone_id == zone_ids[2])
        .expect("edging assignment missing");
    assert_eq!(edging_assignment.material_id, edging_mat_id);

    // Materials
    let mats = pt_repo::material::list_by_tenant(&pool, tenant_id)
        .await
        .unwrap();
    assert_eq!(mats.len(), 3);

    let mulch = mats.iter().find(|m| m.id == mulch_id).unwrap();
    assert_eq!(mulch.unit, Unit::CuYd);
    assert!(
        (mulch.depth_inches.unwrap() - 4.0).abs() < 1e-10,
        "mulch depth should be 4.0"
    );
    assert_eq!(mulch.extrusion, ExtrusionBehavior::Fills { flush: true });

    // 7. Verify other tiers are empty
    let better = tier_assignment::get_by_project_and_tier(&pool, project_id, TierLevel::Better)
        .await
        .unwrap();
    assert!(better.is_empty());
    let best = tier_assignment::get_by_project_and_tier(&pool, project_id, TierLevel::Best)
        .await
        .unwrap();
    assert!(best.is_empty());
}
