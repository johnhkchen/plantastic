//! Tier assignment routes (nested under /projects/:id).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use pt_project::TierLevel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::extract::TenantId;
use crate::routes::zones::verify_project_tenant;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects/{id}/tiers", get(get_all_tiers))
        .route(
            "/projects/{id}/tiers/{tier}",
            axum::routing::put(set_tier_assignments),
        )
}

// ── DTOs ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct TierResponse {
    tier: TierLevel,
    assignments: Vec<AssignmentResponse>,
}

#[derive(Debug, Serialize)]
struct AssignmentResponse {
    zone_id: Uuid,
    material_id: Uuid,
    overrides: Option<serde_json::Value>,
}

impl From<pt_repo::tier_assignment::TierAssignmentRow> for AssignmentResponse {
    fn from(r: pt_repo::tier_assignment::TierAssignmentRow) -> Self {
        Self {
            zone_id: r.zone_id,
            material_id: r.material_id,
            overrides: r.overrides,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SetAssignmentsRequest {
    assignments: Vec<AssignmentInput>,
}

#[derive(Debug, Deserialize)]
struct AssignmentInput {
    zone_id: Uuid,
    material_id: Uuid,
    overrides: Option<serde_json::Value>,
}

// ── Tier level parsing ────────────────────────────────────────

pub(crate) fn parse_tier(s: &str) -> Result<TierLevel, AppError> {
    match s {
        "good" => Ok(TierLevel::Good),
        "better" => Ok(TierLevel::Better),
        "best" => Ok(TierLevel::Best),
        _ => Err(AppError::BadRequest(format!(
            "invalid tier: {s} (expected good, better, or best)"
        ))),
    }
}

// ── Handlers ──────────────────────────────────────────────────

async fn get_all_tiers(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<TierResponse>>, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;

    let mut tiers = Vec::with_capacity(3);
    for level in [TierLevel::Good, TierLevel::Better, TierLevel::Best] {
        let rows =
            pt_repo::tier_assignment::get_by_project_and_tier(&state.pool, project_id, level)
                .await?;
        tiers.push(TierResponse {
            tier: level,
            assignments: rows.into_iter().map(Into::into).collect(),
        });
    }

    Ok(Json(tiers))
}

async fn set_tier_assignments(
    tenant: TenantId,
    State(state): State<AppState>,
    Path((project_id, tier_str)): Path<(Uuid, String)>,
    Json(body): Json<SetAssignmentsRequest>,
) -> Result<StatusCode, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;
    let tier = parse_tier(&tier_str)?;

    let assignments: Vec<pt_repo::tier_assignment::SetAssignment> = body
        .assignments
        .into_iter()
        .map(|a| pt_repo::tier_assignment::SetAssignment {
            zone_id: a.zone_id,
            material_id: a.material_id,
            overrides: a.overrides,
        })
        .collect();

    pt_repo::tier_assignment::set_assignments(&state.pool, project_id, tier, &assignments).await?;
    Ok(StatusCode::NO_CONTENT)
}
