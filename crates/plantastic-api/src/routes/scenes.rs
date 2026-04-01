//! Scene generation routes — generates 3D glTF scenes from project data.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use pt_scene::SceneMetadata;
use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extract::TenantId;
use crate::routes::shared::{build_tier, material_rows_to_materials, zone_rows_to_zones};
use crate::routes::tiers::parse_tier;
use crate::routes::zones::verify_project_tenant;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/projects/{id}/scene/{tier}", get(get_scene))
}

// ── Response ─────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SceneResponse {
    /// Presigned S3 URL to the generated .glb file.
    pub url: String,
    /// Scene metadata (zone count, triangle count, tier).
    pub metadata: SceneMetadata,
}

// ── Handler ──────────────────────────────────────────────────

async fn get_scene(
    tenant: TenantId,
    State(state): State<AppState>,
    Path((project_id, tier_str)): Path<(Uuid, String)>,
) -> Result<Json<SceneResponse>, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;
    let level = parse_tier(&tier_str)?;

    // Load project data from the database.
    let zone_rows = pt_repo::zone::list_by_project(&state.pool, project_id).await?;
    let assignment_rows =
        pt_repo::tier_assignment::get_by_project_and_tier(&state.pool, project_id, level).await?;
    let project_row = pt_repo::project::get_by_id(&state.pool, project_id).await?;
    let material_rows =
        pt_repo::material::list_by_tenant(&state.pool, project_row.tenant_id).await?;

    // Convert repo types → domain types.
    let zones = zone_rows_to_zones(zone_rows);
    let materials = material_rows_to_materials(material_rows);
    let tier = build_tier(level, assignment_rows)?;

    // Scene generation is CPU-bound (triangulation + glTF export) — use spawn_blocking.
    let tier_level = tier.level;
    let assignments = tier.assignments.clone();
    let output = tokio::task::spawn_blocking(move || {
        pt_scene::generate_scene(&zones, &assignments, &materials, tier_level)
    })
    .await
    .map_err(|e| AppError::Internal(format!("scene generation task panicked: {e}")))?
    .map_err(AppError::from)?;

    // Upload GLB to S3.
    let s3_key = format!(
        "scenes/{project_id}/{tier}.glb",
        tier = tier_str.to_lowercase()
    );
    crate::s3::upload_bytes(
        &state.s3_client,
        &state.s3_bucket,
        &s3_key,
        output.glb_bytes,
        "model/gltf-binary",
    )
    .await?;

    // Return presigned URL (1 hour TTL).
    let url =
        crate::s3::presigned_get_url(&state.s3_client, &state.s3_bucket, &s3_key, 3600).await?;

    Ok(Json(SceneResponse {
        url,
        metadata: output.metadata,
    }))
}
