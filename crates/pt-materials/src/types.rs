//! Core material domain types.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a material in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialId(pub Uuid);

impl MaterialId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MaterialId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MaterialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Broad category of a landscape material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialCategory {
    Hardscape,
    Softscape,
    Edging,
    Fill,
}

/// Unit of measurement for pricing and quantity computation.
///
/// Determines how pt-quote computes quantity from zone geometry:
/// - `SqFt` → area
/// - `CuYd` → volume (area × depth)
/// - `LinearFt` → perimeter
/// - `Each` → count (1 per assignment)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    SqFt,
    CuYd,
    LinearFt,
    Each,
}

/// How a material is extruded in the 3D scene.
///
/// Controls both visual rendering and quantity computation for volumetric materials.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExtrusionBehavior {
    /// Sits on top of the terrain surface (pavers, stepping stones).
    SitsOnTop { height_inches: f64 },
    /// Fills to be flush with surrounding grade (gravel base).
    Fills { flush: bool },
    /// Builds up from the surface (walls, raised beds).
    BuildsUp { height_inches: f64 },
}

/// A landscape material in the catalog.
///
/// Represents a product a landscaper sells and installs — pavers, mulch, edging,
/// plants, etc. Each material has pricing, physical properties for quantity
/// computation, and references for 3D rendering and quotes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub id: MaterialId,
    pub name: String,
    pub category: MaterialCategory,
    pub unit: Unit,
    pub price_per_unit: Decimal,
    /// Install depth in inches. Used with area to compute volume for cu_yd materials.
    pub depth_inches: Option<f64>,
    /// Reference to PBR texture set for 3D rendering.
    pub texture_ref: Option<String>,
    /// Reference to product photo for quotes/PDFs.
    pub photo_ref: Option<String>,
    /// Supplier SKU for crew ordering.
    pub supplier_sku: Option<String>,
    pub extrusion: ExtrusionBehavior,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn material_id_uniqueness() {
        let a = MaterialId::new();
        let b = MaterialId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn material_id_display() {
        let id = MaterialId(Uuid::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn category_serde_round_trip() {
        for cat in [
            MaterialCategory::Hardscape,
            MaterialCategory::Softscape,
            MaterialCategory::Edging,
            MaterialCategory::Fill,
        ] {
            let json = serde_json::to_string(&cat).unwrap();
            let back: MaterialCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(cat, back);
        }
    }

    #[test]
    fn unit_serde_round_trip() {
        for unit in [Unit::SqFt, Unit::CuYd, Unit::LinearFt, Unit::Each] {
            let json = serde_json::to_string(&unit).unwrap();
            let back: Unit = serde_json::from_str(&json).unwrap();
            assert_eq!(unit, back);
        }
    }

    #[test]
    fn extrusion_serde_sits_on_top() {
        let e = ExtrusionBehavior::SitsOnTop { height_inches: 1.5 };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"type\":\"sits_on_top\""));
        let back: ExtrusionBehavior = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn extrusion_serde_fills() {
        let e = ExtrusionBehavior::Fills { flush: true };
        let json = serde_json::to_string(&e).unwrap();
        let back: ExtrusionBehavior = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn extrusion_serde_builds_up() {
        let e = ExtrusionBehavior::BuildsUp {
            height_inches: 24.0,
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: ExtrusionBehavior = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn material_full_serde_round_trip() {
        let mat = Material {
            id: MaterialId(Uuid::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap()),
            name: "Travertine Pavers".to_string(),
            category: MaterialCategory::Hardscape,
            unit: Unit::SqFt,
            price_per_unit: Decimal::from_str("8.50").unwrap(),
            depth_inches: Some(1.0),
            texture_ref: Some("textures/travertine.pbr".to_string()),
            photo_ref: Some("photos/travertine.jpg".to_string()),
            supplier_sku: Some("TRAV-12x12-NAT".to_string()),
            extrusion: ExtrusionBehavior::SitsOnTop { height_inches: 1.0 },
        };
        let json = serde_json::to_string_pretty(&mat).unwrap();
        let back: Material = serde_json::from_str(&json).unwrap();
        assert_eq!(mat, back);
    }
}
