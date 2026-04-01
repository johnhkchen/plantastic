//! Integration tests for scan upload and processing routes.
//!
//! All tests require a real PostgreSQL database and S3 (or LocalStack).
//! Run with:
//!   DATABASE_URL=postgres://... S3_BUCKET=plantastic-test cargo test -p plantastic-api -- --ignored

mod common;

use axum::http::{Method, StatusCode};
use serde_json::json;
use tower::ServiceExt;

// ── Upload ───────────────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres + S3 (S.1.1 TwoStar), tracked in T-016-01"]
async fn scan_upload_returns_202() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router_full(pool.clone()).await;
    let tenant_id = common::create_test_tenant(&pool, "Scan Upload Co").await;

    // Create project
    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Scanner"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    // Build synthetic PLY (same as S.1.1 scenario)
    let ply_data = build_test_ply();

    // Upload
    let (status, body) = common::send_multipart(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/scan"),
        tenant_id,
        &ply_data,
    )
    .await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(body["job_id"].is_string(), "response should contain job_id");
    assert_eq!(body["status"], "pending");
}

// ── Status polling ───────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres + S3 (S.1.1 TwoStar), tracked in T-016-01"]
async fn scan_status_completes_after_processing() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router_full(pool.clone()).await;
    let tenant_id = common::create_test_tenant(&pool, "Scan Status Co").await;

    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Status checker"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    // Upload
    let ply_data = build_test_ply();
    let (status, _) = common::send_multipart(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/scan"),
        tenant_id,
        &ply_data,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);

    // Poll until complete (max 30 seconds)
    let mut attempts = 0;
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let (status, body) = common::send(
            &app,
            Method::GET,
            &format!("/projects/{project_id}/scan/status"),
            tenant_id,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let job_status = body["status"].as_str().unwrap();
        if job_status == "complete" {
            // Verify scan_ref is populated
            assert!(
                body["scan_ref"].is_object(),
                "scan_ref should be populated on completion"
            );
            assert!(body["scan_ref"]["terrain_key"].is_string());
            assert!(body["scan_ref"]["planview_key"].is_string());
            assert!(body["scan_ref"]["metadata_key"].is_string());
            break;
        }
        if job_status == "failed" {
            panic!("scan processing failed: {:?}", body["error"]);
        }

        attempts += 1;
        assert!(
            attempts < 60,
            "scan processing did not complete within 30 seconds"
        );
    }
}

// ── scan_ref in project response ─────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres + S3 (S.1.1 TwoStar), tracked in T-016-01"]
async fn project_response_includes_scan_ref() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router_full(pool.clone()).await;
    let tenant_id = common::create_test_tenant(&pool, "ScanRef Co").await;

    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Ref checker"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    // Initially null
    let (_, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}"),
        tenant_id,
        None,
    )
    .await;
    assert!(
        body["scan_ref"].is_null(),
        "scan_ref should be null before upload"
    );

    // Upload and wait for processing
    let ply_data = build_test_ply();
    common::send_multipart(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/scan"),
        tenant_id,
        &ply_data,
    )
    .await;

    wait_for_scan_complete(&app, project_id, tenant_id).await;

    // Now project response should include scan_ref
    let (_, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}"),
        tenant_id,
        None,
    )
    .await;
    assert!(
        body["scan_ref"].is_object(),
        "scan_ref should be populated after processing"
    );
}

// ── Plan view redirect ───────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres + S3 (S.1.1 TwoStar), tracked in T-016-01"]
async fn planview_returns_redirect_after_processing() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router_full(pool.clone()).await;
    let tenant_id = common::create_test_tenant(&pool, "Planview Co").await;

    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Planview checker"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    // Before upload — 404
    let (status, _) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/planview"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Upload and wait
    let ply_data = build_test_ply();
    common::send_multipart(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/scan"),
        tenant_id,
        &ply_data,
    )
    .await;
    wait_for_scan_complete(&app, project_id, tenant_id).await;

    // After processing — 307 redirect
    let request = axum::http::Request::builder()
        .method(Method::GET)
        .uri(format!("/projects/{project_id}/planview"))
        .header("X-Tenant-Id", tenant_id.to_string())
        .header("Content-Length", "0")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert!(response.headers().contains_key("location"));
}

// ── Status before any upload ─────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres (S.1.1), tracked in T-016-01"]
async fn scan_status_returns_none_before_upload() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router_full(pool.clone()).await;
    let tenant_id = common::create_test_tenant(&pool, "NoScan Co").await;

    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "No scan yet"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    let (status, body) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/scan/status"),
        tenant_id,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "none");
    assert!(body["scan_ref"].is_null());
}

// ── Tenant isolation ─────────────────────────────────────────

#[tokio::test]
#[ignore = "Requires Postgres + S3 (S.INFRA.2), tracked in T-016-01"]
async fn scan_upload_tenant_isolation() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    let app = common::test_router_full(pool.clone()).await;
    let tenant_a = common::create_test_tenant(&pool, "Tenant A").await;
    let tenant_b = common::create_test_tenant(&pool, "Tenant B").await;

    // Tenant A creates a project
    let (_, body) = common::send(
        &app,
        Method::POST,
        "/projects",
        tenant_a,
        Some(json!({"client_name": "A's project"})),
    )
    .await;
    let project_id = body["id"].as_str().unwrap();

    // Tenant B cannot upload to A's project
    let ply_data = build_test_ply();
    let (status, _) = common::send_multipart(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/scan"),
        tenant_b,
        &ply_data,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Tenant B cannot check scan status of A's project
    let (status, _) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/scan/status"),
        tenant_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Tenant B cannot access A's planview
    let (status, _) = common::send(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/planview"),
        tenant_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Helpers ──────────────────────────────────────────────────

/// Build a synthetic PLY file for testing (630 points: 500 ground + 100 obstacles + 30 outliers).
/// Same data as S.1.1 scenario — computed independently from pt_scan.
fn build_test_ply() -> Vec<u8> {
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

    // 500 ground points (z near 0)
    for i in 0..500_u32 {
        let x = (i % 25) as f32 * 0.4;
        let y = (i / 25) as f32 * 0.4;
        let z = if i % 2 == 0 { 0.002_f32 } else { -0.002 };
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.extend_from_slice(&z.to_le_bytes());
        buf.extend_from_slice(&[0, 128, 0]);
    }
    // 100 obstacle points (z = 0.5)
    for i in 0..100_u32 {
        let x = 2.0 + (i % 10) as f32 * 0.1;
        let y = 2.0 + (i / 10) as f32 * 0.1;
        let z = 0.5_f32;
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.extend_from_slice(&z.to_le_bytes());
        buf.extend_from_slice(&[128, 0, 0]);
    }
    // 30 outlier points (z = 20+)
    for i in 0..30_u32 {
        let v = 20.0 + i as f32;
        buf.extend_from_slice(&v.to_le_bytes());
        buf.extend_from_slice(&v.to_le_bytes());
        buf.extend_from_slice(&v.to_le_bytes());
        buf.extend_from_slice(&[255, 255, 255]);
    }
    buf
}

/// Poll scan status until complete (max 30 seconds).
async fn wait_for_scan_complete(app: &axum::Router, project_id: &str, tenant_id: uuid::Uuid) {
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let (_, body) = common::send(
            app,
            axum::http::Method::GET,
            &format!("/projects/{project_id}/scan/status"),
            tenant_id,
            None,
        )
        .await;
        match body["status"].as_str() {
            Some("complete") => return,
            Some("failed") => panic!("scan processing failed: {:?}", body["error"]),
            _ => continue,
        }
    }
    panic!("scan processing did not complete within 30 seconds");
}
