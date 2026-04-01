//! Scan upload, processing status, and plan view routes.

use std::sync::Arc;

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extract::TenantId;
use crate::scan_job::ScanJobTracker;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects/{id}/scan", post(upload_scan))
        .route("/projects/{id}/scan/status", get(scan_status))
        .route("/projects/{id}/planview", get(get_planview))
}

// ── DTOs ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct UploadResponse {
    job_id: Uuid,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ScanStatusResponse {
    job_id: Option<Uuid>,
    status: String,
    error: Option<String>,
    scan_ref: Option<serde_json::Value>,
}

// ── Handlers ──────────────────��───────────────────────────────

async fn upload_scan(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    // Verify project belongs to tenant
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;

    // Extract PLY bytes from multipart
    let ply_bytes = extract_ply_field(&mut multipart).await?;

    if ply_bytes.is_empty() {
        return Err(AppError::BadRequest("empty PLY file".to_string()));
    }

    // Upload raw PLY to S3
    let ply_key = format!("scans/{project_id}/raw.ply");
    crate::s3::upload_bytes(
        &state.s3_client,
        &state.s3_bucket,
        &ply_key,
        ply_bytes,
        "application/octet-stream",
    )
    .await?;

    // Create job and spawn processing
    let job = state.scan_jobs.create(project_id);
    let job_id = job.id;

    let pool = state.pool.clone();
    let s3_client = state.s3_client.clone();
    let s3_bucket = state.s3_bucket.clone();
    let scan_jobs = Arc::clone(&state.scan_jobs);

    tokio::spawn(async move {
        process_scan_job(
            pool, s3_client, s3_bucket, scan_jobs, job_id, project_id, ply_key,
        )
        .await;
    });

    let response = UploadResponse {
        job_id,
        status: "pending",
    };
    Ok((StatusCode::ACCEPTED, Json(response)).into_response())
}

async fn scan_status(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ScanStatusResponse>, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;

    let job = state.scan_jobs.get_by_project(project_id);

    match job {
        Some(job) => {
            // If complete, include scan_ref from the database
            let scan_ref = if job.status == crate::scan_job::ScanJobStatus::Complete {
                let row = pt_repo::project::get_by_id(&state.pool, project_id).await?;
                row.scan_ref
            } else {
                None
            };

            Ok(Json(ScanStatusResponse {
                job_id: Some(job.id),
                status: format!("{:?}", job.status).to_lowercase(),
                error: job.error,
                scan_ref,
            }))
        }
        None => {
            // No job exists — check if project already has a scan_ref from a previous session
            let row = pt_repo::project::get_by_id(&state.pool, project_id).await?;
            if row.scan_ref.is_some() {
                Ok(Json(ScanStatusResponse {
                    job_id: None,
                    status: "complete".to_string(),
                    error: None,
                    scan_ref: row.scan_ref,
                }))
            } else {
                Ok(Json(ScanStatusResponse {
                    job_id: None,
                    status: "none".to_string(),
                    error: None,
                    scan_ref: None,
                }))
            }
        }
    }
}

async fn get_planview(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Response, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;

    let row = pt_repo::project::get_by_id(&state.pool, project_id).await?;
    let scan_ref = row.scan_ref.ok_or(AppError::NotFound)?;

    let planview_key = scan_ref
        .get("planview_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Internal("scan_ref missing planview_key".to_string()))?;

    let url = crate::s3::presigned_get_url(&state.s3_client, &state.s3_bucket, planview_key, 3600)
        .await?;

    Ok(Redirect::temporary(&url).into_response())
}

// ── Helpers ───────���───────────────────────────────────────────

async fn verify_project_tenant(
    pool: &sqlx::PgPool,
    project_id: Uuid,
    tenant_id: Uuid,
) -> Result<(), AppError> {
    let row = pt_repo::project::get_by_id(pool, project_id).await?;
    if row.tenant_id != tenant_id {
        return Err(AppError::NotFound);
    }
    Ok(())
}

async fn extract_ply_field(multipart: &mut Multipart) -> Result<Vec<u8>, AppError> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("failed to read file field: {e}")))?;
            return Ok(bytes.to_vec());
        }
    }
    Err(AppError::BadRequest(
        "missing 'file' field in multipart upload".to_string(),
    ))
}

/// Background task: download PLY from S3, process, upload artifacts, update project.
async fn process_scan_job(
    pool: sqlx::PgPool,
    s3_client: aws_sdk_s3::Client,
    s3_bucket: String,
    scan_jobs: Arc<ScanJobTracker>,
    job_id: Uuid,
    project_id: Uuid,
    ply_key: String,
) {
    scan_jobs.set_processing(job_id);

    let result = run_scan_pipeline(&pool, &s3_client, &s3_bucket, project_id, &ply_key).await;

    match result {
        Ok(()) => {
            scan_jobs.set_complete(job_id);
            tracing::info!("Scan processing complete for project {project_id}");
        }
        Err(e) => {
            let msg = format!("{e:?}");
            tracing::error!("Scan processing failed for project {project_id}: {msg}");
            scan_jobs.set_failed(job_id, msg);
        }
    }
}

async fn run_scan_pipeline(
    pool: &sqlx::PgPool,
    s3_client: &aws_sdk_s3::Client,
    s3_bucket: &str,
    project_id: Uuid,
    ply_key: &str,
) -> Result<(), AppError> {
    // 1. Download PLY from S3
    let ply_bytes = crate::s3::download_bytes(s3_client, s3_bucket, ply_key).await?;

    // 2. Process scan (CPU-bound — use spawn_blocking)
    let output = tokio::task::spawn_blocking(move || {
        let config = pt_scan::ScanConfig::default();
        let cloud = pt_scan::process_scan(std::io::Cursor::new(ply_bytes), &config)?;

        let export_config = pt_scan::ExportConfig::default();
        pt_scan::generate_terrain(&cloud, &export_config)
    })
    .await
    .map_err(|e| AppError::Internal(format!("scan task panicked: {e}")))?
    .map_err(AppError::from)?;

    // 3. Upload artifacts to S3
    let terrain_key = format!("scans/{project_id}/terrain.glb");
    let planview_key = format!("scans/{project_id}/planview.png");
    let metadata_key = format!("scans/{project_id}/metadata.json");

    let metadata_json = serde_json::to_vec(&output.metadata)
        .map_err(|e| AppError::Internal(format!("serialize metadata: {e}")))?;

    // Upload all three artifacts
    crate::s3::upload_bytes(
        s3_client,
        s3_bucket,
        &terrain_key,
        output.mesh_glb,
        "model/gltf-binary",
    )
    .await?;

    crate::s3::upload_bytes(
        s3_client,
        s3_bucket,
        &planview_key,
        output.plan_view_png,
        "image/png",
    )
    .await?;

    crate::s3::upload_bytes(
        s3_client,
        s3_bucket,
        &metadata_key,
        metadata_json,
        "application/json",
    )
    .await?;

    // 4. Update project scan_ref
    let scan_ref = serde_json::json!({
        "ply_key": ply_key,
        "terrain_key": terrain_key,
        "planview_key": planview_key,
        "metadata_key": metadata_key,
        "processed_at": chrono::Utc::now().to_rfc3339(),
    });

    pt_repo::project::set_scan_ref(pool, project_id, &scan_ref).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn s3_key_format() {
        let project_id = uuid::Uuid::new_v4();
        let ply_key = format!("scans/{project_id}/raw.ply");
        assert!(ply_key.starts_with("scans/"));
        assert!(ply_key.ends_with("/raw.ply"));

        let terrain_key = format!("scans/{project_id}/terrain.glb");
        assert!(terrain_key.ends_with("/terrain.glb"));
    }
}
