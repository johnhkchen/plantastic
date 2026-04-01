//! Material repository — CRUD operations for the tenant-scoped material catalog.

use chrono::{DateTime, Utc};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::convert::{category_to_str, parse_material_category, parse_unit, unit_to_str};
use crate::error::RepoError;
use pt_materials::{ExtrusionBehavior, MaterialCategory, Unit};

/// Database row for the materials table.
#[derive(Debug, Clone)]
pub struct MaterialRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub category: MaterialCategory,
    pub unit: Unit,
    pub price_per_unit: Decimal,
    pub depth_inches: Option<f64>,
    pub extrusion: ExtrusionBehavior,
    pub texture_key: Option<String>,
    pub photo_key: Option<String>,
    pub supplier_sku: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating or updating a material.
#[derive(Debug)]
pub struct CreateMaterial {
    pub tenant_id: Uuid,
    pub name: String,
    pub category: MaterialCategory,
    pub unit: Unit,
    pub price_per_unit: Decimal,
    pub depth_inches: Option<f64>,
    pub extrusion: ExtrusionBehavior,
    pub texture_key: Option<String>,
    pub photo_key: Option<String>,
    pub supplier_sku: Option<String>,
}

/// List all materials for a tenant.
pub async fn list_by_tenant(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<MaterialRow>, RepoError> {
    let rows = sqlx::query_as::<_, MaterialRowSqlx>(
        "SELECT id, tenant_id, name, category, unit, price_per_unit,
                depth_inches, extrusion, texture_key, photo_key, supplier_sku,
                created_at, updated_at
         FROM materials WHERE tenant_id = $1
         ORDER BY name",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

/// Create a material. Returns the new material's ID.
pub async fn create(pool: &PgPool, input: &CreateMaterial) -> Result<Uuid, RepoError> {
    let extrusion_json =
        serde_json::to_value(&input.extrusion).map_err(|e| RepoError::Conversion(e.to_string()))?;

    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO materials
            (tenant_id, name, category, unit, price_per_unit, depth_inches,
             extrusion, texture_key, photo_key, supplier_sku)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING id",
    )
    .bind(input.tenant_id)
    .bind(&input.name)
    .bind(category_to_str(input.category))
    .bind(unit_to_str(input.unit))
    .bind(input.price_per_unit)
    .bind(input.depth_inches.and_then(Decimal::from_f64))
    .bind(&extrusion_json)
    .bind(&input.texture_key)
    .bind(&input.photo_key)
    .bind(&input.supplier_sku)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Update a material's fields.
pub async fn update(pool: &PgPool, id: Uuid, input: &CreateMaterial) -> Result<(), RepoError> {
    let extrusion_json =
        serde_json::to_value(&input.extrusion).map_err(|e| RepoError::Conversion(e.to_string()))?;

    let result = sqlx::query(
        "UPDATE materials
         SET name = $1, category = $2, unit = $3, price_per_unit = $4,
             depth_inches = $5, extrusion = $6, texture_key = $7,
             photo_key = $8, supplier_sku = $9, updated_at = now()
         WHERE id = $10",
    )
    .bind(&input.name)
    .bind(category_to_str(input.category))
    .bind(unit_to_str(input.unit))
    .bind(input.price_per_unit)
    .bind(input.depth_inches.and_then(Decimal::from_f64))
    .bind(&extrusion_json)
    .bind(&input.texture_key)
    .bind(&input.photo_key)
    .bind(&input.supplier_sku)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

/// Delete a material by ID.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), RepoError> {
    let result = sqlx::query("DELETE FROM materials WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

// ── Internal sqlx row type ─────────────────────────────────────

#[derive(sqlx::FromRow)]
struct MaterialRowSqlx {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    category: String,
    unit: String,
    price_per_unit: Decimal,
    /// NUMERIC in Postgres maps to Decimal in sqlx; converted to f64 for domain.
    depth_inches: Option<Decimal>,
    extrusion: serde_json::Value,
    texture_key: Option<String>,
    photo_key: Option<String>,
    supplier_sku: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<MaterialRowSqlx> for MaterialRow {
    type Error = RepoError;

    fn try_from(r: MaterialRowSqlx) -> Result<Self, Self::Error> {
        let extrusion: ExtrusionBehavior = serde_json::from_value(r.extrusion)
            .map_err(|e| RepoError::Conversion(format!("invalid extrusion JSONB: {e}")))?;

        Ok(Self {
            id: r.id,
            tenant_id: r.tenant_id,
            name: r.name,
            category: parse_material_category(&r.category)?,
            unit: parse_unit(&r.unit)?,
            price_per_unit: r.price_per_unit,
            depth_inches: r.depth_inches.and_then(|d| d.to_f64()),
            extrusion,
            texture_key: r.texture_key,
            photo_key: r.photo_key,
            supplier_sku: r.supplier_sku,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}
