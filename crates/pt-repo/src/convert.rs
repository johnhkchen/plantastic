//! Converters between database string/JSONB representations and domain types.

use geo::Polygon;
use geojson::Geometry;
use pt_materials::{MaterialCategory, Unit};
use pt_project::{ProjectStatus, TierLevel, ZoneType};
use std::convert::TryInto;

use crate::error::RepoError;

// ── ProjectStatus ──────────────────────────────────────────────

pub fn project_status_to_str(s: ProjectStatus) -> &'static str {
    match s {
        ProjectStatus::Draft => "draft",
        ProjectStatus::Quoted => "quoted",
        ProjectStatus::Approved => "approved",
        ProjectStatus::Complete => "complete",
    }
}

pub fn parse_project_status(s: &str) -> Result<ProjectStatus, RepoError> {
    match s {
        "draft" => Ok(ProjectStatus::Draft),
        "quoted" => Ok(ProjectStatus::Quoted),
        "approved" => Ok(ProjectStatus::Approved),
        "complete" => Ok(ProjectStatus::Complete),
        other => Err(RepoError::Conversion(format!(
            "unknown project status: {other}"
        ))),
    }
}

// ── ZoneType ───────────────────────────────────────────────────

pub fn zone_type_to_str(t: ZoneType) -> &'static str {
    match t {
        ZoneType::Bed => "bed",
        ZoneType::Patio => "patio",
        ZoneType::Path => "path",
        ZoneType::Lawn => "lawn",
        ZoneType::Wall => "wall",
        ZoneType::Edging => "edging",
    }
}

pub fn parse_zone_type(s: &str) -> Result<ZoneType, RepoError> {
    match s {
        "bed" => Ok(ZoneType::Bed),
        "patio" => Ok(ZoneType::Patio),
        "path" => Ok(ZoneType::Path),
        "lawn" => Ok(ZoneType::Lawn),
        "wall" => Ok(ZoneType::Wall),
        "edging" => Ok(ZoneType::Edging),
        other => Err(RepoError::Conversion(format!("unknown zone type: {other}"))),
    }
}

// ── TierLevel ──────────────────────────────────────────────────

pub fn tier_level_to_str(t: TierLevel) -> &'static str {
    match t {
        TierLevel::Good => "good",
        TierLevel::Better => "better",
        TierLevel::Best => "best",
    }
}

pub fn parse_tier_level(s: &str) -> Result<TierLevel, RepoError> {
    match s {
        "good" => Ok(TierLevel::Good),
        "better" => Ok(TierLevel::Better),
        "best" => Ok(TierLevel::Best),
        other => Err(RepoError::Conversion(format!(
            "unknown tier level: {other}"
        ))),
    }
}

// ── MaterialCategory ───────────────────────────────────────────

pub fn category_to_str(c: MaterialCategory) -> &'static str {
    match c {
        MaterialCategory::Hardscape => "hardscape",
        MaterialCategory::Softscape => "softscape",
        MaterialCategory::Edging => "edging",
        MaterialCategory::Fill => "fill",
    }
}

pub fn parse_material_category(s: &str) -> Result<MaterialCategory, RepoError> {
    match s {
        "hardscape" => Ok(MaterialCategory::Hardscape),
        "softscape" => Ok(MaterialCategory::Softscape),
        "edging" => Ok(MaterialCategory::Edging),
        "fill" => Ok(MaterialCategory::Fill),
        other => Err(RepoError::Conversion(format!(
            "unknown material category: {other}"
        ))),
    }
}

// ── Unit ───────────────────────────────────────────────────────

pub fn unit_to_str(u: Unit) -> &'static str {
    match u {
        Unit::SqFt => "sq_ft",
        Unit::CuYd => "cu_yd",
        Unit::LinearFt => "linear_ft",
        Unit::Each => "each",
    }
}

pub fn parse_unit(s: &str) -> Result<Unit, RepoError> {
    match s {
        "sq_ft" => Ok(Unit::SqFt),
        "cu_yd" => Ok(Unit::CuYd),
        "linear_ft" => Ok(Unit::LinearFt),
        "each" => Ok(Unit::Each),
        other => Err(RepoError::Conversion(format!("unknown unit: {other}"))),
    }
}

// ── Geometry ───────────────────────────────────────────────────

/// Convert a `geo::Polygon<f64>` to a GeoJSON string for `ST_GeomFromGeoJSON`.
pub fn polygon_to_geojson_string(p: &Polygon<f64>) -> String {
    let geom = Geometry::from(p);
    serde_json::to_string(&geom).expect("geometry serialization should not fail")
}

/// Parse a GeoJSON string (from `ST_AsGeoJSON`) back to `geo::Polygon<f64>`.
pub fn geojson_string_to_polygon(s: &str) -> Result<Polygon<f64>, RepoError> {
    let geom: Geometry = serde_json::from_str(s)
        .map_err(|e| RepoError::Conversion(format!("invalid GeoJSON: {e}")))?;

    let geo_geom: geo::Geometry<f64> = geom
        .try_into()
        .map_err(|e| RepoError::Conversion(format!("geometry conversion failed: {e}")))?;

    match geo_geom {
        geo::Geometry::Polygon(p) => Ok(p),
        other => Err(RepoError::Conversion(format!(
            "expected Polygon, got {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::polygon;

    #[test]
    fn project_status_round_trip() {
        for status in [
            ProjectStatus::Draft,
            ProjectStatus::Quoted,
            ProjectStatus::Approved,
            ProjectStatus::Complete,
        ] {
            let s = project_status_to_str(status);
            assert_eq!(parse_project_status(s).unwrap(), status);
        }
    }

    #[test]
    fn zone_type_round_trip() {
        for zt in [
            ZoneType::Bed,
            ZoneType::Patio,
            ZoneType::Path,
            ZoneType::Lawn,
            ZoneType::Wall,
            ZoneType::Edging,
        ] {
            let s = zone_type_to_str(zt);
            assert_eq!(parse_zone_type(s).unwrap(), zt);
        }
    }

    #[test]
    fn tier_level_round_trip() {
        for level in [TierLevel::Good, TierLevel::Better, TierLevel::Best] {
            let s = tier_level_to_str(level);
            assert_eq!(parse_tier_level(s).unwrap(), level);
        }
    }

    #[test]
    fn material_category_round_trip() {
        for cat in [
            MaterialCategory::Hardscape,
            MaterialCategory::Softscape,
            MaterialCategory::Edging,
            MaterialCategory::Fill,
        ] {
            let s = category_to_str(cat);
            assert_eq!(parse_material_category(s).unwrap(), cat);
        }
    }

    #[test]
    fn unit_round_trip() {
        for unit in [Unit::SqFt, Unit::CuYd, Unit::LinearFt, Unit::Each] {
            let s = unit_to_str(unit);
            assert_eq!(parse_unit(s).unwrap(), unit);
        }
    }

    #[test]
    fn invalid_enum_strings() {
        assert!(parse_project_status("bogus").is_err());
        assert!(parse_zone_type("bogus").is_err());
        assert!(parse_tier_level("bogus").is_err());
        assert!(parse_material_category("bogus").is_err());
        assert!(parse_unit("bogus").is_err());
    }

    #[test]
    fn polygon_geojson_round_trip() {
        let poly = polygon![
            (x: -122.4194, y: 37.7749),
            (x: -122.4180, y: 37.7749),
            (x: -122.4180, y: 37.7760),
            (x: -122.4194, y: 37.7760),
        ];
        let json = polygon_to_geojson_string(&poly);
        let back = geojson_string_to_polygon(&json).unwrap();

        let orig_pts: Vec<_> = poly.exterior().points().collect();
        let back_pts: Vec<_> = back.exterior().points().collect();
        assert_eq!(orig_pts.len(), back_pts.len());
        for (a, b) in orig_pts.iter().zip(back_pts.iter()) {
            assert!((a.x() - b.x()).abs() < 1e-10);
            assert!((a.y() - b.y()).abs() < 1e-10);
        }
    }

    #[test]
    fn invalid_geojson_string() {
        assert!(geojson_string_to_polygon("not json").is_err());
    }
}
