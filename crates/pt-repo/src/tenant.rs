//! Tenant repository — minimal CRUD for multi-tenancy support.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::RepoError;

/// Database row for the tenants table.
#[derive(Debug, Clone)]
pub struct TenantRow {
    pub id: Uuid,
    pub name: String,
    pub logo_url: Option<String>,
    pub brand_color: Option<String>,
    pub contact: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create a tenant with a name. Returns the new tenant's ID.
pub async fn create(pool: &PgPool, name: &str) -> Result<Uuid, RepoError> {
    let row = sqlx::query_scalar::<_, Uuid>("INSERT INTO tenants (name) VALUES ($1) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(row)
}

/// Fetch a tenant by ID.
pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<TenantRow, RepoError> {
    let row = sqlx::query_as::<_, TenantRowSqlx>(
        "SELECT id, name, logo_url, brand_color, contact, created_at, updated_at
         FROM tenants WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(RepoError::NotFound)?;

    Ok(row.into())
}

// Internal sqlx-friendly row type (derives FromRow).
#[derive(sqlx::FromRow)]
struct TenantRowSqlx {
    id: Uuid,
    name: String,
    logo_url: Option<String>,
    brand_color: Option<String>,
    contact: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TenantRowSqlx> for TenantRow {
    fn from(r: TenantRowSqlx) -> Self {
        Self {
            id: r.id,
            name: r.name,
            logo_url: r.logo_url,
            brand_color: r.brand_color,
            contact: r.contact,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
