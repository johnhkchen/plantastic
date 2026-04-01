//! Project CRUD routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::extract::TenantId;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects", post(create_project).get(list_projects))
        .route("/projects/{id}", get(get_project).delete(delete_project))
}

// ── DTOs ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CreateProjectRequest {
    address: Option<String>,
    client_name: Option<String>,
    client_email: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProjectResponse {
    id: Uuid,
    tenant_id: Uuid,
    client_name: Option<String>,
    client_email: Option<String>,
    address: Option<String>,
    scan_ref: Option<serde_json::Value>,
    baseline: Option<serde_json::Value>,
    status: pt_project::ProjectStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<pt_repo::project::ProjectRow> for ProjectResponse {
    fn from(r: pt_repo::project::ProjectRow) -> Self {
        Self {
            id: r.id,
            tenant_id: r.tenant_id,
            client_name: r.client_name,
            client_email: r.client_email,
            address: r.address,
            scan_ref: r.scan_ref,
            baseline: r.baseline,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────

async fn create_project(
    tenant: TenantId,
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    let address_for_baseline = body.address.clone();
    let input = pt_repo::project::CreateProject {
        tenant_id: tenant.0,
        client_name: body.client_name,
        client_email: body.client_email,
        address: body.address,
    };
    let id = pt_repo::project::create(&state.pool, &input).await?;

    // If an address was provided, attempt satellite baseline generation.
    if let Some(addr) = address_for_baseline {
        let baseline_result = tokio::task::spawn_blocking(move || {
            let source = pt_satellite::EmbeddedSource;
            let builder =
                pt_satellite::BaselineBuilder::new(source.clone(), source.clone(), source);
            builder.build(&addr)
        })
        .await;

        match baseline_result {
            Ok(Ok(baseline)) => match serde_json::to_value(&baseline) {
                Ok(json_val) => {
                    if let Err(e) = pt_repo::project::set_baseline(&state.pool, id, &json_val).await
                    {
                        tracing::warn!("failed to store baseline for project {id}: {e}");
                    }
                }
                Err(e) => {
                    tracing::error!("failed to serialize baseline: {e}");
                }
            },
            Ok(Err(e)) => {
                tracing::warn!("baseline generation skipped for project {id}: {e}");
            }
            Err(e) => {
                tracing::error!("baseline task panicked for project {id}: {e}");
            }
        }
    }

    let row = pt_repo::project::get_by_id(&state.pool, id).await?;
    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn list_projects(
    tenant: TenantId,
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectResponse>>, AppError> {
    let rows = pt_repo::project::list_by_tenant(&state.pool, tenant.0).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

async fn get_project(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, AppError> {
    let row = pt_repo::project::get_by_id(&state.pool, id).await?;
    if row.tenant_id != tenant.0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(row.into()))
}

async fn delete_project(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let row = pt_repo::project::get_by_id(&state.pool, id).await?;
    if row.tenant_id != tenant.0 {
        return Err(AppError::NotFound);
    }
    pt_repo::project::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
