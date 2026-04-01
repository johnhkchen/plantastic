//! Material catalog CRUD routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use pt_materials::{ExtrusionBehavior, MaterialCategory, Unit};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::extract::TenantId;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/materials", get(list_materials).post(create_material))
        .route(
            "/materials/{id}",
            axum::routing::patch(update_material).delete(delete_material),
        )
}

// ── DTOs ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CreateMaterialRequest {
    name: String,
    category: MaterialCategory,
    unit: Unit,
    price_per_unit: Decimal,
    depth_inches: Option<f64>,
    extrusion: ExtrusionBehavior,
    texture_key: Option<String>,
    photo_key: Option<String>,
    supplier_sku: Option<String>,
}

#[derive(Debug, Serialize)]
struct MaterialResponse {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    category: MaterialCategory,
    unit: Unit,
    price_per_unit: Decimal,
    depth_inches: Option<f64>,
    extrusion: ExtrusionBehavior,
    texture_key: Option<String>,
    photo_key: Option<String>,
    supplier_sku: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<pt_repo::material::MaterialRow> for MaterialResponse {
    fn from(r: pt_repo::material::MaterialRow) -> Self {
        Self {
            id: r.id,
            tenant_id: r.tenant_id,
            name: r.name,
            category: r.category,
            unit: r.unit,
            price_per_unit: r.price_per_unit,
            depth_inches: r.depth_inches,
            extrusion: r.extrusion,
            texture_key: r.texture_key,
            photo_key: r.photo_key,
            supplier_sku: r.supplier_sku,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────

async fn list_materials(
    tenant: TenantId,
    State(state): State<AppState>,
) -> Result<Json<Vec<MaterialResponse>>, AppError> {
    let rows = pt_repo::material::list_by_tenant(&state.pool, tenant.0).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

async fn create_material(
    tenant: TenantId,
    State(state): State<AppState>,
    Json(body): Json<CreateMaterialRequest>,
) -> Result<(StatusCode, Json<MaterialResponse>), AppError> {
    let input = pt_repo::material::CreateMaterial {
        tenant_id: tenant.0,
        name: body.name,
        category: body.category,
        unit: body.unit,
        price_per_unit: body.price_per_unit,
        depth_inches: body.depth_inches,
        extrusion: body.extrusion,
        texture_key: body.texture_key,
        photo_key: body.photo_key,
        supplier_sku: body.supplier_sku,
    };
    let id = pt_repo::material::create(&state.pool, &input).await?;

    // Re-fetch to get server-generated fields (created_at, updated_at)
    let rows = pt_repo::material::list_by_tenant(&state.pool, tenant.0).await?;
    let row = rows
        .into_iter()
        .find(|r| r.id == id)
        .ok_or(AppError::Internal("created material not found".to_string()))?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn update_material(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateMaterialRequest>,
) -> Result<StatusCode, AppError> {
    // Verify material belongs to tenant by listing and checking
    let rows = pt_repo::material::list_by_tenant(&state.pool, tenant.0).await?;
    if !rows.iter().any(|r| r.id == id) {
        return Err(AppError::NotFound);
    }

    let input = pt_repo::material::CreateMaterial {
        tenant_id: tenant.0,
        name: body.name,
        category: body.category,
        unit: body.unit,
        price_per_unit: body.price_per_unit,
        depth_inches: body.depth_inches,
        extrusion: body.extrusion,
        texture_key: body.texture_key,
        photo_key: body.photo_key,
        supplier_sku: body.supplier_sku,
    };
    pt_repo::material::update(&state.pool, id, &input).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_material(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let rows = pt_repo::material::list_by_tenant(&state.pool, tenant.0).await?;
    if !rows.iter().any(|r| r.id == id) {
        return Err(AppError::NotFound);
    }

    pt_repo::material::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
