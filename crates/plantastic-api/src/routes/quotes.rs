//! Quote computation routes.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::error::AppError;
use crate::extract::TenantId;
use crate::routes::shared::{build_tier, material_rows_to_materials, zone_rows_to_zones};
use crate::routes::tiers::parse_tier;
use crate::routes::zones::verify_project_tenant;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/projects/{id}/quote/{tier}", get(get_quote))
}

// ── Handler ──────────────────────────────────────────────────

async fn get_quote(
    tenant: TenantId,
    State(state): State<AppState>,
    Path((project_id, tier_str)): Path<(Uuid, String)>,
) -> Result<Json<pt_quote::Quote>, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;
    let level = parse_tier(&tier_str)?;

    // Load data from the database
    let zone_rows = pt_repo::zone::list_by_project(&state.pool, project_id).await?;
    let assignment_rows =
        pt_repo::tier_assignment::get_by_project_and_tier(&state.pool, project_id, level).await?;
    let project_row = pt_repo::project::get_by_id(&state.pool, project_id).await?;
    let material_rows =
        pt_repo::material::list_by_tenant(&state.pool, project_row.tenant_id).await?;

    // Convert repo types → domain types
    let zones = zone_rows_to_zones(zone_rows);
    let materials = material_rows_to_materials(material_rows);
    let tier = build_tier(level, assignment_rows)?;

    // Compute the quote (pure, no I/O)
    let quote = pt_quote::compute_quote(&zones, &tier, &materials, None)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(quote))
}
