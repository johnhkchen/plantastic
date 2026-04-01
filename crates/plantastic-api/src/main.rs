//! Plantastic API — Axum server with Lambda/local dual mode.
//!
//! Detects Lambda vs local via the `AWS_LAMBDA_RUNTIME_API` env var.
//! Lambda mode uses lambda_http adapter. Local mode binds to PORT (default 3000).

use std::sync::Arc;

use plantastic_api::scan_job::ScanJobTracker;
use plantastic_api::AppState;

#[tokio::main]
async fn main() {
    // Load .env for local development (silently ignored in Lambda)
    dotenvy::dotenv().ok();

    // Initialize tracing
    init_tracing();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        tracing::warn!("DATABASE_URL not set — using default localhost connection");
        "postgres://localhost:5432/plantastic_dev".to_string()
    });

    let pool = pt_repo::create_pool(&database_url)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to create database pool: {e}");
            std::process::exit(1);
        });

    tracing::info!("Database pool created");

    let s3_client = plantastic_api::s3::create_s3_client().await;
    let s3_bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| {
        tracing::warn!("S3_BUCKET not set — using default 'plantastic-dev'");
        "plantastic-dev".to_string()
    });

    tracing::info!("S3 client initialized (bucket: {s3_bucket})");

    let state = AppState {
        pool,
        s3_client,
        s3_bucket,
        scan_jobs: Arc::new(ScanJobTracker::new()),
        proposal_generator: Arc::new(pt_proposal::BamlProposalGenerator),
    };
    let router = plantastic_api::router(state);

    if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
        run_lambda(router).await;
    } else {
        run_local(router).await;
    }
}

async fn run_lambda(router: axum::Router) {
    tracing::info!("Starting in Lambda mode");
    lambda_http::run(router)
        .await
        .expect("Lambda runtime error");
}

async fn run_local(router: axum::Router) {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting local server on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, router).await.expect("Server error");
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let is_lambda = std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok();

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("plantastic_api=debug,pt_repo=debug,info"));

    if is_lambda {
        // JSON format for CloudWatch
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .with_target(false)
            .init();
    } else {
        // Pretty format for local dev
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .pretty()
            .init();
    }
}
