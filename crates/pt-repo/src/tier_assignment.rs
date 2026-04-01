//! Tier assignment repository — links materials to zones within a tier.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::convert::{parse_tier_level, tier_level_to_str};
use crate::error::RepoError;
use pt_project::TierLevel;

/// Database row for the tier_assignments table.
#[derive(Debug, Clone)]
pub struct TierAssignmentRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub tier: TierLevel,
    pub zone_id: Uuid,
    pub material_id: Uuid,
    pub overrides: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for setting a single assignment within a tier.
#[derive(Debug)]
pub struct SetAssignment {
    pub zone_id: Uuid,
    pub material_id: Uuid,
    pub overrides: Option<serde_json::Value>,
}

/// Get all assignments for a project and tier.
pub async fn get_by_project_and_tier(
    pool: &PgPool,
    project_id: Uuid,
    tier: TierLevel,
) -> Result<Vec<TierAssignmentRow>, RepoError> {
    let rows = sqlx::query_as::<_, TierAssignmentRowSqlx>(
        "SELECT id, project_id, tier, zone_id, material_id, overrides,
                created_at, updated_at
         FROM tier_assignments
         WHERE project_id = $1 AND tier = $2
         ORDER BY created_at",
    )
    .bind(project_id)
    .bind(tier_level_to_str(tier))
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

/// Replace all assignments for a project + tier in one transaction.
/// Deletes existing assignments, then inserts the new set.
pub async fn set_assignments(
    pool: &PgPool,
    project_id: Uuid,
    tier: TierLevel,
    assignments: &[SetAssignment],
) -> Result<(), RepoError> {
    let tier_str = tier_level_to_str(tier);
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM tier_assignments WHERE project_id = $1 AND tier = $2")
        .bind(project_id)
        .bind(tier_str)
        .execute(&mut *tx)
        .await?;

    for a in assignments {
        sqlx::query(
            "INSERT INTO tier_assignments (project_id, tier, zone_id, material_id, overrides)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(project_id)
        .bind(tier_str)
        .bind(a.zone_id)
        .bind(a.material_id)
        .bind(&a.overrides)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

// ── Internal sqlx row type ─────────────────────────────────────

#[derive(sqlx::FromRow)]
struct TierAssignmentRowSqlx {
    id: Uuid,
    project_id: Uuid,
    tier: String,
    zone_id: Uuid,
    material_id: Uuid,
    overrides: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TierAssignmentRowSqlx> for TierAssignmentRow {
    type Error = RepoError;

    fn try_from(r: TierAssignmentRowSqlx) -> Result<Self, Self::Error> {
        Ok(Self {
            id: r.id,
            project_id: r.project_id,
            tier: parse_tier_level(&r.tier)?,
            zone_id: r.zone_id,
            material_id: r.material_id,
            overrides: r.overrides,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}
