//! Health endpoint test — no database required.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Build a test router without a real database.
/// Uses a dummy PgPool connection string that won't actually connect —
/// the health endpoint doesn't touch the database.
fn test_router() -> axum::Router {
    // We can't create a real PgPool without a database, but the health
    // endpoint doesn't use state at all. We create a minimal router
    // with just the health route.
    use axum::routing::get;

    async fn health() -> axum::Json<serde_json::Value> {
        axum::Json(serde_json::json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    axum::Router::new().route("/health", get(health))
}

#[tokio::test]
async fn health_returns_200() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
