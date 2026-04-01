//! Core project domain types.

use geo::Polygon;
use pt_materials::MaterialId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a zone within a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneId(pub Uuid);

impl ZoneId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ZoneId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ZoneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of landscape zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneType {
    Bed,
    Patio,
    Path,
    Lawn,
    Wall,
    Edging,
}

/// A typed polygon zone within a project.
///
/// Each zone has geometry (a polygon in feet), a type that determines how it's
/// treated in the quote engine and 3D renderer, and an optional human label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: ZoneId,
    #[serde(with = "crate::serde_helpers::geojson_polygon")]
    pub geometry: Polygon<f64>,
    pub zone_type: ZoneType,
    pub label: Option<String>,
}

/// Quality tier level (good / better / best).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TierLevel {
    Good,
    Better,
    Best,
}

/// Per-assignment overrides that differ from the material's catalog defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentOverrides {
    pub price_override: Option<Decimal>,
    pub depth_override_inches: Option<f64>,
}

/// Assignment of a material to a zone within a tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialAssignment {
    pub zone_id: ZoneId,
    pub material_id: MaterialId,
    pub overrides: Option<AssignmentOverrides>,
}

/// A tier (good / better / best) containing material assignments for each zone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tier {
    pub level: TierLevel,
    pub assignments: Vec<MaterialAssignment>,
}

/// Lifecycle status of a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Draft,
    Quoted,
    Approved,
    Complete,
}

impl ProjectStatus {
    /// Returns whether transitioning from `self` to `to` is valid.
    pub fn can_transition_to(self, to: ProjectStatus) -> bool {
        matches!(
            (self, to),
            // Forward transitions
            (ProjectStatus::Draft, ProjectStatus::Quoted)
                | (ProjectStatus::Quoted, ProjectStatus::Approved)
                | (ProjectStatus::Approved, ProjectStatus::Complete)
                // Reset to draft from any state
                | (_, ProjectStatus::Draft)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::polygon;

    #[test]
    fn zone_id_uniqueness() {
        let a = ZoneId::new();
        let b = ZoneId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn zone_type_serde_round_trip() {
        for zt in [
            ZoneType::Bed,
            ZoneType::Patio,
            ZoneType::Path,
            ZoneType::Lawn,
            ZoneType::Wall,
            ZoneType::Edging,
        ] {
            let json = serde_json::to_string(&zt).unwrap();
            let back: ZoneType = serde_json::from_str(&json).unwrap();
            assert_eq!(zt, back);
        }
    }

    #[test]
    fn zone_serde_round_trip() {
        let zone = Zone {
            id: ZoneId::new(),
            geometry: polygon![
                (x: 0.0, y: 0.0),
                (x: 12.0, y: 0.0),
                (x: 12.0, y: 15.0),
                (x: 0.0, y: 15.0),
            ],
            zone_type: ZoneType::Patio,
            label: Some("Back patio".to_string()),
        };
        let json = serde_json::to_string(&zone).unwrap();
        let back: Zone = serde_json::from_str(&json).unwrap();
        assert_eq!(zone, back);
    }

    #[test]
    fn tier_level_serde_round_trip() {
        for level in [TierLevel::Good, TierLevel::Better, TierLevel::Best] {
            let json = serde_json::to_string(&level).unwrap();
            let back: TierLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, back);
        }
    }

    #[test]
    fn project_status_serde_round_trip() {
        for status in [
            ProjectStatus::Draft,
            ProjectStatus::Quoted,
            ProjectStatus::Approved,
            ProjectStatus::Complete,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: ProjectStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn status_valid_forward_transitions() {
        assert!(ProjectStatus::Draft.can_transition_to(ProjectStatus::Quoted));
        assert!(ProjectStatus::Quoted.can_transition_to(ProjectStatus::Approved));
        assert!(ProjectStatus::Approved.can_transition_to(ProjectStatus::Complete));
    }

    #[test]
    fn status_reset_to_draft_always_valid() {
        assert!(ProjectStatus::Quoted.can_transition_to(ProjectStatus::Draft));
        assert!(ProjectStatus::Approved.can_transition_to(ProjectStatus::Draft));
        assert!(ProjectStatus::Complete.can_transition_to(ProjectStatus::Draft));
    }

    #[test]
    fn status_invalid_transitions() {
        assert!(!ProjectStatus::Draft.can_transition_to(ProjectStatus::Approved));
        assert!(!ProjectStatus::Draft.can_transition_to(ProjectStatus::Complete));
        assert!(!ProjectStatus::Quoted.can_transition_to(ProjectStatus::Complete));
    }
}
