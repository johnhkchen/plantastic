//! Error types for the project domain.

use crate::types::{ProjectStatus, ZoneId};
use std::fmt;

/// Errors that can occur when operating on a [`Project`](crate::Project).
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectError {
    /// Attempted an invalid status transition.
    InvalidStatusTransition {
        from: ProjectStatus,
        to: ProjectStatus,
    },
    /// Referenced a zone that doesn't exist in the project.
    ZoneNotFound(ZoneId),
    /// Attempted to add a zone with an ID that already exists.
    DuplicateZone(ZoneId),
    /// GeoJSON conversion failed.
    GeoJsonConversion(String),
    /// Project must have exactly 3 tiers.
    InvalidTierCount { expected: usize, got: usize },
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStatusTransition { from, to } => {
                write!(f, "invalid status transition: {from:?} → {to:?}")
            }
            Self::ZoneNotFound(id) => write!(f, "zone not found: {id}"),
            Self::DuplicateZone(id) => write!(f, "duplicate zone ID: {id}"),
            Self::GeoJsonConversion(msg) => write!(f, "GeoJSON conversion error: {msg}"),
            Self::InvalidTierCount { expected, got } => {
                write!(f, "expected {expected} tiers, got {got}")
            }
        }
    }
}

impl std::error::Error for ProjectError {}
