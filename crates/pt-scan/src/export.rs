//! Export: glTF binary (.glb), PNG plan view, and metadata generation.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use crate::error::ScanError;
use crate::mesh::{self, MeshConfig, TerrainMesh};
use crate::types::{BoundingBox, Point, PointCloud};
use image::{ImageBuffer, Rgb};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::time::Instant;

/// All output artifacts from terrain generation.
#[derive(Debug, Clone)]
pub struct TerrainOutput {
    /// Binary glTF 2.0 (.glb) terrain mesh.
    pub mesh_glb: Vec<u8>,
    /// Top-down orthographic PNG image (plan view).
    pub plan_view_png: Vec<u8>,
    /// JSON-serializable metadata.
    pub metadata: TerrainMetadata,
}

/// Metadata about the generated terrain artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainMetadata {
    pub bbox: BoundingBox,
    /// Elevation range: [min_z, max_z] in meters.
    pub elevation_range: [f32; 2],
    pub original_point_count: usize,
    pub decimated_triangle_count: usize,
    pub vertex_count: usize,
    pub processing_time_ms: u64,
}

/// Configuration for terrain export.
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Mesh generation settings.
    pub mesh: MeshConfig,
    /// Plan view resolution in pixels per meter (default: 30.0 ≈ 10 px/ft).
    pub pixels_per_meter: f32,
    /// Whether to render obstacle points as darker overlay on plan view.
    pub canopy_overlay: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            mesh: MeshConfig::default(),
            pixels_per_meter: 30.0,
            canopy_overlay: true,
        }
    }
}

/// Generate terrain artifacts from a processed point cloud.
///
/// Pipeline: triangulate ground points → decimate → export glTF + PNG + metadata.
///
/// # Errors
///
/// Returns `ScanError::MeshGeneration` if triangulation fails, or
/// `ScanError::ExportError` if artifact encoding fails.
pub fn generate_terrain(
    cloud: &PointCloud,
    config: &ExportConfig,
) -> Result<TerrainOutput, ScanError> {
    let start = Instant::now();

    let full_mesh = mesh::triangulate(&cloud.ground)?;
    let decimated = mesh::decimate(&full_mesh, config.mesh.target_triangles);

    let mesh_glb = to_glb(&decimated)?;
    let plan_view_png =
        to_plan_view_png(&decimated, &cloud.obstacles, &cloud.metadata.bbox, config)?;

    let (min_z, max_z) = elevation_range(&decimated.positions);

    let metadata = TerrainMetadata {
        bbox: cloud.metadata.bbox.clone(),
        elevation_range: [min_z, max_z],
        original_point_count: cloud.metadata.original_count,
        decimated_triangle_count: decimated.triangle_count(),
        vertex_count: decimated.positions.len(),
        processing_time_ms: start.elapsed().as_millis() as u64,
    };

    Ok(TerrainOutput {
        mesh_glb,
        plan_view_png,
        metadata,
    })
}

fn elevation_range(positions: &[[f32; 3]]) -> (f32, f32) {
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;
    for p in positions {
        if p[2] < min_z {
            min_z = p[2];
        }
        if p[2] > max_z {
            max_z = p[2];
        }
    }
    (min_z, max_z)
}

// ── glTF Binary Export ─────────────────────────────────────────

/// GLB magic number: "glTF" in little-endian.
const GLB_MAGIC: u32 = 0x4654_6C67;
/// GLB format version 2.
const GLB_VERSION: u32 = 2;
/// JSON chunk type.
const CHUNK_JSON: u32 = 0x4E4F_534A;
/// Binary chunk type.
const CHUNK_BIN: u32 = 0x004E_4942;

/// Serialize a terrain mesh to binary glTF 2.0 (.glb).
fn to_glb(mesh: &TerrainMesh) -> Result<Vec<u8>, ScanError> {
    let vertex_count = mesh.positions.len();
    let index_count = mesh.indices.len();

    // Binary buffer layout:
    //   positions: vertex_count × 12 bytes (3 × f32)
    //   normals:   vertex_count × 12 bytes (3 × f32)
    //   colors:    vertex_count × 4 bytes  (3 × u8 + 1 padding, using RGBA)
    //   indices:   index_count × 4 bytes   (u32)
    let pos_size = vertex_count * 12;
    let norm_size = vertex_count * 12;
    let color_size = vertex_count * 4; // RGBA u8 (glTF requires 4-byte alignment for VEC3 u8)
    let idx_size = index_count * 4;
    let bin_size = pos_size + norm_size + color_size + idx_size;

    // Build binary buffer
    let mut bin = Vec::with_capacity(bin_size);

    // Positions — transform from scan Z-up to glTF Y-up: [x, y, z] → [x, z, -y]
    let (mut pos_min, mut pos_max) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for p in &mesh.positions {
        let transformed = [p[0], p[2], -p[1]];
        for i in 0..3 {
            if transformed[i] < pos_min[i] {
                pos_min[i] = transformed[i];
            }
            if transformed[i] > pos_max[i] {
                pos_max[i] = transformed[i];
            }
        }
        for &v in &transformed {
            bin.extend_from_slice(&v.to_le_bytes());
        }
    }

    // Normals — same Z-up → Y-up transform: [nx, ny, nz] → [nx, nz, -ny]
    for n in &mesh.normals {
        let transformed = [n[0], n[2], -n[1]];
        for &v in &transformed {
            bin.extend_from_slice(&v.to_le_bytes());
        }
    }

    // Colors (RGB → RGBA with alpha=255, glTF VEC4 unsigned byte)
    for c in &mesh.colors {
        bin.push(c[0]);
        bin.push(c[1]);
        bin.push(c[2]);
        bin.push(255); // alpha
    }

    // Indices
    let max_index = mesh.indices.iter().copied().max().unwrap_or(0);
    for &idx in &mesh.indices {
        bin.extend_from_slice(&idx.to_le_bytes());
    }

    // Build JSON
    let json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "plantastic/pt-scan" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0, "name": "terrain" }],
        "meshes": [{
            "primitives": [{
                "attributes": {
                    "POSITION": 0,
                    "NORMAL": 1,
                    "COLOR_0": 2
                },
                "indices": 3,
                "mode": 4  // TRIANGLES
            }]
        }],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126, // FLOAT
                "count": vertex_count,
                "type": "VEC3",
                "min": pos_min,
                "max": pos_max
            },
            {
                "bufferView": 1,
                "componentType": 5126,
                "count": vertex_count,
                "type": "VEC3"
            },
            {
                "bufferView": 2,
                "componentType": 5121, // UNSIGNED_BYTE
                "count": vertex_count,
                "type": "VEC4",
                "normalized": true
            },
            {
                "bufferView": 3,
                "componentType": 5125, // UNSIGNED_INT
                "count": index_count,
                "type": "SCALAR",
                "min": [0],
                "max": [max_index]
            }
        ],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": pos_size, "target": 34962 },
            { "buffer": 0, "byteOffset": pos_size, "byteLength": norm_size, "target": 34962 },
            { "buffer": 0, "byteOffset": pos_size + norm_size, "byteLength": color_size, "target": 34962 },
            { "buffer": 0, "byteOffset": pos_size + norm_size + color_size, "byteLength": idx_size, "target": 34963 }
        ],
        "buffers": [{
            "byteLength": bin_size
        }]
    });

    let json_str = serde_json::to_string(&json)
        .map_err(|e| ScanError::ExportError(format!("JSON serialization failed: {e}")))?;

    // Pad JSON to 4-byte alignment (with spaces)
    let mut json_bytes = json_str.into_bytes();
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }

    // Pad binary to 4-byte alignment (with zeros)
    while bin.len() % 4 != 0 {
        bin.push(0);
    }

    // Assemble GLB
    let total_size = 12 + 8 + json_bytes.len() + 8 + bin.len();
    let mut glb = Vec::with_capacity(total_size);

    // Header: magic + version + total_length
    glb.extend_from_slice(&GLB_MAGIC.to_le_bytes());
    glb.extend_from_slice(&GLB_VERSION.to_le_bytes());
    #[allow(clippy::cast_possible_truncation)]
    glb.extend_from_slice(&(total_size as u32).to_le_bytes());

    // JSON chunk
    #[allow(clippy::cast_possible_truncation)]
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(&CHUNK_JSON.to_le_bytes());
    glb.extend_from_slice(&json_bytes);

    // Binary chunk
    #[allow(clippy::cast_possible_truncation)]
    glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    glb.extend_from_slice(&CHUNK_BIN.to_le_bytes());
    glb.extend_from_slice(&bin);

    Ok(glb)
}

// ── PNG Plan View ──────────────────────────────────────────────

/// Render a top-down orthographic projection of the terrain mesh as PNG.
fn to_plan_view_png(
    mesh: &TerrainMesh,
    obstacles: &[Point],
    bbox: &BoundingBox,
    config: &ExportConfig,
) -> Result<Vec<u8>, ScanError> {
    let width_m = bbox.max[0] - bbox.min[0];
    let height_m = bbox.max[1] - bbox.min[1];

    if width_m <= 0.0 || height_m <= 0.0 {
        return Err(ScanError::ExportError(
            "bounding box has zero or negative extent".to_string(),
        ));
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let img_w = (width_m * config.pixels_per_meter).ceil().max(1.0) as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let img_h = (height_m * config.pixels_per_meter).ceil().max(1.0) as u32;

    // Cap image size to prevent memory explosion
    let img_w = img_w.min(4096);
    let img_h = img_h.min(4096);

    // Background: light gray
    let mut img = ImageBuffer::from_pixel(img_w, img_h, Rgb([220u8, 220, 220]));

    let (min_z, max_z) = elevation_range(&mesh.positions);
    let z_range = (max_z - min_z).max(0.001);

    // Rasterize mesh triangles
    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);

        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];

        let c0 = color_for_vertex(&mesh.colors[i0], p0[2], min_z, z_range);
        let c1 = color_for_vertex(&mesh.colors[i1], p1[2], min_z, z_range);
        let c2 = color_for_vertex(&mesh.colors[i2], p2[2], min_z, z_range);

        // Map world XY to pixel coords (Y is flipped: world Y+ = pixel Y-)
        let px0 = world_to_pixel(p0[0], p0[1], bbox, img_w, img_h);
        let px1 = world_to_pixel(p1[0], p1[1], bbox, img_w, img_h);
        let px2 = world_to_pixel(p2[0], p2[1], bbox, img_w, img_h);

        rasterize_triangle(&mut img, px0, px1, px2, c0, c1, c2);
    }

    // Optional canopy overlay: render obstacle points as dark spots
    if config.canopy_overlay {
        for p in obstacles {
            let (px, py) = world_to_pixel(p.position[0], p.position[1], bbox, img_w, img_h);
            let px_i = px as i32;
            let py_i = py as i32;
            // Draw a small dark dot (3×3 pixels)
            for dy in -1..=1_i32 {
                for dx in -1..=1_i32 {
                    let nx = px_i + dx;
                    let ny = py_i + dy;
                    if nx >= 0 && ny >= 0 && (nx as u32) < img_w && (ny as u32) < img_h {
                        let pixel = img.get_pixel_mut(nx as u32, ny as u32);
                        // Darken existing pixel by 40%
                        pixel[0] = (f32::from(pixel[0]) * 0.6) as u8;
                        pixel[1] = (f32::from(pixel[1]) * 0.6) as u8;
                        pixel[2] = (f32::from(pixel[2]) * 0.6) as u8;
                    }
                }
            }
        }
    }

    // Encode to PNG
    let mut png_buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(Cursor::new(&mut png_buf));
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        img_w,
        img_h,
        image::ExtendedColorType::Rgb8,
    )
    .map_err(|e| ScanError::ExportError(format!("PNG encoding failed: {e}")))?;

    Ok(png_buf)
}

/// Map world coordinates to pixel coordinates.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn world_to_pixel(
    x: f32,
    y: f32,
    bbox: &BoundingBox,
    img_w: u32,
    img_h: u32,
) -> (f32, f32) {
    let nx = (x - bbox.min[0]) / (bbox.max[0] - bbox.min[0]);
    let ny = (y - bbox.min[1]) / (bbox.max[1] - bbox.min[1]);
    let px = nx * (img_w.saturating_sub(1)) as f32;
    // Flip Y: world Y+ is up, image Y+ is down
    let py = (1.0 - ny) * (img_h.saturating_sub(1)) as f32;
    (px, py)
}

/// Blend vertex color with elevation-based shading.
fn color_for_vertex(rgb: &[u8; 3], z: f32, min_z: f32, z_range: f32) -> [u8; 3] {
    // Elevation factor: 0.0 at min_z, 1.0 at max_z
    let t = ((z - min_z) / z_range).clamp(0.0, 1.0);
    // Brighten with elevation (0.7 at lowest, 1.0 at highest)
    let brightness = 0.7 + 0.3 * t;
    [
        (f32::from(rgb[0]) * brightness).min(255.0) as u8,
        (f32::from(rgb[1]) * brightness).min(255.0) as u8,
        (f32::from(rgb[2]) * brightness).min(255.0) as u8,
    ]
}

/// Rasterize a triangle using barycentric coordinates.
fn rasterize_triangle(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    v0: (f32, f32),
    v1: (f32, f32),
    v2: (f32, f32),
    c0: [u8; 3],
    c1: [u8; 3],
    c2: [u8; 3],
) {
    let (w, h) = (img.width(), img.height());

    // Bounding box of the triangle, clamped to image bounds
    let min_x = v0.0.min(v1.0).min(v2.0).max(0.0) as u32;
    let max_x = (v0.0.max(v1.0).max(v2.0).ceil() as u32).min(w.saturating_sub(1));
    let min_y = v0.1.min(v1.1).min(v2.1).max(0.0) as u32;
    let max_y = (v0.1.max(v1.1).max(v2.1).ceil() as u32).min(h.saturating_sub(1));

    let denom = (v1.1 - v2.1) * (v0.0 - v2.0) + (v2.0 - v1.0) * (v0.1 - v2.1);
    if denom.abs() < f32::EPSILON {
        return; // Degenerate triangle
    }
    let inv_denom = 1.0 / denom;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let w0 = ((v1.1 - v2.1) * (px - v2.0) + (v2.0 - v1.0) * (py - v2.1)) * inv_denom;
            let w1 = ((v2.1 - v0.1) * (px - v2.0) + (v0.0 - v2.0) * (py - v2.1)) * inv_denom;
            let w2 = 1.0 - w0 - w1;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                // Interpolate color
                let r = (w0 * f32::from(c0[0]) + w1 * f32::from(c1[0]) + w2 * f32::from(c2[0]))
                    .clamp(0.0, 255.0) as u8;
                let g = (w0 * f32::from(c0[1]) + w1 * f32::from(c1[1]) + w2 * f32::from(c2[1]))
                    .clamp(0.0, 255.0) as u8;
                let b = (w0 * f32::from(c0[2]) + w1 * f32::from(c1[2]) + w2 * f32::from(c2[2]))
                    .clamp(0.0, 255.0) as u8;
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ScanMetadata;

    fn make_test_cloud() -> PointCloud {
        let ground: Vec<Point> = (0..100)
            .map(|i| {
                let x = (i % 10) as f32 * 0.5;
                let y = (i / 10) as f32 * 0.5;
                Point {
                    position: [x, y, 0.01 * (i % 3) as f32],
                    color: Some([0, 128, 0]),
                }
            })
            .collect();

        let obstacles: Vec<Point> = (0..20)
            .map(|i| Point {
                position: [1.0 + (i % 5) as f32 * 0.1, 1.0 + (i / 5) as f32 * 0.1, 0.5],
                color: Some([128, 0, 0]),
            })
            .collect();

        let bbox = BoundingBox {
            min: [0.0, 0.0, 0.0],
            max: [4.5, 4.5, 0.5],
        };

        PointCloud {
            metadata: ScanMetadata {
                bbox,
                original_count: 120,
                filtered_count: 120,
                ground_count: ground.len(),
                obstacle_count: obstacles.len(),
                ground_plane: crate::types::Plane {
                    normal: [0.0, 0.0, 1.0],
                    d: 0.0,
                },
            },
            ground,
            obstacles,
        }
    }

    #[test]
    fn test_generate_terrain_produces_outputs() {
        pt_test_utils::timed(|| {
            let cloud = make_test_cloud();
            let config = ExportConfig::default();
            let output = generate_terrain(&cloud, &config).unwrap();

            assert!(!output.mesh_glb.is_empty(), "GLB should not be empty");
            assert!(!output.plan_view_png.is_empty(), "PNG should not be empty");
            assert!(output.metadata.decimated_triangle_count > 0);
            assert!(output.metadata.vertex_count > 0);
        });
    }

    #[test]
    fn test_glb_magic_and_version() {
        pt_test_utils::timed(|| {
            let cloud = make_test_cloud();
            let output = generate_terrain(&cloud, &ExportConfig::default()).unwrap();

            let glb = &output.mesh_glb;
            assert!(glb.len() >= 12, "GLB too short for header");

            // Magic: "glTF"
            let magic = u32::from_le_bytes([glb[0], glb[1], glb[2], glb[3]]);
            assert_eq!(magic, GLB_MAGIC, "GLB magic mismatch");

            // Version: 2
            let version = u32::from_le_bytes([glb[4], glb[5], glb[6], glb[7]]);
            assert_eq!(version, 2, "GLB version should be 2");

            // Total length matches actual size
            let total_len = u32::from_le_bytes([glb[8], glb[9], glb[10], glb[11]]);
            assert_eq!(total_len as usize, glb.len(), "GLB total length mismatch");
        });
    }

    #[test]
    fn test_glb_json_chunk_parseable() {
        pt_test_utils::timed(|| {
            let cloud = make_test_cloud();
            let output = generate_terrain(&cloud, &ExportConfig::default()).unwrap();

            let glb = &output.mesh_glb;

            // JSON chunk: starts at byte 12
            let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
            let chunk_type = u32::from_le_bytes([glb[16], glb[17], glb[18], glb[19]]);
            assert_eq!(chunk_type, CHUNK_JSON, "first chunk should be JSON");

            let json_bytes = &glb[20..20 + json_len];
            let parsed: serde_json::Value =
                serde_json::from_slice(json_bytes).expect("JSON chunk should be valid JSON");

            // Verify key glTF fields
            assert_eq!(parsed["asset"]["version"], "2.0");
            assert!(parsed["meshes"].is_array());
            assert!(parsed["accessors"].is_array());
            assert!(parsed["bufferViews"].is_array());
        });
    }

    #[test]
    fn test_png_magic_bytes() {
        pt_test_utils::timed(|| {
            let cloud = make_test_cloud();
            let output = generate_terrain(&cloud, &ExportConfig::default()).unwrap();

            let png = &output.plan_view_png;
            // PNG magic: 0x89 "PNG" \r\n \x1a \n
            assert!(png.len() >= 8, "PNG too short");
            assert_eq!(&png[..4], &[0x89, 0x50, 0x4E, 0x47], "PNG magic bytes");
        });
    }

    #[test]
    fn test_metadata_consistency() {
        pt_test_utils::timed(|| {
            let cloud = make_test_cloud();
            let output = generate_terrain(&cloud, &ExportConfig::default()).unwrap();

            // original_point_count matches what we put in
            assert_eq!(output.metadata.original_point_count, 120);

            // triangle count and vertex count should be positive
            assert!(output.metadata.decimated_triangle_count > 0);
            assert!(output.metadata.vertex_count > 0);

            // elevation range should cover our ground points (z from 0.0 to 0.02)
            assert!(output.metadata.elevation_range[0] >= -0.01);
            assert!(output.metadata.elevation_range[1] <= 0.05);

            // processing time should be positive
            assert!(
                output.metadata.processing_time_ms < 10_000,
                "should finish under 10s"
            );
        });
    }
}
