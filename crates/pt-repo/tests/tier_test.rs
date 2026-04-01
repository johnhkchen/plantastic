#![cfg(feature = "integration")]
//! Integration tests for the tier assignment repository.

use geo::polygon;
use pt_materials::{ExtrusionBehavior, MaterialCategory, Unit};
use pt_project::{TierLevel, ZoneType};
use pt_repo::material::CreateMaterial;
use pt_repo::project::CreateProject;
use pt_repo::tier_assignment::{self, SetAssignment};
use pt_repo::zone::CreateZone;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;

/// Create a full fixture: tenant -> project -> zone -> material.
/// Returns (project_id, zone_id, material_id).
async fn create_fixture(pool: &PgPool) -> (uuid::Uuid, uuid::Uuid, uuid::Uuid) {
    let tenant_id = pt_repo::tenant::create(pool, "Tier Test Co").await.unwrap();

    let project_id = pt_repo::project::create(
        pool,
        &CreateProject {
            tenant_id,
            client_name: Some("Tier Tester".into()),
            client_email: None,
            address: None,
        },
    )
    .await
    .unwrap();

    let poly = polygon![
        (x: -122.42, y: 37.77),
        (x: -122.41, y: 37.77),
        (x: -122.41, y: 37.78),
        (x: -122.42, y: 37.78),
    ];

    let zone_id = pt_repo::zone::add(
        pool,
        &CreateZone {
            project_id,
            geometry: poly,
            zone_type: ZoneType::Patio,
            label: Some("Main Patio".into()),
            sort_order: 0,
        },
    )
    .await
    .unwrap();

    let material_id = pt_repo::material::create(
        pool,
        &CreateMaterial {
            tenant_id,
            name: "Standard Pavers".into(),
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

    (project_id, zone_id, material_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_and_get_assignments(pool: PgPool) {
    let (project_id, zone_id, material_id) = create_fixture(&pool).await;

    let assignments = vec![SetAssignment {
        zone_id,
        material_id,
        overrides: None,
    }];

    tier_assignment::set_assignments(&pool, project_id, TierLevel::Good, &assignments)
        .await
        .unwrap();

    let rows = tier_assignment::get_by_project_and_tier(&pool, project_id, TierLevel::Good)
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].zone_id, zone_id);
    assert_eq!(rows[0].material_id, material_id);
    assert_eq!(rows[0].tier, TierLevel::Good);
    assert!(rows[0].overrides.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_assignments_replaces_existing(pool: PgPool) {
    let (project_id, zone_id, material_id) = create_fixture(&pool).await;

    // Set initial assignment
    let initial = vec![SetAssignment {
        zone_id,
        material_id,
        overrides: None,
    }];
    tier_assignment::set_assignments(&pool, project_id, TierLevel::Better, &initial)
        .await
        .unwrap();

    // Create a second material for replacement
    let tenant_id = pt_repo::project::get_by_id(&pool, project_id)
        .await
        .unwrap()
        .tenant_id;

    let material_id_2 = pt_repo::material::create(
        &pool,
        &CreateMaterial {
            tenant_id,
            name: "Premium Pavers".into(),
            category: MaterialCategory::Hardscape,
            unit: Unit::SqFt,
            price_per_unit: Decimal::from_str("15.00").unwrap(),
            depth_inches: None,
            extrusion: ExtrusionBehavior::SitsOnTop { height_inches: 1.5 },
            texture_key: None,
            photo_key: None,
            supplier_sku: None,
        },
    )
    .await
    .unwrap();

    // Replace with new assignment
    let replacement = vec![SetAssignment {
        zone_id,
        material_id: material_id_2,
        overrides: Some(serde_json::json!({"price_override": "20.00"})),
    }];
    tier_assignment::set_assignments(&pool, project_id, TierLevel::Better, &replacement)
        .await
        .unwrap();

    let rows = tier_assignment::get_by_project_and_tier(&pool, project_id, TierLevel::Better)
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].material_id, material_id_2);
    assert!(rows[0].overrides.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn tiers_are_independent(pool: PgPool) {
    let (project_id, zone_id, material_id) = create_fixture(&pool).await;

    let assignments = vec![SetAssignment {
        zone_id,
        material_id,
        overrides: None,
    }];

    // Set Good tier
    tier_assignment::set_assignments(&pool, project_id, TierLevel::Good, &assignments)
        .await
        .unwrap();

    // Best tier should be empty
    let best = tier_assignment::get_by_project_and_tier(&pool, project_id, TierLevel::Best)
        .await
        .unwrap();
    assert!(best.is_empty());

    // Good tier should have one
    let good = tier_assignment::get_by_project_and_tier(&pool, project_id, TierLevel::Good)
        .await
        .unwrap();
    assert_eq!(good.len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn empty_assignments_clears_tier(pool: PgPool) {
    let (project_id, zone_id, material_id) = create_fixture(&pool).await;

    // Set an assignment
    let assignments = vec![SetAssignment {
        zone_id,
        material_id,
        overrides: None,
    }];
    tier_assignment::set_assignments(&pool, project_id, TierLevel::Good, &assignments)
        .await
        .unwrap();

    // Clear by setting empty
    tier_assignment::set_assignments(&pool, project_id, TierLevel::Good, &[])
        .await
        .unwrap();

    let rows = tier_assignment::get_by_project_and_tier(&pool, project_id, TierLevel::Good)
        .await
        .unwrap();
    assert!(rows.is_empty());
}
