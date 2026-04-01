//! GeoJSON FeatureCollection conversion for [`Project`].
//!
//! Separate from serde — this produces GeoJSON format (FeatureCollection) for
//! interchange with PostGIS, map frontends, and external tools.

use geojson::{Feature, FeatureCollection, GeoJson, Geometry};
use std::convert::TryInto;

use crate::error::ProjectError;
use crate::project::Project;
use crate::types::{Zone, ZoneId, ZoneType};

impl Project {
    /// Serialize the project as a GeoJSON FeatureCollection.
    ///
    /// Each zone becomes a Feature with geometry and properties (id, zone_type, label).
    /// Project-level metadata is stored in the FeatureCollection's foreign members.
    pub fn to_geojson(&self) -> GeoJson {
        let features: Vec<Feature> = self
            .zones
            .iter()
            .map(|zone| {
                let geom = Geometry::from(&zone.geometry);
                let mut properties = serde_json::Map::new();
                properties.insert("id".to_string(), serde_json::json!(zone.id.0.to_string()));
                properties.insert(
                    "zone_type".to_string(),
                    serde_json::to_value(zone.zone_type).unwrap(),
                );
                if let Some(ref label) = zone.label {
                    properties.insert("label".to_string(), serde_json::json!(label));
                }
                Feature {
                    bbox: None,
                    geometry: Some(geom),
                    id: None,
                    properties: Some(properties),
                    foreign_members: None,
                }
            })
            .collect();

        let mut foreign_members = serde_json::Map::new();
        foreign_members.insert(
            "project_id".to_string(),
            serde_json::json!(self.id.to_string()),
        );
        foreign_members.insert(
            "status".to_string(),
            serde_json::to_value(self.status).unwrap(),
        );
        if let Some(ref scan_ref) = self.scan_ref {
            foreign_members.insert("scan_ref".to_string(), serde_json::json!(scan_ref));
        }
        foreign_members.insert(
            "tiers".to_string(),
            serde_json::to_value(&self.tiers).unwrap(),
        );
        foreign_members.insert(
            "created_at".to_string(),
            serde_json::json!(self.created_at.to_rfc3339()),
        );
        foreign_members.insert(
            "updated_at".to_string(),
            serde_json::json!(self.updated_at.to_rfc3339()),
        );

        GeoJson::FeatureCollection(FeatureCollection {
            bbox: None,
            features,
            foreign_members: Some(foreign_members),
        })
    }

    /// Deserialize a project from a GeoJSON FeatureCollection.
    ///
    /// # Errors
    /// Returns [`ProjectError::GeoJsonConversion`] if the input is not a valid
    /// FeatureCollection or contains malformed zone features.
    pub fn from_geojson(geojson: &GeoJson) -> Result<Self, ProjectError> {
        let fc = match geojson {
            GeoJson::FeatureCollection(fc) => fc,
            _ => {
                return Err(ProjectError::GeoJsonConversion(
                    "expected FeatureCollection".to_string(),
                ))
            }
        };

        let foreign = fc.foreign_members.as_ref().ok_or_else(|| {
            ProjectError::GeoJsonConversion("missing foreign members".to_string())
        })?;

        let project_id: uuid::Uuid = foreign
            .get("project_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProjectError::GeoJsonConversion("missing project_id".to_string()))?
            .parse()
            .map_err(|e| ProjectError::GeoJsonConversion(format!("invalid project_id: {e}")))?;

        let status = foreign
            .get("status")
            .ok_or_else(|| ProjectError::GeoJsonConversion("missing status".to_string()))
            .and_then(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| ProjectError::GeoJsonConversion(format!("invalid status: {e}")))
            })?;

        let scan_ref = foreign
            .get("scan_ref")
            .and_then(|v| v.as_str())
            .map(String::from);

        let tiers = foreign
            .get("tiers")
            .ok_or_else(|| ProjectError::GeoJsonConversion("missing tiers".to_string()))
            .and_then(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| ProjectError::GeoJsonConversion(format!("invalid tiers: {e}")))
            })?;

        let created_at = foreign
            .get("created_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProjectError::GeoJsonConversion("missing created_at".to_string()))
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(|e| {
                        ProjectError::GeoJsonConversion(format!("invalid created_at: {e}"))
                    })
            })?;

        let updated_at = foreign
            .get("updated_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProjectError::GeoJsonConversion("missing updated_at".to_string()))
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(|e| {
                        ProjectError::GeoJsonConversion(format!("invalid updated_at: {e}"))
                    })
            })?;

        let mut zones = Vec::with_capacity(fc.features.len());
        for feature in &fc.features {
            zones.push(zone_from_feature(feature)?);
        }

        Ok(Project {
            id: project_id,
            scan_ref,
            zones,
            tiers,
            status,
            created_at,
            updated_at,
        })
    }
}

fn zone_from_feature(feature: &Feature) -> Result<Zone, ProjectError> {
    let props = feature
        .properties
        .as_ref()
        .ok_or_else(|| ProjectError::GeoJsonConversion("feature missing properties".to_string()))?;

    let id_str = props
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProjectError::GeoJsonConversion("zone missing id".to_string()))?;

    let id = ZoneId(
        id_str
            .parse()
            .map_err(|e| ProjectError::GeoJsonConversion(format!("invalid zone id: {e}")))?,
    );

    let zone_type: ZoneType = props
        .get("zone_type")
        .ok_or_else(|| ProjectError::GeoJsonConversion("zone missing zone_type".to_string()))
        .and_then(|v| {
            serde_json::from_value(v.clone())
                .map_err(|e| ProjectError::GeoJsonConversion(format!("invalid zone_type: {e}")))
        })?;

    let label = props
        .get("label")
        .and_then(|v| v.as_str())
        .map(String::from);

    let geom = feature
        .geometry
        .as_ref()
        .ok_or_else(|| ProjectError::GeoJsonConversion("feature missing geometry".to_string()))?;

    let geo_geom: geo::Geometry<f64> = geom
        .clone()
        .try_into()
        .map_err(|e| ProjectError::GeoJsonConversion(format!("invalid geometry: {e}")))?;

    let polygon = match geo_geom {
        geo::Geometry::Polygon(p) => p,
        other => {
            return Err(ProjectError::GeoJsonConversion(format!(
                "expected Polygon, got {:?}",
                other
            )))
        }
    };

    Ok(Zone {
        id,
        geometry: polygon,
        zone_type,
        label,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ProjectStatus, TierLevel};
    use geo::polygon;

    fn make_test_project() -> Project {
        let mut p = Project::new();
        p.add_zone(Zone {
            id: ZoneId::new(),
            geometry: polygon![
                (x: 0.0, y: 0.0),
                (x: 12.0, y: 0.0),
                (x: 12.0, y: 15.0),
                (x: 0.0, y: 15.0),
            ],
            zone_type: ZoneType::Patio,
            label: Some("Back patio".to_string()),
        })
        .unwrap();
        p.add_zone(Zone {
            id: ZoneId::new(),
            geometry: polygon![
                (x: 20.0, y: 0.0),
                (x: 28.0, y: 0.0),
                (x: 28.0, y: 20.0),
                (x: 20.0, y: 20.0),
            ],
            zone_type: ZoneType::Bed,
            label: Some("Garden bed".to_string()),
        })
        .unwrap();
        p
    }

    #[test]
    fn empty_project_geojson_round_trip() {
        let p = Project::new();
        let geojson = p.to_geojson();
        let back = Project::from_geojson(&geojson).unwrap();

        assert_eq!(p.id, back.id);
        assert_eq!(p.status, back.status);
        assert_eq!(p.zones.len(), back.zones.len());
        assert_eq!(p.tiers.len(), back.tiers.len());
    }

    #[test]
    fn multi_zone_geojson_round_trip() {
        let p = make_test_project();
        let geojson = p.to_geojson();
        let back = Project::from_geojson(&geojson).unwrap();

        assert_eq!(p.id, back.id);
        assert_eq!(p.status, back.status);
        assert_eq!(p.zones.len(), back.zones.len());

        for (orig, restored) in p.zones.iter().zip(back.zones.iter()) {
            assert_eq!(orig.id, restored.id);
            assert_eq!(orig.zone_type, restored.zone_type);
            assert_eq!(orig.label, restored.label);
        }
    }

    #[test]
    fn geojson_geometry_coordinates_preserved() {
        let p = make_test_project();
        let geojson = p.to_geojson();
        let back = Project::from_geojson(&geojson).unwrap();

        // Check first zone coordinates
        let orig_exterior: Vec<_> = p.zones[0].geometry.exterior().points().collect();
        let back_exterior: Vec<_> = back.zones[0].geometry.exterior().points().collect();
        assert_eq!(orig_exterior.len(), back_exterior.len());
        for (a, b) in orig_exterior.iter().zip(back_exterior.iter()) {
            assert_eq!(a.x(), b.x());
            assert_eq!(a.y(), b.y());
        }
    }

    #[test]
    fn geojson_zone_properties_preserved() {
        let p = make_test_project();
        let geojson = p.to_geojson();
        let back = Project::from_geojson(&geojson).unwrap();

        assert_eq!(back.zones[0].zone_type, ZoneType::Patio);
        assert_eq!(back.zones[0].label.as_deref(), Some("Back patio"));
        assert_eq!(back.zones[1].zone_type, ZoneType::Bed);
        assert_eq!(back.zones[1].label.as_deref(), Some("Garden bed"));
    }

    #[test]
    fn geojson_status_and_tiers_preserved() {
        let mut p = make_test_project();
        p.transition_to(ProjectStatus::Quoted).unwrap();
        let geojson = p.to_geojson();
        let back = Project::from_geojson(&geojson).unwrap();

        assert_eq!(back.status, ProjectStatus::Quoted);
        assert_eq!(back.tiers.len(), 3);
        assert_eq!(back.tier(TierLevel::Good).level, TierLevel::Good);
        assert_eq!(back.tier(TierLevel::Better).level, TierLevel::Better);
        assert_eq!(back.tier(TierLevel::Best).level, TierLevel::Best);
    }

    #[test]
    fn geojson_invalid_input() {
        let geojson = GeoJson::Geometry(Geometry::from(&polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
        ]));
        let err = Project::from_geojson(&geojson).unwrap_err();
        assert!(matches!(err, ProjectError::GeoJsonConversion(_)));
    }
}
