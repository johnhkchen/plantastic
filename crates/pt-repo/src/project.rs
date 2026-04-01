//! Project repository — CRUD operations for the projects table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::convert::{parse_project_status, project_status_to_str};
use crate::error::RepoError;
use pt_project::ProjectStatus;

/// Database row for the projects table.
#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub client_name: Option<String>,
    pub client_email: Option<String>,
    pub address: Option<String>,
    pub scan_ref: Option<serde_json::Value>,
    pub baseline: Option<serde_json::Value>,
    pub status: ProjectStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new project.
#[derive(Debug)]
pub struct CreateProject {
    pub tenant_id: Uuid,
    pub client_name: Option<String>,
    pub client_email: Option<String>,
    pub address: Option<String>,
}

/// Create a project. Returns the new project's ID.
pub async fn create(pool: &PgPool, input: &CreateProject) -> Result<Uuid, RepoError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO projects (tenant_id, client_name, client_email, address)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(input.tenant_id)
    .bind(&input.client_name)
    .bind(&input.client_email)
    .bind(&input.address)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Fetch a project by ID.
pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<ProjectRow, RepoError> {
    let row = sqlx::query_as::<_, ProjectRowSqlx>(
        "SELECT id, tenant_id, client_name, client_email, address,
                scan_ref, baseline, status, created_at, updated_at
         FROM projects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(RepoError::NotFound)?;

    row.try_into()
}

/// List all projects for a tenant.
pub async fn list_by_tenant(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<ProjectRow>, RepoError> {
    let rows = sqlx::query_as::<_, ProjectRowSqlx>(
        "SELECT id, tenant_id, client_name, client_email, address,
                scan_ref, baseline, status, created_at, updated_at
         FROM projects WHERE tenant_id = $1
         ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

/// Update a project's status. Validates the transition in the application layer.
pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    new_status: ProjectStatus,
) -> Result<(), RepoError> {
    // Fetch current status to validate transition
    let current_row = sqlx::query_scalar::<_, String>("SELECT status FROM projects WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(RepoError::NotFound)?;

    let current = parse_project_status(&current_row)?;
    if !current.can_transition_to(new_status) {
        return Err(RepoError::Conflict(format!(
            "invalid status transition: {current:?} → {new_status:?}"
        )));
    }

    sqlx::query("UPDATE projects SET status = $1, updated_at = now() WHERE id = $2")
        .bind(project_status_to_str(new_status))
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Set the satellite baseline for a project.
pub async fn set_baseline(
    pool: &PgPool,
    id: Uuid,
    baseline: &serde_json::Value,
) -> Result<(), RepoError> {
    let result = sqlx::query("UPDATE projects SET baseline = $1, updated_at = now() WHERE id = $2")
        .bind(baseline)
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

/// Set the scan artifact references for a project.
pub async fn set_scan_ref(
    pool: &PgPool,
    id: Uuid,
    scan_ref: &serde_json::Value,
) -> Result<(), RepoError> {
    let result = sqlx::query("UPDATE projects SET scan_ref = $1, updated_at = now() WHERE id = $2")
        .bind(scan_ref)
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

/// Delete a project by ID. Zones and tier assignments cascade.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), RepoError> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
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
struct ProjectRowSqlx {
    id: Uuid,
    tenant_id: Uuid,
    client_name: Option<String>,
    client_email: Option<String>,
    address: Option<String>,
    scan_ref: Option<serde_json::Value>,
    baseline: Option<serde_json::Value>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ProjectRowSqlx> for ProjectRow {
    type Error = RepoError;

    fn try_from(r: ProjectRowSqlx) -> Result<Self, Self::Error> {
        Ok(Self {
            id: r.id,
            tenant_id: r.tenant_id,
            client_name: r.client_name,
            client_email: r.client_email,
            address: r.address,
            scan_ref: r.scan_ref,
            baseline: r.baseline,
            status: parse_project_status(&r.status)?,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}
