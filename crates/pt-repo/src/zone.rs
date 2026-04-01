//! Zone repository — CRUD operations with PostGIS geometry conversion.

use chrono::{DateTime, Utc};
use geo::Polygon;
use sqlx::PgPool;
use uuid::Uuid;

use crate::convert::{
    geojson_string_to_polygon, parse_zone_type, polygon_to_geojson_string, zone_type_to_str,
};
use crate::error::RepoError;
use pt_project::ZoneType;

/// Database row for the zones table with geometry as a parsed polygon.
#[derive(Debug, Clone)]
pub struct ZoneRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub geometry: Polygon<f64>,
    pub zone_type: ZoneType,
    pub label: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a zone.
#[derive(Debug)]
pub struct CreateZone {
    pub project_id: Uuid,
    pub geometry: Polygon<f64>,
    pub zone_type: ZoneType,
    pub label: Option<String>,
    pub sort_order: i32,
}

/// List all zones for a project, ordered by sort_order.
pub async fn list_by_project(pool: &PgPool, project_id: Uuid) -> Result<Vec<ZoneRow>, RepoError> {
    let rows = sqlx::query_as::<_, ZoneRowSqlx>(
        "SELECT id, project_id, ST_AsGeoJSON(geometry)::text as geometry_geojson,
                zone_type, label, sort_order, created_at, updated_at
         FROM zones WHERE project_id = $1
         ORDER BY sort_order, created_at",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

/// Add a single zone. Returns the new zone's ID.
pub async fn add(pool: &PgPool, input: &CreateZone) -> Result<Uuid, RepoError> {
    let geojson = polygon_to_geojson_string(&input.geometry);
    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO zones (project_id, geometry, zone_type, label, sort_order)
         VALUES ($1, ST_GeomFromGeoJSON($2), $3, $4, $5)
         RETURNING id",
    )
    .bind(input.project_id)
    .bind(&geojson)
    .bind(zone_type_to_str(input.zone_type))
    .bind(&input.label)
    .bind(input.sort_order)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Update a zone's geometry, type, label, and sort order.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    geometry: &Polygon<f64>,
    zone_type: ZoneType,
    label: Option<&str>,
    sort_order: i32,
) -> Result<(), RepoError> {
    let geojson = polygon_to_geojson_string(geometry);
    let result = sqlx::query(
        "UPDATE zones
         SET geometry = ST_GeomFromGeoJSON($1), zone_type = $2, label = $3,
             sort_order = $4, updated_at = now()
         WHERE id = $5",
    )
    .bind(&geojson)
    .bind(zone_type_to_str(zone_type))
    .bind(label)
    .bind(sort_order)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

/// Delete a zone by ID.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), RepoError> {
    let result = sqlx::query("DELETE FROM zones WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

/// Replace all zones for a project in a single transaction.
/// Deletes existing zones and inserts the new set. Returns the new zone IDs.
pub async fn bulk_upsert(
    pool: &PgPool,
    project_id: Uuid,
    zones: &[CreateZone],
) -> Result<Vec<Uuid>, RepoError> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM zones WHERE project_id = $1")
        .bind(project_id)
        .execute(&mut *tx)
        .await?;

    let mut ids = Vec::with_capacity(zones.len());
    for zone in zones {
        let geojson = polygon_to_geojson_string(&zone.geometry);
        let id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO zones (project_id, geometry, zone_type, label, sort_order)
             VALUES ($1, ST_GeomFromGeoJSON($2), $3, $4, $5)
             RETURNING id",
        )
        .bind(project_id)
        .bind(&geojson)
        .bind(zone_type_to_str(zone.zone_type))
        .bind(&zone.label)
        .bind(zone.sort_order)
        .fetch_one(&mut *tx)
        .await?;
        ids.push(id);
    }

    tx.commit().await?;
    Ok(ids)
}

// ── Internal sqlx row type ─────────────────────────────────────

#[derive(sqlx::FromRow)]
struct ZoneRowSqlx {
    id: Uuid,
    project_id: Uuid,
    geometry_geojson: String,
    zone_type: String,
    label: Option<String>,
    sort_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ZoneRowSqlx> for ZoneRow {
    type Error = RepoError;

    fn try_from(r: ZoneRowSqlx) -> Result<Self, Self::Error> {
        Ok(Self {
            id: r.id,
            project_id: r.project_id,
            geometry: geojson_string_to_polygon(&r.geometry_geojson)?,
            zone_type: parse_zone_type(&r.zone_type)?,
            label: r.label,
            sort_order: r.sort_order,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}
