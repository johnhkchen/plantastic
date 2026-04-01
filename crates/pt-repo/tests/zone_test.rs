#![cfg(feature = "integration")]
//! Integration tests for the zone repository.

use geo::polygon;
use pt_project::ZoneType;
use pt_repo::zone::{self, CreateZone};
use sqlx::PgPool;

async fn create_project(pool: &PgPool) -> uuid::Uuid {
    let tenant_id = pt_repo::tenant::create(pool, "Zone Test Co").await.unwrap();
    pt_repo::project::create(
        pool,
        &pt_repo::project::CreateProject {
            tenant_id,
            client_name: Some("Zone Tester".into()),
            client_email: None,
            address: None,
        },
    )
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn add_and_list_zones(pool: PgPool) {
    let project_id = create_project(&pool).await;

    let poly = polygon![
        (x: -122.42, y: 37.77),
        (x: -122.41, y: 37.77),
        (x: -122.41, y: 37.78),
        (x: -122.42, y: 37.78),
    ];

    let id = zone::add(
        &pool,
        &CreateZone {
            project_id,
            geometry: poly.clone(),
            zone_type: ZoneType::Patio,
            label: Some("Front Patio".into()),
            sort_order: 0,
        },
    )
    .await
    .unwrap();

    let zones = zone::list_by_project(&pool, project_id).await.unwrap();
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].id, id);
    assert_eq!(zones[0].zone_type, ZoneType::Patio);
    assert_eq!(zones[0].label.as_deref(), Some("Front Patio"));

    // Verify geometry coordinates round-trip through PostGIS
    let orig_pts: Vec<_> = poly.exterior().points().collect();
    let back_pts: Vec<_> = zones[0].geometry.exterior().points().collect();
    assert_eq!(orig_pts.len(), back_pts.len());
    for (a, b) in orig_pts.iter().zip(back_pts.iter()) {
        assert!(
            (a.x() - b.x()).abs() < 1e-10,
            "x mismatch: {} vs {}",
            a.x(),
            b.x()
        );
        assert!(
            (a.y() - b.y()).abs() < 1e-10,
            "y mismatch: {} vs {}",
            a.y(),
            b.y()
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_zone(pool: PgPool) {
    let project_id = create_project(&pool).await;

    let poly = polygon![
        (x: -122.42, y: 37.77),
        (x: -122.41, y: 37.77),
        (x: -122.41, y: 37.78),
        (x: -122.42, y: 37.78),
    ];

    let id = zone::add(
        &pool,
        &CreateZone {
            project_id,
            geometry: poly,
            zone_type: ZoneType::Bed,
            label: None,
            sort_order: 0,
        },
    )
    .await
    .unwrap();

    let new_poly = polygon![
        (x: -122.43, y: 37.76),
        (x: -122.42, y: 37.76),
        (x: -122.42, y: 37.77),
        (x: -122.43, y: 37.77),
    ];

    zone::update(&pool, id, &new_poly, ZoneType::Lawn, Some("Back Lawn"), 5)
        .await
        .unwrap();

    let zones = zone::list_by_project(&pool, project_id).await.unwrap();
    assert_eq!(zones[0].zone_type, ZoneType::Lawn);
    assert_eq!(zones[0].label.as_deref(), Some("Back Lawn"));
    assert_eq!(zones[0].sort_order, 5);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_zone(pool: PgPool) {
    let project_id = create_project(&pool).await;

    let poly = polygon![
        (x: -122.42, y: 37.77),
        (x: -122.41, y: 37.77),
        (x: -122.41, y: 37.78),
        (x: -122.42, y: 37.78),
    ];

    let id = zone::add(
        &pool,
        &CreateZone {
            project_id,
            geometry: poly,
            zone_type: ZoneType::Path,
            label: None,
            sort_order: 0,
        },
    )
    .await
    .unwrap();

    zone::delete(&pool, id).await.unwrap();
    let zones = zone::list_by_project(&pool, project_id).await.unwrap();
    assert!(zones.is_empty());

    // Delete non-existent returns NotFound
    let err = zone::delete(&pool, id).await.unwrap_err();
    assert!(matches!(err, pt_repo::RepoError::NotFound));
}

#[sqlx::test(migrations = "../../migrations")]
async fn bulk_upsert_replaces_zones(pool: PgPool) {
    let project_id = create_project(&pool).await;

    let poly1 = polygon![
        (x: -122.42, y: 37.77),
        (x: -122.41, y: 37.77),
        (x: -122.41, y: 37.78),
        (x: -122.42, y: 37.78),
    ];

    // Add an initial zone
    zone::add(
        &pool,
        &CreateZone {
            project_id,
            geometry: poly1,
            zone_type: ZoneType::Bed,
            label: Some("Old".into()),
            sort_order: 0,
        },
    )
    .await
    .unwrap();

    // Bulk upsert replaces with two new zones
    let poly2 = polygon![
        (x: -122.40, y: 37.75),
        (x: -122.39, y: 37.75),
        (x: -122.39, y: 37.76),
        (x: -122.40, y: 37.76),
    ];
    let poly3 = polygon![
        (x: -122.38, y: 37.74),
        (x: -122.37, y: 37.74),
        (x: -122.37, y: 37.75),
        (x: -122.38, y: 37.75),
    ];

    let new_zones = vec![
        CreateZone {
            project_id,
            geometry: poly2,
            zone_type: ZoneType::Patio,
            label: Some("New Patio".into()),
            sort_order: 0,
        },
        CreateZone {
            project_id,
            geometry: poly3,
            zone_type: ZoneType::Wall,
            label: Some("New Wall".into()),
            sort_order: 1,
        },
    ];

    let ids = zone::bulk_upsert(&pool, project_id, &new_zones)
        .await
        .unwrap();
    assert_eq!(ids.len(), 2);

    let zones = zone::list_by_project(&pool, project_id).await.unwrap();
    assert_eq!(zones.len(), 2);
    // Old "Bed" zone should be gone
    assert!(zones.iter().all(|z| z.zone_type != ZoneType::Bed));
}
