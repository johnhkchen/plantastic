//! Zone CRUD routes (nested under /projects/:id).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use geo::Polygon;
use pt_project::ZoneType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::extract::TenantId;
use crate::state::AppState;
use pt_geo::area::area_sqft;
use pt_geo::perimeter::perimeter_ft;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{id}/zones",
            get(list_zones).post(add_zone).put(bulk_update_zones),
        )
        .route(
            "/projects/{id}/zones/{zid}",
            axum::routing::patch(update_zone).delete(delete_zone),
        )
}

// ── Tenant verification ───────────────────────────────────────

/// Verify that a project belongs to the given tenant. Returns 404 if not.
pub(crate) async fn verify_project_tenant(
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

// ── DTOs ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ZoneResponse {
    id: Uuid,
    project_id: Uuid,
    geometry: serde_json::Value,
    zone_type: ZoneType,
    label: Option<String>,
    sort_order: i32,
    area_sqft: f64,
    perimeter_ft: f64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn zone_row_to_response(r: pt_repo::zone::ZoneRow) -> ZoneResponse {
    let geom = geojson::Geometry::from(&r.geometry);
    let area = area_sqft(&r.geometry);
    let perimeter = perimeter_ft(&r.geometry);
    ZoneResponse {
        id: r.id,
        project_id: r.project_id,
        geometry: serde_json::to_value(geom).expect("geometry serialization"),
        zone_type: r.zone_type,
        label: r.label,
        sort_order: r.sort_order,
        area_sqft: area,
        perimeter_ft: perimeter,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(Debug, Deserialize)]
struct AddZoneRequest {
    geometry: serde_json::Value,
    zone_type: ZoneType,
    label: Option<String>,
    #[serde(default)]
    sort_order: i32,
}

#[derive(Debug, Deserialize)]
struct UpdateZoneRequest {
    geometry: serde_json::Value,
    zone_type: ZoneType,
    label: Option<String>,
    #[serde(default)]
    sort_order: i32,
}

#[derive(Debug, Deserialize)]
struct BulkZoneEntry {
    geometry: serde_json::Value,
    zone_type: ZoneType,
    label: Option<String>,
    #[serde(default)]
    sort_order: i32,
}

// ── Geometry conversion ───────────────────────────────────────

fn parse_geojson_polygon(value: &serde_json::Value) -> Result<Polygon<f64>, AppError> {
    let geom: geojson::Geometry = serde_json::from_value(value.clone())
        .map_err(|e| AppError::BadRequest(format!("invalid GeoJSON geometry: {e}")))?;

    let geo_geom: geo::Geometry<f64> = geom
        .try_into()
        .map_err(|e| AppError::BadRequest(format!("geometry conversion failed: {e}")))?;

    match geo_geom {
        geo::Geometry::Polygon(p) => Ok(p),
        _ => Err(AppError::BadRequest(
            "expected Polygon geometry".to_string(),
        )),
    }
}

// ── Handlers ──────────────────────────────────────────────────

async fn list_zones(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ZoneResponse>>, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;
    let rows = pt_repo::zone::list_by_project(&state.pool, project_id).await?;
    Ok(Json(rows.into_iter().map(zone_row_to_response).collect()))
}

async fn add_zone(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<AddZoneRequest>,
) -> Result<(StatusCode, Json<ZoneResponse>), AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;
    let polygon = parse_geojson_polygon(&body.geometry)?;

    let input = pt_repo::zone::CreateZone {
        project_id,
        geometry: polygon,
        zone_type: body.zone_type,
        label: body.label,
        sort_order: body.sort_order,
    };
    let zone_id = pt_repo::zone::add(&state.pool, &input).await?;

    // Re-fetch to get all server-generated fields
    let rows = pt_repo::zone::list_by_project(&state.pool, project_id).await?;
    let row = rows
        .into_iter()
        .find(|r| r.id == zone_id)
        .ok_or(AppError::Internal("created zone not found".to_string()))?;

    Ok((StatusCode::CREATED, Json(zone_row_to_response(row))))
}

async fn bulk_update_zones(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<Vec<BulkZoneEntry>>,
) -> Result<Json<Vec<Uuid>>, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;

    let mut create_zones = Vec::with_capacity(body.len());
    for entry in &body {
        let polygon = parse_geojson_polygon(&entry.geometry)?;
        create_zones.push(pt_repo::zone::CreateZone {
            project_id,
            geometry: polygon,
            zone_type: entry.zone_type,
            label: entry.label.clone(),
            sort_order: entry.sort_order,
        });
    }

    let ids = pt_repo::zone::bulk_upsert(&state.pool, project_id, &create_zones).await?;
    Ok(Json(ids))
}

async fn update_zone(
    tenant: TenantId,
    State(state): State<AppState>,
    Path((project_id, zone_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateZoneRequest>,
) -> Result<StatusCode, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;
    let polygon = parse_geojson_polygon(&body.geometry)?;

    pt_repo::zone::update(
        &state.pool,
        zone_id,
        &polygon,
        body.zone_type,
        body.label.as_deref(),
        body.sort_order,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_zone(
    tenant: TenantId,
    State(state): State<AppState>,
    Path((project_id, zone_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;
    pt_repo::zone::delete(&state.pool, zone_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
