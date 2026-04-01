//! GLB (binary glTF 2.0) assembly from zone meshes.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use crate::error::SceneError;
use crate::mesh::ZoneMesh;
use pt_materials::MaterialCategory;

/// GLB magic number: "glTF" in little-endian.
const GLB_MAGIC: u32 = 0x4654_6C67;
/// GLB format version 2.
const GLB_VERSION: u32 = 2;
/// JSON chunk type.
const CHUNK_JSON: u32 = 0x4E4F_534A;
/// Binary chunk type.
const CHUNK_BIN: u32 = 0x004E_4942;

/// A named mesh with material color, ready for GLB assembly.
#[derive(Debug)]
pub(crate) struct GlbMesh {
    pub name: String,
    pub mesh: ZoneMesh,
    pub base_color: [f32; 4],
}

/// Map a material category to an sRGB base color (RGBA, 0.0–1.0).
pub(crate) fn category_base_color(category: MaterialCategory) -> [f32; 4] {
    match category {
        MaterialCategory::Hardscape => [0.706, 0.706, 0.706, 1.0], // gray
        MaterialCategory::Softscape => [0.545, 0.353, 0.169, 1.0], // brown
        MaterialCategory::Edging => [0.392, 0.392, 0.392, 1.0],    // dark gray
        MaterialCategory::Fill => [0.824, 0.706, 0.549, 1.0],      // tan
    }
}

/// Assemble a binary glTF 2.0 (.glb) from zone meshes.
///
/// Each mesh becomes a named node in the scene. Materials are deduplicated by
/// base color.
pub(crate) fn to_glb(meshes: &[GlbMesh]) -> Result<Vec<u8>, SceneError> {
    if meshes.is_empty() {
        return build_empty_glb();
    }

    // Collect unique materials and map each mesh to its material index.
    let mut unique_colors: Vec<[f32; 4]> = Vec::new();
    let mut mesh_material_indices: Vec<usize> = Vec::new();

    for m in meshes {
        let idx = unique_colors
            .iter()
            .position(|c| colors_equal(c, &m.base_color))
            .unwrap_or_else(|| {
                unique_colors.push(m.base_color);
                unique_colors.len() - 1
            });
        mesh_material_indices.push(idx);
    }

    // Build binary buffer: for each mesh, positions then normals then indices.
    let mut bin = Vec::new();
    let mut mesh_layouts: Vec<MeshLayout> = Vec::new();

    for m in meshes {
        let pos_offset = bin.len();
        let vertex_count = m.mesh.positions.len();
        let index_count = m.mesh.indices.len();

        // Positions
        let (mut pos_min, mut pos_max) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for p in &m.mesh.positions {
            for i in 0..3 {
                pos_min[i] = pos_min[i].min(p[i]);
                pos_max[i] = pos_max[i].max(p[i]);
            }
            for &v in p {
                bin.extend_from_slice(&v.to_le_bytes());
            }
        }

        let norm_offset = bin.len();
        for n in &m.mesh.normals {
            for &v in n {
                bin.extend_from_slice(&v.to_le_bytes());
            }
        }

        let idx_offset = bin.len();
        let max_index = m.mesh.indices.iter().copied().max().unwrap_or(0);
        for &idx in &m.mesh.indices {
            bin.extend_from_slice(&idx.to_le_bytes());
        }

        mesh_layouts.push(MeshLayout {
            pos_offset,
            pos_size: vertex_count * 12,
            norm_offset,
            norm_size: vertex_count * 12,
            idx_offset,
            idx_size: index_count * 4,
            vertex_count,
            index_count,
            pos_min,
            pos_max,
            max_index,
        });
    }

    // Pad binary to 4-byte alignment
    while bin.len() % 4 != 0 {
        bin.push(0);
    }

    // Build glTF JSON
    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut gltf_meshes = Vec::new();
    let mut nodes = Vec::new();
    let mut accessor_idx = 0_usize;

    for (i, (m, layout)) in meshes.iter().zip(&mesh_layouts).enumerate() {
        // bufferView for positions
        let bv_pos = buffer_views.len();
        buffer_views.push(serde_json::json!({
            "buffer": 0,
            "byteOffset": layout.pos_offset,
            "byteLength": layout.pos_size,
            "target": 34962 // ARRAY_BUFFER
        }));

        // bufferView for normals
        let bv_norm = buffer_views.len();
        buffer_views.push(serde_json::json!({
            "buffer": 0,
            "byteOffset": layout.norm_offset,
            "byteLength": layout.norm_size,
            "target": 34962
        }));

        // bufferView for indices
        let bv_idx = buffer_views.len();
        buffer_views.push(serde_json::json!({
            "buffer": 0,
            "byteOffset": layout.idx_offset,
            "byteLength": layout.idx_size,
            "target": 34963 // ELEMENT_ARRAY_BUFFER
        }));

        // Accessors: position, normal, indices
        let acc_pos = accessor_idx;
        accessors.push(serde_json::json!({
            "bufferView": bv_pos,
            "componentType": 5126, // FLOAT
            "count": layout.vertex_count,
            "type": "VEC3",
            "min": layout.pos_min,
            "max": layout.pos_max
        }));
        accessor_idx += 1;

        let acc_norm = accessor_idx;
        accessors.push(serde_json::json!({
            "bufferView": bv_norm,
            "componentType": 5126,
            "count": layout.vertex_count,
            "type": "VEC3"
        }));
        accessor_idx += 1;

        let acc_idx = accessor_idx;
        accessors.push(serde_json::json!({
            "bufferView": bv_idx,
            "componentType": 5125, // UNSIGNED_INT
            "count": layout.index_count,
            "type": "SCALAR",
            "min": [0],
            "max": [layout.max_index]
        }));
        accessor_idx += 1;

        // Mesh
        gltf_meshes.push(serde_json::json!({
            "primitives": [{
                "attributes": {
                    "POSITION": acc_pos,
                    "NORMAL": acc_norm
                },
                "indices": acc_idx,
                "material": mesh_material_indices[i],
                "mode": 4 // TRIANGLES
            }]
        }));

        // Node
        nodes.push(serde_json::json!({
            "mesh": i,
            "name": m.name
        }));
    }

    // Materials
    let gltf_materials: Vec<serde_json::Value> = unique_colors
        .iter()
        .map(|color| {
            serde_json::json!({
                "pbrMetallicRoughness": {
                    "baseColorFactor": color,
                    "metallicFactor": 0.0,
                    "roughnessFactor": 0.8
                }
            })
        })
        .collect();

    let node_indices: Vec<usize> = (0..meshes.len()).collect();

    let json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "plantastic/pt-scene" },
        "scene": 0,
        "scenes": [{ "nodes": node_indices }],
        "nodes": nodes,
        "meshes": gltf_meshes,
        "materials": gltf_materials,
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{ "byteLength": bin.len() }]
    });

    assemble_glb(&json, &bin)
}

fn build_empty_glb() -> Result<Vec<u8>, SceneError> {
    let json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "plantastic/pt-scene" },
        "scene": 0,
        "scenes": [{ "nodes": [] }]
    });
    assemble_glb(&json, &[])
}

fn assemble_glb(json: &serde_json::Value, bin: &[u8]) -> Result<Vec<u8>, SceneError> {
    let json_str = serde_json::to_string(json)
        .map_err(|e| SceneError::Export(format!("JSON serialization failed: {e}")))?;

    let mut json_bytes = json_str.into_bytes();
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }

    let has_bin = !bin.is_empty();
    let bin_chunk_size = if has_bin { 8 + bin.len() } else { 0 };
    let total_size = 12 + 8 + json_bytes.len() + bin_chunk_size;
    let mut glb = Vec::with_capacity(total_size);

    // Header
    glb.extend_from_slice(&GLB_MAGIC.to_le_bytes());
    glb.extend_from_slice(&GLB_VERSION.to_le_bytes());
    glb.extend_from_slice(&(total_size as u32).to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(&CHUNK_JSON.to_le_bytes());
    glb.extend_from_slice(&json_bytes);

    // Binary chunk (only if non-empty)
    if has_bin {
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&CHUNK_BIN.to_le_bytes());
        glb.extend_from_slice(bin);
    }

    Ok(glb)
}

fn colors_equal(a: &[f32; 4], b: &[f32; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6)
}

struct MeshLayout {
    pos_offset: usize,
    pos_size: usize,
    norm_offset: usize,
    norm_size: usize,
    idx_offset: usize,
    idx_size: usize,
    vertex_count: usize,
    index_count: usize,
    pos_min: [f32; 3],
    pos_max: [f32; 3],
    max_index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::ZoneMesh;

    fn make_triangle_mesh() -> ZoneMesh {
        ZoneMesh {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            indices: vec![0, 1, 2],
        }
    }

    #[test]
    fn glb_magic_and_version() {
        pt_test_utils::timed(|| {
            let meshes = vec![GlbMesh {
                name: "test".to_string(),
                mesh: make_triangle_mesh(),
                base_color: [0.5, 0.5, 0.5, 1.0],
            }];
            let glb = to_glb(&meshes).unwrap();

            assert!(glb.len() >= 12);
            let magic = u32::from_le_bytes([glb[0], glb[1], glb[2], glb[3]]);
            assert_eq!(magic, GLB_MAGIC);
            let version = u32::from_le_bytes([glb[4], glb[5], glb[6], glb[7]]);
            assert_eq!(version, 2);
            let total = u32::from_le_bytes([glb[8], glb[9], glb[10], glb[11]]);
            assert_eq!(total as usize, glb.len());
        });
    }

    #[test]
    fn glb_json_chunk_parseable() {
        pt_test_utils::timed(|| {
            let meshes = vec![GlbMesh {
                name: "zone_a".to_string(),
                mesh: make_triangle_mesh(),
                base_color: [0.5, 0.5, 0.5, 1.0],
            }];
            let glb = to_glb(&meshes).unwrap();

            let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
            let chunk_type = u32::from_le_bytes([glb[16], glb[17], glb[18], glb[19]]);
            assert_eq!(chunk_type, CHUNK_JSON);

            let json_bytes = &glb[20..20 + json_len];
            let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();

            assert_eq!(parsed["asset"]["version"], "2.0");
            assert_eq!(parsed["asset"]["generator"], "plantastic/pt-scene");
            assert!(parsed["meshes"].is_array());
            assert!(parsed["nodes"].is_array());
            assert!(parsed["materials"].is_array());

            // Node name matches input
            assert_eq!(parsed["nodes"][0]["name"], "zone_a");
        });
    }

    #[test]
    fn multiple_meshes_distinct_names() {
        pt_test_utils::timed(|| {
            let meshes = vec![
                GlbMesh {
                    name: "patio".to_string(),
                    mesh: make_triangle_mesh(),
                    base_color: category_base_color(MaterialCategory::Hardscape),
                },
                GlbMesh {
                    name: "flower_bed".to_string(),
                    mesh: make_triangle_mesh(),
                    base_color: category_base_color(MaterialCategory::Softscape),
                },
            ];
            let glb = to_glb(&meshes).unwrap();

            let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
            let json_bytes = &glb[20..20 + json_len];
            let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();

            let nodes = parsed["nodes"].as_array().unwrap();
            assert_eq!(nodes.len(), 2);
            assert_eq!(nodes[0]["name"], "patio");
            assert_eq!(nodes[1]["name"], "flower_bed");

            // Two distinct materials (different colors)
            let materials = parsed["materials"].as_array().unwrap();
            assert_eq!(materials.len(), 2);
        });
    }

    #[test]
    fn empty_meshes_valid_glb() {
        pt_test_utils::timed(|| {
            let glb = to_glb(&[]).unwrap();

            assert!(glb.len() >= 12);
            let magic = u32::from_le_bytes([glb[0], glb[1], glb[2], glb[3]]);
            assert_eq!(magic, GLB_MAGIC);

            let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
            let json_bytes = &glb[20..20 + json_len];
            let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();
            assert_eq!(parsed["asset"]["version"], "2.0");
            assert_eq!(parsed["scenes"][0]["nodes"].as_array().unwrap().len(), 0);
        });
    }

    #[test]
    fn same_color_deduplicates_materials() {
        pt_test_utils::timed(|| {
            let color = category_base_color(MaterialCategory::Hardscape);
            let meshes = vec![
                GlbMesh {
                    name: "a".to_string(),
                    mesh: make_triangle_mesh(),
                    base_color: color,
                },
                GlbMesh {
                    name: "b".to_string(),
                    mesh: make_triangle_mesh(),
                    base_color: color,
                },
            ];
            let glb = to_glb(&meshes).unwrap();

            let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
            let json_bytes = &glb[20..20 + json_len];
            let parsed: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();

            // Same color → one material
            let materials = parsed["materials"].as_array().unwrap();
            assert_eq!(materials.len(), 1);
        });
    }
}
