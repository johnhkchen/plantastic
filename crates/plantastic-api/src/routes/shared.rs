//! Shared type conversion helpers used by multiple route modules.

use pt_materials::{Material, MaterialId};
use pt_project::{AssignmentOverrides, MaterialAssignment, Tier, TierLevel, Zone, ZoneId};

use crate::error::AppError;

pub(crate) fn zone_rows_to_zones(rows: Vec<pt_repo::zone::ZoneRow>) -> Vec<Zone> {
    rows.into_iter()
        .map(|r| Zone {
            id: ZoneId(r.id),
            geometry: r.geometry,
            zone_type: r.zone_type,
            label: r.label,
        })
        .collect()
}

pub(crate) fn material_rows_to_materials(
    rows: Vec<pt_repo::material::MaterialRow>,
) -> Vec<Material> {
    rows.into_iter()
        .map(|r| Material {
            id: MaterialId(r.id),
            name: r.name,
            category: r.category,
            unit: r.unit,
            price_per_unit: r.price_per_unit,
            depth_inches: r.depth_inches,
            texture_ref: r.texture_key,
            photo_ref: r.photo_key,
            supplier_sku: r.supplier_sku,
            extrusion: r.extrusion,
        })
        .collect()
}

pub(crate) fn build_tier(
    level: TierLevel,
    rows: Vec<pt_repo::tier_assignment::TierAssignmentRow>,
) -> Result<Tier, AppError> {
    let mut assignments = Vec::with_capacity(rows.len());
    for r in rows {
        let overrides = match r.overrides {
            Some(v) => {
                let parsed: AssignmentOverrides = serde_json::from_value(v)
                    .map_err(|e| AppError::Internal(format!("invalid overrides JSONB: {e}")))?;
                Some(parsed)
            }
            None => None,
        };
        assignments.push(MaterialAssignment {
            zone_id: ZoneId(r.zone_id),
            material_id: MaterialId(r.material_id),
            overrides,
        });
    }
    Ok(Tier { level, assignments })
}
