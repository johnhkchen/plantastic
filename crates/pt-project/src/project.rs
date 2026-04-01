//! Project aggregate root with zone CRUD and status transitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ProjectError;
use crate::types::{ProjectStatus, Tier, TierLevel, Zone, ZoneId};

/// A landscaping project — the single source of truth for the design.
///
/// Contains zones (typed polygons), three tiers of material assignments,
/// and a lifecycle status. Every other component reads from or writes to
/// this model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub scan_ref: Option<String>,
    pub zones: Vec<Zone>,
    pub tiers: Vec<Tier>,
    pub status: ProjectStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

impl Project {
    /// Create a new empty draft project with three default tiers.
    pub fn new() -> Self {
        Self::with_id(Uuid::new_v4())
    }

    /// Create a new empty draft project with a specific ID.
    pub fn with_id(id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id,
            scan_ref: None,
            zones: Vec::new(),
            tiers: vec![
                Tier {
                    level: TierLevel::Good,
                    assignments: Vec::new(),
                },
                Tier {
                    level: TierLevel::Better,
                    assignments: Vec::new(),
                },
                Tier {
                    level: TierLevel::Best,
                    assignments: Vec::new(),
                },
            ],
            status: ProjectStatus::Draft,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a zone to the project.
    ///
    /// # Errors
    /// Returns [`ProjectError::DuplicateZone`] if a zone with the same ID already exists.
    pub fn add_zone(&mut self, zone: Zone) -> Result<(), ProjectError> {
        if self.zones.iter().any(|z| z.id == zone.id) {
            return Err(ProjectError::DuplicateZone(zone.id));
        }
        self.zones.push(zone);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove a zone by ID. Returns the removed zone.
    ///
    /// # Errors
    /// Returns [`ProjectError::ZoneNotFound`] if no zone with that ID exists.
    pub fn remove_zone(&mut self, id: ZoneId) -> Result<Zone, ProjectError> {
        let pos = self
            .zones
            .iter()
            .position(|z| z.id == id)
            .ok_or(ProjectError::ZoneNotFound(id))?;
        let zone = self.zones.remove(pos);
        self.updated_at = Utc::now();
        Ok(zone)
    }

    /// Get a reference to a zone by ID.
    pub fn get_zone(&self, id: ZoneId) -> Option<&Zone> {
        self.zones.iter().find(|z| z.id == id)
    }

    /// Get a mutable reference to a zone by ID.
    pub fn get_zone_mut(&mut self, id: ZoneId) -> Option<&mut Zone> {
        self.zones.iter_mut().find(|z| z.id == id)
    }

    /// Transition the project to a new status.
    ///
    /// Valid transitions: Draft→Quoted, Quoted→Approved, Approved→Complete, Any→Draft.
    ///
    /// # Errors
    /// Returns [`ProjectError::InvalidStatusTransition`] if the transition is not valid.
    pub fn transition_to(&mut self, status: ProjectStatus) -> Result<(), ProjectError> {
        if !self.status.can_transition_to(status) {
            return Err(ProjectError::InvalidStatusTransition {
                from: self.status,
                to: status,
            });
        }
        self.status = status;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get a reference to a specific tier.
    pub fn tier(&self, level: TierLevel) -> &Tier {
        self.tiers
            .iter()
            .find(|t| t.level == level)
            .expect("Project must have all three tiers")
    }

    /// Get a mutable reference to a specific tier.
    pub fn tier_mut(&mut self, level: TierLevel) -> &mut Tier {
        self.tiers
            .iter_mut()
            .find(|t| t.level == level)
            .expect("Project must have all three tiers")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ZoneType;
    use geo::polygon;
    use pt_materials::MaterialId;

    fn make_zone(zone_type: ZoneType, label: Option<&str>) -> Zone {
        Zone {
            id: ZoneId::new(),
            geometry: polygon![
                (x: 0.0, y: 0.0),
                (x: 10.0, y: 0.0),
                (x: 10.0, y: 10.0),
                (x: 0.0, y: 10.0),
            ],
            zone_type,
            label: label.map(String::from),
        }
    }

    #[test]
    fn new_project_has_three_tiers_and_draft_status() {
        let p = Project::new();
        assert_eq!(p.status, ProjectStatus::Draft);
        assert_eq!(p.tiers.len(), 3);
        assert_eq!(p.tier(TierLevel::Good).level, TierLevel::Good);
        assert_eq!(p.tier(TierLevel::Better).level, TierLevel::Better);
        assert_eq!(p.tier(TierLevel::Best).level, TierLevel::Best);
        assert!(p.zones.is_empty());
    }

    #[test]
    fn add_and_get_zone() {
        let mut p = Project::new();
        let zone = make_zone(ZoneType::Patio, Some("Back patio"));
        let id = zone.id;
        p.add_zone(zone).unwrap();

        let got = p.get_zone(id).unwrap();
        assert_eq!(got.zone_type, ZoneType::Patio);
        assert_eq!(got.label.as_deref(), Some("Back patio"));
    }

    #[test]
    fn add_duplicate_zone_errors() {
        let mut p = Project::new();
        let zone = make_zone(ZoneType::Bed, None);
        let id = zone.id;
        p.add_zone(zone.clone()).unwrap();

        let err = p.add_zone(zone).unwrap_err();
        assert_eq!(err, ProjectError::DuplicateZone(id));
    }

    #[test]
    fn remove_zone_returns_it() {
        let mut p = Project::new();
        let zone = make_zone(ZoneType::Path, Some("Front walk"));
        let id = zone.id;
        p.add_zone(zone).unwrap();

        let removed = p.remove_zone(id).unwrap();
        assert_eq!(removed.label.as_deref(), Some("Front walk"));
        assert!(p.get_zone(id).is_none());
    }

    #[test]
    fn remove_nonexistent_zone_errors() {
        let mut p = Project::new();
        let id = ZoneId::new();
        let err = p.remove_zone(id).unwrap_err();
        assert_eq!(err, ProjectError::ZoneNotFound(id));
    }

    #[test]
    fn get_zone_mut_updates() {
        let mut p = Project::new();
        let zone = make_zone(ZoneType::Lawn, None);
        let id = zone.id;
        p.add_zone(zone).unwrap();

        let z = p.get_zone_mut(id).unwrap();
        z.label = Some("Side lawn".to_string());

        assert_eq!(p.get_zone(id).unwrap().label.as_deref(), Some("Side lawn"));
    }

    #[test]
    fn valid_status_transitions() {
        let mut p = Project::new();
        assert_eq!(p.status, ProjectStatus::Draft);

        p.transition_to(ProjectStatus::Quoted).unwrap();
        assert_eq!(p.status, ProjectStatus::Quoted);

        p.transition_to(ProjectStatus::Approved).unwrap();
        assert_eq!(p.status, ProjectStatus::Approved);

        p.transition_to(ProjectStatus::Complete).unwrap();
        assert_eq!(p.status, ProjectStatus::Complete);
    }

    #[test]
    fn reset_to_draft_from_any() {
        let mut p = Project::new();
        p.transition_to(ProjectStatus::Quoted).unwrap();
        p.transition_to(ProjectStatus::Draft).unwrap();
        assert_eq!(p.status, ProjectStatus::Draft);
    }

    #[test]
    fn invalid_transition_draft_to_complete() {
        let mut p = Project::new();
        let err = p.transition_to(ProjectStatus::Complete).unwrap_err();
        assert_eq!(
            err,
            ProjectError::InvalidStatusTransition {
                from: ProjectStatus::Draft,
                to: ProjectStatus::Complete,
            }
        );
    }

    #[test]
    fn invalid_transition_draft_to_approved() {
        let mut p = Project::new();
        let err = p.transition_to(ProjectStatus::Approved).unwrap_err();
        assert_eq!(
            err,
            ProjectError::InvalidStatusTransition {
                from: ProjectStatus::Draft,
                to: ProjectStatus::Approved,
            }
        );
    }

    #[test]
    fn tier_access() {
        let mut p = Project::new();
        let zone = make_zone(ZoneType::Bed, None);
        let zone_id = zone.id;
        let material_id = MaterialId::new();
        p.add_zone(zone).unwrap();

        let good = p.tier_mut(TierLevel::Good);
        good.assignments.push(crate::types::MaterialAssignment {
            zone_id,
            material_id,
            overrides: None,
        });

        assert_eq!(p.tier(TierLevel::Good).assignments.len(), 1);
        assert_eq!(p.tier(TierLevel::Good).assignments[0].zone_id, zone_id);
    }

    #[test]
    fn project_json_serde_round_trip() {
        let mut p = Project::new();
        let zone = make_zone(ZoneType::Patio, Some("Test patio"));
        p.add_zone(zone).unwrap();

        let json = serde_json::to_string_pretty(&p).unwrap();
        let back: Project = serde_json::from_str(&json).unwrap();

        assert_eq!(p.id, back.id);
        assert_eq!(p.status, back.status);
        assert_eq!(p.zones.len(), back.zones.len());
        assert_eq!(p.zones[0].zone_type, back.zones[0].zone_type);
        assert_eq!(p.zones[0].label, back.zones[0].label);
        assert_eq!(p.tiers.len(), back.tiers.len());
    }
}
