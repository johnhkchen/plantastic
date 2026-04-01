//! Mesh generation: Delaunay triangulation and decimation.

use crate::error::ScanError;
use crate::types::Point;
use serde::{Deserialize, Serialize};

/// Triangle mesh with per-vertex positions, normals, and colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainMesh {
    /// Vertex positions in meters: one [x, y, z] per vertex.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex unit normals (smooth-shaded).
    pub normals: Vec<[f32; 3]>,
    /// Per-vertex RGB colors.
    pub colors: Vec<[u8; 3]>,
    /// Triangle indices (length is always a multiple of 3).
    pub indices: Vec<u32>,
}

impl TerrainMesh {
    /// Number of triangles in the mesh.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Configuration for mesh generation.
#[derive(Debug, Clone)]
pub struct MeshConfig {
    /// Target number of triangles after decimation (default: 50,000).
    pub target_triangles: usize,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            target_triangles: 50_000,
        }
    }
}

/// Build a triangle mesh from ground points using 2D Delaunay triangulation.
///
/// Projects points onto the XY plane for triangulation, then uses original
/// 3D positions for vertex data. Colors default to green `[0, 128, 0]`
/// when a point has no color.
///
/// # Errors
///
/// Returns `ScanError::MeshGeneration` if there are fewer than 3 points
/// or if triangulation produces no triangles.
pub fn triangulate(points: &[Point]) -> Result<TerrainMesh, ScanError> {
    if points.len() < 3 {
        return Err(ScanError::MeshGeneration(format!(
            "need at least 3 points for triangulation, got {}",
            points.len()
        )));
    }

    // Project onto XY for 2D Delaunay
    let coords: Vec<delaunator::Point> = points
        .iter()
        .map(|p| delaunator::Point {
            x: f64::from(p.position[0]),
            y: f64::from(p.position[1]),
        })
        .collect();

    let result = delaunator::triangulate(&coords);

    if result.triangles.is_empty() {
        return Err(ScanError::MeshGeneration(
            "triangulation produced no triangles (points may be collinear)".to_string(),
        ));
    }

    let positions: Vec<[f32; 3]> = points.iter().map(|p| p.position).collect();
    let colors: Vec<[u8; 3]> = points
        .iter()
        .map(|p| p.color.unwrap_or([0, 128, 0]))
        .collect();

    #[allow(clippy::cast_possible_truncation)]
    let indices: Vec<u32> = result.triangles.iter().map(|&i| i as u32).collect();

    let normals = compute_normals(&positions, &indices);

    Ok(TerrainMesh {
        positions,
        normals,
        colors,
        indices,
    })
}

/// Decimate a mesh to approximately `target_triangles` using meshopt.
///
/// If the mesh already has fewer triangles than `target_triangles`, returns
/// it unchanged. Vertex colors are preserved by mapping each surviving
/// vertex to the nearest original vertex.
pub fn decimate(mesh: &TerrainMesh, target_triangles: usize) -> TerrainMesh {
    if mesh.triangle_count() <= target_triangles {
        return mesh.clone();
    }

    let vertex_count = mesh.positions.len();

    let target_index_count = target_triangles * 3;
    let target_error = 0.01; // 1cm error threshold

    let simplified_indices = meshopt::simplify_decoder(
        &mesh.indices,
        &mesh.positions,
        target_index_count,
        target_error,
        meshopt::SimplifyOptions::empty(),
        None,
    );

    if simplified_indices.is_empty() {
        return mesh.clone();
    }

    // Remap: collect only vertices referenced by simplified indices
    let mut vertex_remap = vec![u32::MAX; vertex_count];
    let mut new_positions = Vec::new();
    let mut new_colors = Vec::new();
    let mut new_count: u32 = 0;

    for &idx in &simplified_indices {
        let idx = idx as usize;
        if vertex_remap[idx] == u32::MAX {
            vertex_remap[idx] = new_count;
            new_positions.push(mesh.positions[idx]);
            new_colors.push(mesh.colors[idx]);
            new_count += 1;
        }
    }

    let new_indices: Vec<u32> = simplified_indices
        .iter()
        .map(|&idx| vertex_remap[idx as usize])
        .collect();

    let new_normals = compute_normals(&new_positions, &new_indices);

    TerrainMesh {
        positions: new_positions,
        normals: new_normals,
        colors: new_colors,
        indices: new_indices,
    }
}

/// Compute smooth per-vertex normals by averaging incident face normals.
fn compute_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0_f32; 3]; positions.len()];

    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];

        // Edge vectors
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        // Cross product (face normal, not normalized — area-weighted)
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

    // Normalize
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > f32::EPSILON {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        } else {
            // Default to +Z for degenerate cases
            *n = [0.0, 0.0, 1.0];
        }
    }

    normals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangulate_square() {
        pt_test_utils::timed(|| {
            // Four corners of a 1m square at z=0
            let points = vec![
                Point {
                    position: [0.0, 0.0, 0.0],
                    color: Some([255, 0, 0]),
                },
                Point {
                    position: [1.0, 0.0, 0.0],
                    color: Some([0, 255, 0]),
                },
                Point {
                    position: [1.0, 1.0, 0.0],
                    color: Some([0, 0, 255]),
                },
                Point {
                    position: [0.0, 1.0, 0.0],
                    color: Some([255, 255, 0]),
                },
            ];

            let mesh = triangulate(&points).unwrap();

            // 4 vertices, 2 triangles = 6 indices
            assert_eq!(mesh.positions.len(), 4);
            assert_eq!(mesh.indices.len(), 6);
            assert_eq!(mesh.triangle_count(), 2);
            assert_eq!(mesh.colors.len(), 4);
            assert_eq!(mesh.normals.len(), 4);

            // All normals should point roughly in +Z direction (flat surface)
            for n in &mesh.normals {
                assert!(n[2].abs() > 0.9, "normal z={} should be near ±1", n[2]);
            }

            // Colors preserved
            assert_eq!(mesh.colors[0], [255, 0, 0]);
            assert_eq!(mesh.colors[1], [0, 255, 0]);
        });
    }

    #[test]
    fn test_triangulate_insufficient_points() {
        pt_test_utils::timed(|| {
            let points = vec![
                Point {
                    position: [0.0, 0.0, 0.0],
                    color: None,
                },
                Point {
                    position: [1.0, 0.0, 0.0],
                    color: None,
                },
            ];
            let result = triangulate(&points);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_triangulate_default_color() {
        pt_test_utils::timed(|| {
            let points = vec![
                Point {
                    position: [0.0, 0.0, 0.0],
                    color: None,
                },
                Point {
                    position: [1.0, 0.0, 0.0],
                    color: None,
                },
                Point {
                    position: [0.5, 1.0, 0.0],
                    color: None,
                },
            ];
            let mesh = triangulate(&points).unwrap();
            // Default color is green [0, 128, 0]
            for c in &mesh.colors {
                assert_eq!(*c, [0, 128, 0]);
            }
        });
    }

    #[test]
    fn test_decimate_passthrough_when_below_target() {
        pt_test_utils::timed(|| {
            let points = vec![
                Point {
                    position: [0.0, 0.0, 0.0],
                    color: Some([255, 0, 0]),
                },
                Point {
                    position: [1.0, 0.0, 0.0],
                    color: Some([0, 255, 0]),
                },
                Point {
                    position: [0.5, 1.0, 0.0],
                    color: Some([0, 0, 255]),
                },
            ];
            let mesh = triangulate(&points).unwrap();
            assert_eq!(mesh.triangle_count(), 1);

            // Target 100 triangles, mesh has 1 — should return unchanged
            let decimated = decimate(&mesh, 100);
            assert_eq!(decimated.triangle_count(), 1);
            assert_eq!(decimated.positions.len(), 3);
        });
    }

    #[test]
    fn test_decimate_reduces_triangle_count() {
        pt_test_utils::timed(|| {
            // Generate a grid of points to get many triangles
            let mut points = Vec::new();
            let grid_size = 30; // 30×30 = 900 points → ~1700 triangles
            for i in 0..grid_size {
                for j in 0..grid_size {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    points.push(Point {
                        position: [i as f32 * 0.1, j as f32 * 0.1, 0.0],
                        color: Some([
                            ((i * 255) / grid_size) as u8,
                            ((j * 255) / grid_size) as u8,
                            128,
                        ]),
                    });
                }
            }

            let mesh = triangulate(&points).unwrap();
            assert!(
                mesh.triangle_count() > 100,
                "need enough triangles to test decimation"
            );

            let target = 100;
            let decimated = decimate(&mesh, target);

            // meshopt may not hit exact target but should reduce significantly
            assert!(
                decimated.triangle_count() <= target * 2,
                "decimated to {} triangles, target was {}",
                decimated.triangle_count(),
                target
            );
            assert!(
                decimated.triangle_count() < mesh.triangle_count(),
                "should have fewer triangles after decimation"
            );

            // All indices should be valid
            let vert_count = decimated.positions.len();
            for &idx in &decimated.indices {
                assert!(
                    (idx as usize) < vert_count,
                    "invalid index {idx} with {vert_count} vertices"
                );
            }

            // Colors and normals should have matching length
            assert_eq!(decimated.colors.len(), vert_count);
            assert_eq!(decimated.normals.len(), vert_count);
        });
    }
}
