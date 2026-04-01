//! Zone polygon → extruded 3D triangle mesh.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use crate::error::SceneError;
use pt_materials::ExtrusionBehavior;

/// Default fill depth in inches when Fills extrusion is used.
const DEFAULT_FILL_DEPTH_INCHES: f64 = 4.0;

/// Extruded triangle mesh for a single zone.
#[derive(Debug, Clone)]
pub(crate) struct ZoneMesh {
    /// Vertex positions: X = polygon X, Y = height (up), Z = polygon Y.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex unit normals.
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

impl ZoneMesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Generate an extruded 3D mesh from a zone polygon and its material extrusion.
///
/// Coordinate mapping: polygon X → scene X, polygon Y → scene Z, height → scene Y.
/// Units: feet (1 scene unit = 1 foot).
pub(crate) fn extrude_zone(
    polygon: &pt_geo::Polygon<f64>,
    extrusion: &ExtrusionBehavior,
) -> Result<ZoneMesh, SceneError> {
    let (bottom_y, top_y) = extrusion_heights(extrusion);

    let exterior = polygon.exterior();
    let coords: Vec<_> = exterior.0.iter().collect();

    // Need at least 3 unique vertices (polygon rings are closed: first == last)
    if coords.len() < 4 {
        return Ok(ZoneMesh {
            positions: vec![],
            normals: vec![],
            indices: vec![],
        });
    }

    // Triangulate the polygon face using earcutr.
    // earcutr expects a flat array of [x, y, x, y, ...] coordinates.
    let ring_len = coords.len() - 1; // exclude closing vertex
    let mut flat_coords: Vec<f64> = Vec::with_capacity(ring_len * 2);
    for c in &coords[..ring_len] {
        flat_coords.push(c.x);
        flat_coords.push(c.y);
    }

    let face_indices = earcutr::earcut(&flat_coords, &[], 2)
        .map_err(|e| SceneError::Triangulation(format!("{e:?}")))?;

    if face_indices.is_empty() {
        return Ok(ZoneMesh {
            positions: vec![],
            normals: vec![],
            indices: vec![],
        });
    }

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Top face: vertices at top_y
    let top_base = positions.len() as u32;
    for c in &coords[..ring_len] {
        positions.push([c.x as f32, top_y as f32, c.y as f32]);
    }
    for &idx in &face_indices {
        indices.push(top_base + idx as u32);
    }

    // Bottom face: vertices at bottom_y, reversed winding
    let bot_base = positions.len() as u32;
    for c in &coords[..ring_len] {
        positions.push([c.x as f32, bottom_y as f32, c.y as f32]);
    }
    for tri in face_indices.chunks_exact(3) {
        indices.push(bot_base + tri[2] as u32);
        indices.push(bot_base + tri[1] as u32);
        indices.push(bot_base + tri[0] as u32);
    }

    // Side walls: one quad (2 triangles) per edge
    for i in 0..ring_len {
        let j = (i + 1) % ring_len;
        let c0 = &coords[i];
        let c1 = &coords[j];

        let base = positions.len() as u32;
        // Quad: top-left, top-right, bottom-right, bottom-left
        positions.push([c0.x as f32, top_y as f32, c0.y as f32]);
        positions.push([c1.x as f32, top_y as f32, c1.y as f32]);
        positions.push([c1.x as f32, bottom_y as f32, c1.y as f32]);
        positions.push([c0.x as f32, bottom_y as f32, c0.y as f32]);

        // Two triangles (CCW when viewed from outside)
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 1);

        indices.push(base);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    let normals = compute_normals(&positions, &indices);

    Ok(ZoneMesh {
        positions,
        normals,
        indices,
    })
}

/// Compute extrusion bottom and top Y values in feet.
fn extrusion_heights(extrusion: &ExtrusionBehavior) -> (f64, f64) {
    match extrusion {
        ExtrusionBehavior::SitsOnTop { height_inches }
        | ExtrusionBehavior::BuildsUp { height_inches } => (0.0, height_inches / 12.0),
        ExtrusionBehavior::Fills { .. } => (-DEFAULT_FILL_DEPTH_INCHES / 12.0, 0.0),
    }
}

/// Compute smooth per-vertex normals by accumulating face normals.
fn compute_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0_f32; 3]; positions.len()];

    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];

        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        for &vi in &[i0, i1, i2] {
            normals[vi][0] += n[0];
            normals[vi][1] += n[1];
            normals[vi][2] += n[2];
        }
    }

    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > f32::EPSILON {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        } else {
            *n = [0.0, 1.0, 0.0]; // default to +Y (up)
        }
    }

    normals
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_geo::polygon;

    #[test]
    fn square_sits_on_top_vertex_count() {
        pt_test_utils::timed(|| {
            let poly = polygon![
                (x: 0.0, y: 0.0),
                (x: 10.0, y: 0.0),
                (x: 10.0, y: 10.0),
                (x: 0.0, y: 10.0),
            ];
            let extrusion = ExtrusionBehavior::SitsOnTop {
                height_inches: 12.0,
            };
            let mesh = extrude_zone(&poly, &extrusion).unwrap();

            // 4 vertices: top face (4) + bottom face (4) + walls (4 edges × 4 verts) = 24
            assert_eq!(mesh.positions.len(), 24);
            assert_eq!(mesh.normals.len(), 24);
            // Triangles: top (2) + bottom (2) + walls (4 edges × 2 tris) = 12
            assert_eq!(mesh.triangle_count(), 12);
        });
    }

    #[test]
    fn triangle_extrusion_index_count() {
        pt_test_utils::timed(|| {
            let poly = polygon![
                (x: 0.0, y: 0.0),
                (x: 6.0, y: 0.0),
                (x: 3.0, y: 4.0),
            ];
            let extrusion = ExtrusionBehavior::BuildsUp {
                height_inches: 24.0,
            };
            let mesh = extrude_zone(&poly, &extrusion).unwrap();

            // 3 verts: top (3) + bottom (3) + walls (3 edges × 4) = 18
            assert_eq!(mesh.positions.len(), 18);
            // Triangles: top (1) + bottom (1) + walls (3 edges × 2) = 8
            assert_eq!(mesh.triangle_count(), 8);
            assert_eq!(mesh.indices.len(), 24); // 8 × 3
        });
    }

    #[test]
    fn sits_on_top_height_matches_spec() {
        pt_test_utils::timed(|| {
            let poly = polygon![
                (x: 0.0, y: 0.0),
                (x: 5.0, y: 0.0),
                (x: 5.0, y: 5.0),
                (x: 0.0, y: 5.0),
            ];
            let height_inches = 6.0;
            let extrusion = ExtrusionBehavior::SitsOnTop { height_inches };
            let mesh = extrude_zone(&poly, &extrusion).unwrap();

            // Independent calculation: 6 inches = 0.5 feet
            let expected_top_y: f32 = 0.5;
            let expected_bottom_y: f32 = 0.0;

            // Top face vertices (first 4) should be at expected_top_y
            for pos in &mesh.positions[..4] {
                approx::assert_relative_eq!(pos[1], expected_top_y, epsilon = 1e-6);
            }
            // Bottom face vertices (next 4) should be at expected_bottom_y
            for pos in &mesh.positions[4..8] {
                approx::assert_relative_eq!(pos[1], expected_bottom_y, epsilon = 1e-6);
            }
        });
    }

    #[test]
    fn fills_extrusion_below_grade() {
        pt_test_utils::timed(|| {
            let poly = polygon![
                (x: 0.0, y: 0.0),
                (x: 8.0, y: 0.0),
                (x: 8.0, y: 8.0),
                (x: 0.0, y: 8.0),
            ];
            let extrusion = ExtrusionBehavior::Fills { flush: true };
            let mesh = extrude_zone(&poly, &extrusion).unwrap();

            // Independent: fill depth = 4 inches = 4/12 = 0.3333... feet
            let expected_bottom: f32 = -(4.0_f32 / 12.0);
            let expected_top: f32 = 0.0;

            // Top face at grade (0.0)
            for pos in &mesh.positions[..4] {
                approx::assert_relative_eq!(pos[1], expected_top, epsilon = 1e-6);
            }
            // Bottom face below grade
            for pos in &mesh.positions[4..8] {
                approx::assert_relative_eq!(pos[1], expected_bottom, epsilon = 1e-4);
            }
        });
    }

    #[test]
    fn empty_polygon_returns_empty_mesh() {
        pt_test_utils::timed(|| {
            let poly = pt_geo::Polygon::new(pt_geo::LineString::new(vec![]), vec![]);
            let extrusion = ExtrusionBehavior::SitsOnTop { height_inches: 1.0 };
            let mesh = extrude_zone(&poly, &extrusion).unwrap();
            assert!(mesh.positions.is_empty());
            assert!(mesh.indices.is_empty());
        });
    }

    #[test]
    fn builds_up_same_as_sits_on_top() {
        pt_test_utils::timed(|| {
            let poly = polygon![
                (x: 0.0, y: 0.0),
                (x: 4.0, y: 0.0),
                (x: 4.0, y: 4.0),
                (x: 0.0, y: 4.0),
            ];
            let h = 18.0; // inches
            let mesh_sot =
                extrude_zone(&poly, &ExtrusionBehavior::SitsOnTop { height_inches: h }).unwrap();
            let mesh_bu =
                extrude_zone(&poly, &ExtrusionBehavior::BuildsUp { height_inches: h }).unwrap();

            assert_eq!(mesh_sot.positions.len(), mesh_bu.positions.len());
            assert_eq!(mesh_sot.indices.len(), mesh_bu.indices.len());
            for (a, b) in mesh_sot.positions.iter().zip(&mesh_bu.positions) {
                approx::assert_relative_eq!(a[1], b[1], epsilon = 1e-6);
            }
        });
    }
}
