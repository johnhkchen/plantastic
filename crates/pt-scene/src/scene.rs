//! Scene generation orchestrator.

use crate::error::SceneError;
use crate::glb::{self, GlbMesh};
use crate::mesh;
use pt_materials::Material;
use pt_project::{MaterialAssignment, TierLevel, Zone};
use serde::{Deserialize, Serialize};

/// Output of scene generation.
#[derive(Debug, Clone)]
pub struct SceneOutput {
    /// Binary glTF 2.0 (.glb) bytes.
    pub glb_bytes: Vec<u8>,
    /// Scene metadata.
    pub metadata: SceneMetadata,
}

/// Metadata about a generated scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMetadata {
    /// Number of zones with geometry in the scene.
    pub zone_count: usize,
    /// Total triangle count across all zone meshes.
    pub triangle_count: usize,
    /// Tier level this scene was generated for.
    pub tier: TierLevel,
}

/// Generate a 3D scene from project zones and material assignments.
///
/// Each assigned zone is extruded into a 3D mesh based on its material's
/// extrusion behavior. The output is a valid glTF 2.0 binary (.glb) where
/// each zone is a named node (matching the viewer's tap-to-inspect protocol).
///
/// # Arguments
///
/// * `zones` — All zones in the project
/// * `assignments` — Material assignments for the desired tier
/// * `materials` — Material catalog (must contain all referenced material IDs)
/// * `tier` — Tier level (used in metadata only)
///
/// # Errors
///
/// Returns `SceneError::MissingMaterial` if an assignment references a material
/// not in the catalog. Returns `SceneError::Triangulation` if polygon
/// triangulation fails.
pub fn generate_scene(
    zones: &[Zone],
    assignments: &[MaterialAssignment],
    materials: &[Material],
    tier: TierLevel,
) -> Result<SceneOutput, SceneError> {
    if zones.is_empty() {
        let glb_bytes = glb::to_glb(&[])?;
        return Ok(SceneOutput {
            glb_bytes,
            metadata: SceneMetadata {
                zone_count: 0,
                triangle_count: 0,
                tier,
            },
        });
    }

    let mut glb_meshes = Vec::new();
    let mut total_triangles = 0_usize;

    for assignment in assignments {
        // Find the zone for this assignment.
        let zone = match zones.iter().find(|z| z.id == assignment.zone_id) {
            Some(z) => z,
            None => continue,
        };

        // Find the material.
        let material = materials
            .iter()
            .find(|m| m.id == assignment.material_id)
            .ok_or(SceneError::MissingMaterial {
                zone_id: assignment.zone_id,
                material_id: assignment.material_id,
            })?;

        // Generate extruded mesh.
        let zone_mesh = mesh::extrude_zone(&zone.geometry, &material.extrusion)?;

        if zone_mesh.positions.is_empty() {
            continue;
        }

        total_triangles += zone_mesh.triangle_count();

        let name = zone.label.clone().unwrap_or_else(|| zone.id.to_string());

        glb_meshes.push(GlbMesh {
            name,
            mesh: zone_mesh,
            base_color: glb::category_base_color(material.category),
        });
    }

    let glb_bytes = glb::to_glb(&glb_meshes)?;

    Ok(SceneOutput {
        glb_bytes,
        metadata: SceneMetadata {
            zone_count: glb_meshes.len(),
            triangle_count: total_triangles,
            tier,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_geo::polygon;
    use pt_materials::{ExtrusionBehavior, Material, MaterialCategory, MaterialId, Unit};
    use pt_project::{Zone, ZoneId, ZoneType};
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use uuid::Uuid;

    fn make_zone(label: &str) -> Zone {
        Zone {
            id: ZoneId::new(),
            geometry: polygon![
                (x: 0.0, y: 0.0),
                (x: 12.0, y: 0.0),
                (x: 12.0, y: 15.0),
                (x: 0.0, y: 15.0),
            ],
            zone_type: ZoneType::Patio,
            label: Some(label.to_string()),
        }
    }

    fn make_material(category: MaterialCategory, height_inches: f64) -> Material {
        Material {
            id: MaterialId::new(),
            name: "Test Material".to_string(),
            category,
            unit: Unit::SqFt,
            price_per_unit: Decimal::from_str("8.50").unwrap(),
            depth_inches: None,
            texture_ref: None,
            photo_ref: None,
            supplier_sku: None,
            extrusion: ExtrusionBehavior::SitsOnTop { height_inches },
        }
    }

    #[test]
    fn single_zone_produces_valid_glb() {
        pt_test_utils::timed(|| {
            let zone = make_zone("Back patio");
            let mat = make_material(MaterialCategory::Hardscape, 1.5);
            let assignment = MaterialAssignment {
                zone_id: zone.id,
                material_id: mat.id,
                overrides: None,
            };

            let output = generate_scene(&[zone], &[assignment], &[mat], TierLevel::Good).unwrap();

            assert!(output.glb_bytes.len() >= 12);
            let magic = u32::from_le_bytes([
                output.glb_bytes[0],
                output.glb_bytes[1],
                output.glb_bytes[2],
                output.glb_bytes[3],
            ]);
            assert_eq!(magic, 0x4654_6C67);

            assert_eq!(output.metadata.zone_count, 1);
            assert!(output.metadata.triangle_count > 0);
            assert_eq!(output.metadata.tier, TierLevel::Good);
        });
    }

    #[test]
    fn multiple_zones_all_present() {
        pt_test_utils::timed(|| {
            let z1 = make_zone("Patio");
            let z2 = Zone {
                id: ZoneId::new(),
                geometry: polygon![
                    (x: 20.0, y: 0.0),
                    (x: 30.0, y: 0.0),
                    (x: 30.0, y: 10.0),
                    (x: 20.0, y: 10.0),
                ],
                zone_type: ZoneType::Bed,
                label: Some("Front bed".to_string()),
            };
            let m1 = make_material(MaterialCategory::Hardscape, 1.0);
            let m2 = make_material(MaterialCategory::Softscape, 3.0);
            let assignments = vec![
                MaterialAssignment {
                    zone_id: z1.id,
                    material_id: m1.id,
                    overrides: None,
                },
                MaterialAssignment {
                    zone_id: z2.id,
                    material_id: m2.id,
                    overrides: None,
                },
            ];

            let output =
                generate_scene(&[z1, z2], &assignments, &[m1, m2], TierLevel::Better).unwrap();

            assert_eq!(output.metadata.zone_count, 2);
            assert!(output.metadata.triangle_count > 0);

            let glb = &output.glb_bytes;
            let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
            let json_bytes = &glb[20..20 + json_len];
            let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();
            let nodes = parsed["nodes"].as_array().unwrap();
            assert_eq!(nodes.len(), 2);

            let names: Vec<&str> = nodes.iter().map(|n| n["name"].as_str().unwrap()).collect();
            assert!(names.contains(&"Patio"));
            assert!(names.contains(&"Front bed"));
        });
    }

    #[test]
    fn empty_zones_returns_empty_scene() {
        pt_test_utils::timed(|| {
            let output = generate_scene(&[], &[], &[], TierLevel::Good).unwrap();

            assert_eq!(output.metadata.zone_count, 0);
            assert_eq!(output.metadata.triangle_count, 0);

            let magic = u32::from_le_bytes([
                output.glb_bytes[0],
                output.glb_bytes[1],
                output.glb_bytes[2],
                output.glb_bytes[3],
            ]);
            assert_eq!(magic, 0x4654_6C67);
        });
    }

    #[test]
    fn missing_material_returns_error() {
        pt_test_utils::timed(|| {
            let zone = make_zone("test");
            let fake_material_id = MaterialId(Uuid::new_v4());
            let assignment = MaterialAssignment {
                zone_id: zone.id,
                material_id: fake_material_id,
                overrides: None,
            };

            let result = generate_scene(&[zone], &[assignment], &[], TierLevel::Good);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                matches!(err, SceneError::MissingMaterial { .. }),
                "expected MissingMaterial, got: {err}"
            );
        });
    }

    #[test]
    fn extrusion_height_in_output_matches_spec() {
        pt_test_utils::timed(|| {
            let zone = make_zone("paver_zone");
            let mat = make_material(MaterialCategory::Hardscape, 1.5);
            let assignment = MaterialAssignment {
                zone_id: zone.id,
                material_id: mat.id,
                overrides: None,
            };

            let output = generate_scene(&[zone], &[assignment], &[mat], TierLevel::Good).unwrap();

            let glb = &output.glb_bytes;
            let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
            let json_bytes = &glb[20..20 + json_len];
            let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();

            let pos_max = &parsed["accessors"][0]["max"];
            let max_y = pos_max[1].as_f64().unwrap();

            // Independent calculation: 1.5 inches / 12 = 0.125 feet
            let expected_height: f64 = 0.125;
            approx::assert_relative_eq!(max_y, expected_height, epsilon = 1e-3);
        });
    }

    #[test]
    fn zone_without_label_uses_id() {
        pt_test_utils::timed(|| {
            let zone = Zone {
                id: ZoneId::new(),
                geometry: polygon![
                    (x: 0.0, y: 0.0),
                    (x: 5.0, y: 0.0),
                    (x: 5.0, y: 5.0),
                    (x: 0.0, y: 5.0),
                ],
                zone_type: ZoneType::Lawn,
                label: None,
            };
            let mat = make_material(MaterialCategory::Softscape, 2.0);
            let assignment = MaterialAssignment {
                zone_id: zone.id,
                material_id: mat.id,
                overrides: None,
            };

            let expected_name = zone.id.to_string();
            let output = generate_scene(&[zone], &[assignment], &[mat], TierLevel::Good).unwrap();

            let glb = &output.glb_bytes;
            let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
            let json_bytes = &glb[20..20 + json_len];
            let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();
            assert_eq!(parsed["nodes"][0]["name"].as_str().unwrap(), expected_name);
        });
    }
}
