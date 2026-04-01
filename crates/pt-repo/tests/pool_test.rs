//! Integration tests for database pool creation and configuration.
//!
//! These tests verify that create_pool() and create_pool_with_config() work
//! against a real Postgres instance. They exercise the retry and timeout logic
//! indirectly — a local Postgres should connect on the first attempt.

#![cfg(feature = "integration")]

use std::time::Duration;

use pt_repo::{create_pool, create_pool_with_config, PoolConfig};

fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://plantastic:plantastic@localhost:5432/plantastic".into())
}

#[tokio::test]
async fn pool_connects_with_defaults() {
    let pool = create_pool(&database_url()).await.expect("should connect");
    // Verify the pool is functional
    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
    assert_eq!(row.0, 1);
    pool.close().await;
}

#[tokio::test]
async fn pool_connects_with_custom_config() {
    let config = PoolConfig {
        connect_timeout: Duration::from_secs(5),
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(10),
        max_connections: 2,
        min_connections: 0,
        max_retries: 1,
        initial_backoff: Duration::from_millis(100),
    };
    let pool = create_pool_with_config(&database_url(), &config)
        .await
        .expect("should connect with custom config");
    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
    assert_eq!(row.0, 1);
    pool.close().await;
}

#[tokio::test]
async fn pool_rejects_invalid_url_without_retry() {
    // Configuration errors should fail immediately, not retry
    let result = create_pool("not-a-valid-url").await;
    assert!(result.is_err(), "invalid URL should fail");
}

#[tokio::test]
async fn pool_respects_max_connections() {
    let config = PoolConfig {
        max_connections: 2,
        ..PoolConfig::default()
    };
    let pool = create_pool_with_config(&database_url(), &config)
        .await
        .expect("should connect");
    assert_eq!(pool.options().get_max_connections(), 2);
    pool.close().await;
}
