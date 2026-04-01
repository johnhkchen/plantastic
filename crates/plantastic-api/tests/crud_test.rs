//! Integration tests for CRUD routes.
//!
//! All tests require a real PostgreSQL database with PostGIS.
//! Run with: DATABASE_URL=postgres://... cargo test -p plantastic-api -- --ignored

mod common;

use axum::http::{Method, StatusCode};
use serde_json::json;
use tower::ServiceExt;

// ── Project CRUD ──────────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-004-02"]
async fn project_crud_lifecycle() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router(pool.clone());
    let tenant_id = common::create_test_tenant(&pool, "CRUD Test Co").await;

    // Create
    let (status, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({
            "address": "123 Main St",
            "client_name": "Alice",
            "client_email": "alice@example.com"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let project_id = body["id"].as_str().unwrap();
    assert_eq!(body["address"], "123 Main St");
    assert_eq!(body["client_name"], "Alice");
    assert_eq!(body["status"], "draft");

    // Get
    let (status, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], project_id);

    // List
    let (status, body) = common::send(&app, Method::GET, "/projects", tenant_id, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == project_id));

    // Delete
    let (status, _) = common::send(
        &app,
        Method::DELETE,
        &format!("/projects/{project_id}"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Get after delete → 404
    let (status, _) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Tenant isolation ──────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres (S.INFRA.2), tracked in T-004-02"]
async fn tenant_isolation() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router(pool.clone());
    let tenant_a = common::create_test_tenant(&pool, "Tenant A").await;
    let tenant_b = common::create_test_tenant(&pool, "Tenant B").await;

    // Tenant A creates a project
    let (status, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_a,
        Some(json!({"client_name": "A's project"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let project_id = body["id"].as_str().unwrap();

    // Tenant B cannot see it
    let (status, _) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}"),
        tenant_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Tenant B's project list doesn't include it
    let (status, body) = common::send(&app, Method::GET, "/projects", tenant_b, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == project_id));

    // Tenant B cannot delete it
    let (status, _) = common::send(
        &app,
        Method::DELETE,
        &format!("/projects/{project_id}"),
        tenant_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Zone CRUD ─────────────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-004-02"]
async fn zone_crud() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router(pool.clone());
    let tenant_id = common::create_test_tenant(&pool, "Zone Test Co").await;

    // Create project
    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Zone tester"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    let geojson_poly = json!({
        "type": "Polygon",
        "coordinates": [[[0.0, 0.0], [12.0, 0.0], [12.0, 15.0], [0.0, 15.0], [0.0, 0.0]]]
    });

    // Add zone
    let (status, body) = common::send(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        Some(json!({
            "geometry": geojson_poly,
            "zone_type": "patio",
            "label": "Back patio",
            "sort_order": 1
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let zone_id = body["id"].as_str().unwrap();
    assert_eq!(body["zone_type"], "patio");
    assert_eq!(body["label"], "Back patio");

    // Verify computed measurements: 12 × 15 = 180 sq ft, perimeter = 12+15+12+15 = 54 ft
    let area = body["area_sqft"].as_f64().unwrap();
    assert!(
        (area - 180.0).abs() < 0.01,
        "expected area ~180.0, got {area}"
    );
    let perimeter = body["perimeter_ft"].as_f64().unwrap();
    assert!(
        (perimeter - 54.0).abs() < 0.01,
        "expected perimeter ~54.0, got {perimeter}"
    );

    // List zones
    let (status, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);

    // Update zone
    let updated_poly = json!({
        "type": "Polygon",
        "coordinates": [[[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0], [0.0, 0.0]]]
    });
    let (status, _) = common::send(
        &app,
        Method::PATCH,
        &format!("/projects/{project_id}/zones/{zone_id}"),
        tenant_id,
        Some(json!({
            "geometry": updated_poly,
            "zone_type": "lawn",
            "label": "Big lawn",
            "sort_order": 0
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Delete zone
    let (status, _) = common::send(
        &app,
        Method::DELETE,
        &format!("/projects/{project_id}/zones/{zone_id}"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // List zones after delete → empty
    let (status, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().is_empty());
}

// ── Bulk zone update ──────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-004-02"]
async fn zone_bulk_update() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router(pool.clone());
    let tenant_id = common::create_test_tenant(&pool, "Bulk Zone Co").await;

    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Bulk tester"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    // Add two zones individually first
    for label in ["Zone A", "Zone B"] {
        common::send(
            &app,
            Method::POST,
            &format!("/projects/{project_id}/zones"),
            tenant_id,
            Some(json!({
                "geometry": {"type": "Polygon", "coordinates": [[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0],[0.0,0.0]]]},
                "zone_type": "bed",
                "label": label
            })),
        )
        .await;
    }

    // Bulk replace with three new zones
    let (status, body) = common::send(
        &app,
        Method::PUT,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        Some(json!([
            {"geometry": {"type": "Polygon", "coordinates": [[[0.0,0.0],[5.0,0.0],[5.0,5.0],[0.0,5.0],[0.0,0.0]]]}, "zone_type": "patio", "label": "New A"},
            {"geometry": {"type": "Polygon", "coordinates": [[[10.0,0.0],[15.0,0.0],[15.0,5.0],[10.0,5.0],[10.0,0.0]]]}, "zone_type": "lawn", "label": "New B"},
            {"geometry": {"type": "Polygon", "coordinates": [[[20.0,0.0],[25.0,0.0],[25.0,5.0],[20.0,5.0],[20.0,0.0]]]}, "zone_type": "bed", "label": "New C"}
        ])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 3);

    // List should now show exactly 3
    let (_, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(body.as_array().unwrap().len(), 3);
}

// ── Material CRUD ─────────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-004-02"]
async fn material_crud() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router(pool.clone());
    let tenant_id = common::create_test_tenant(&pool, "Material Test Co").await;

    // Create
    let (status, body) = common::send(
        &app,
        Method::POST,
        "/materials",
        tenant_id,
        Some(json!({
            "name": "Travertine Pavers",
            "category": "hardscape",
            "unit": "sq_ft",
            "price_per_unit": "8.50",
            "depth_inches": 1.0,
            "extrusion": {"type": "sits_on_top", "height_inches": 1.0}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let material_id = body["id"].as_str().unwrap();
    assert_eq!(body["name"], "Travertine Pavers");
    assert_eq!(body["category"], "hardscape");

    // List
    let (status, body) = common::send(&app, Method::GET, "/materials", tenant_id, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["id"] == material_id));

    // Update
    let (status, _) = common::send(
        &app,
        Method::PATCH,
        &format!("/materials/{material_id}"),
        tenant_id,
        Some(json!({
            "name": "Premium Travertine",
            "category": "hardscape",
            "unit": "sq_ft",
            "price_per_unit": "12.00",
            "extrusion": {"type": "sits_on_top", "height_inches": 1.5}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Delete
    let (status, _) = common::send(
        &app,
        Method::DELETE,
        &format!("/materials/{material_id}"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ── Tier assignments ──────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-004-02"]
async fn tier_assignments() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router(pool.clone());
    let tenant_id = common::create_test_tenant(&pool, "Tier Test Co").await;

    // Create project, zone, material
    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Tier tester"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    let (_, body) = common::send(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        Some(json!({
            "geometry": {"type": "Polygon", "coordinates": [[[0.0,0.0],[10.0,0.0],[10.0,10.0],[0.0,10.0],[0.0,0.0]]]},
            "zone_type": "patio"
        })),
    )
    .await;
    let zone_id = body["id"].as_str().unwrap();

    let (_, body) = common::send(
        &app,
        Method::POST,
        "/materials",
        tenant_id,
        Some(json!({
            "name": "Concrete",
            "category": "hardscape",
            "unit": "sq_ft",
            "price_per_unit": "5.00",
            "extrusion": {"type": "sits_on_top", "height_inches": 0.5}
        })),
    )
    .await;
    let material_id = body["id"].as_str().unwrap();

    // Set tier assignments for "good"
    let (status, _) = common::send(
        &app,
        Method::PUT,
        &format!("/projects/{project_id}/tiers/good"),
        tenant_id,
        Some(json!({
            "assignments": [
                {"zone_id": zone_id, "material_id": material_id}
            ]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Get all tiers
    let (status, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/tiers"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let tiers = body.as_array().unwrap();
    assert_eq!(tiers.len(), 3);

    // "good" tier should have our assignment
    let good = tiers.iter().find(|t| t["tier"] == "good").unwrap();
    assert_eq!(good["assignments"].as_array().unwrap().len(), 1);
    assert_eq!(good["assignments"][0]["zone_id"], zone_id);
    assert_eq!(good["assignments"][0]["material_id"], material_id);

    // "better" and "best" should be empty
    let better = tiers.iter().find(|t| t["tier"] == "better").unwrap();
    assert!(better["assignments"].as_array().unwrap().is_empty());
}

// ── Quote route ───────────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres (S.3.1/S.3.2), tracked in T-008-01"]
async fn quote_route_integration() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router(pool.clone());
    let tenant_id = common::create_test_tenant(&pool, "Quote Test Co").await;

    // Create project
    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Quote tester"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    // Create zone: 12×15 ft patio (area = 180 sq ft)
    let (_, body) = common::send(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        Some(json!({
            "geometry": {
                "type": "Polygon",
                "coordinates": [[[0.0,0.0],[12.0,0.0],[12.0,15.0],[0.0,15.0],[0.0,0.0]]]
            },
            "zone_type": "patio",
            "label": "Back patio",
            "sort_order": 1
        })),
    )
    .await;
    let zone_id = body["id"].as_str().unwrap();

    // Create material: Pavers at $8.50/sq_ft
    let (_, body) = common::send(
        &app,
        Method::POST,
        "/materials",
        tenant_id,
        Some(json!({
            "name": "Travertine Pavers",
            "category": "hardscape",
            "unit": "sq_ft",
            "price_per_unit": "8.50",
            "extrusion": {"type": "sits_on_top", "height_inches": 1.0}
        })),
    )
    .await;
    let material_id = body["id"].as_str().unwrap();

    // Set tier assignment for "good"
    let (status, _) = common::send(
        &app,
        Method::PUT,
        &format!("/projects/{project_id}/tiers/good"),
        tenant_id,
        Some(json!({
            "assignments": [{"zone_id": zone_id, "material_id": material_id}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // GET quote for "good" tier — should have 1 line item
    let (status, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/quote/good"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["tier"], "good");

    let line_items = body["line_items"].as_array().unwrap();
    assert_eq!(line_items.len(), 1);

    // 12 × 15 = 180 sq_ft × $8.50 = $1,530.00
    assert_eq!(line_items[0]["material_name"], "Travertine Pavers");
    assert_eq!(line_items[0]["line_total"], "1530.00");
    assert_eq!(body["subtotal"], "1530.00");
    assert_eq!(body["total"], "1530.00");
    assert_eq!(body["tax"], serde_json::Value::Null);

    // GET quote for "better" tier — no assignments, should return empty quote
    let (status, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/quote/better"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["tier"], "better");
    assert!(body["line_items"].as_array().unwrap().is_empty());
    assert_eq!(body["subtotal"], "0");
    assert_eq!(body["total"], "0");

    // Invalid tier name → 400
    let (status, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/quote/premium"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("invalid tier"));

    // Non-existent project → 404
    let fake_id = uuid::Uuid::new_v4();
    let (status, _) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{fake_id}/quote/good"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Validation ────────────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-004-02"]
async fn missing_tenant_header_returns_400() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router(pool);

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/projects")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-004-02"]
async fn invalid_tier_name_returns_400() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router(pool.clone());
    let tenant_id = common::create_test_tenant(&pool, "Validation Co").await;

    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Validator"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    let (status, body) = common::send(
        &app,
        Method::PUT,
        &format!("/projects/{project_id}/tiers/premium"),
        tenant_id,
        Some(json!({"assignments": []})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("invalid tier"));
}
