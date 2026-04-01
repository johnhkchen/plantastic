# T-034-02 Research: Annotated Plan View

## Objective

Overlay classified feature labels, bounding boxes, and category-coded colors onto the existing plan-view PNG, producing a professional-looking annotated site plan alongside the plain one.

## Current Plan View Pipeline

### `to_plan_view_png()` — `crates/pt-scan/src/export.rs:282-369`

Private function. Signature:

```rust
fn to_plan_view_png(
    mesh: &TerrainMesh,
    obstacles: &[Point],
    bbox: &BoundingBox,
    config: &ExportConfig,
) -> Result<Vec<u8>, ScanError>
```

Pipeline:
1. Compute image dimensions from bbox × `pixels_per_meter` (capped at 4096×4096)
2. Fill light-gray background `[220, 220, 220]`
3. Rasterize mesh triangles via barycentric interpolation with elevation shading
4. Optionally darken obstacle points (3×3 pixels, 40% intensity reduction)
5. Encode to PNG via `image::codecs::png::PngEncoder`

### Coordinate Mapping — `world_to_pixel()` (line 373)

World XY → pixel: `nx = (x - bbox.min[0]) / width`, `px = nx * (img_w - 1)`. Y is flipped.

### Caller: `generate_terrain()` (line 68)

Returns `TerrainOutput { mesh_glb, plan_view_png, metadata }`. Currently produces one PNG only.

### CLI Example — `examples/process_sample.rs`

7-stage pipeline: parse → downsample → outlier → RANSAC → terrain → cluster → gaps. Writes `{stem}-planview.png`. Does NOT currently run classification (that's in pt-features, a separate crate).

## Classification Data Available

### `FeatureCandidate` — `crates/pt-scan/src/feature.rs:20-31`

```rust
pub struct FeatureCandidate {
    pub cluster_id: usize,
    pub centroid: [f64; 3],       // world coords
    pub bbox_min: [f64; 3],       // world coords
    pub bbox_max: [f64; 3],       // world coords
    pub height_ft: f64,
    pub spread_ft: f64,
    pub point_count: usize,
    pub dominant_color: String,
    pub vertical_profile: String,
    pub density: f64,
}
```

Key: `bbox_min` / `bbox_max` and `centroid` are in world coordinates — directly mappable via `world_to_pixel()`. This is the bounding box source for drawing overlays.

### `ClassifiedFeature` — `baml_client/types/classes.rs:11-27`

```rust
pub struct ClassifiedFeature {
    pub cluster_id: i64,
    pub label: String,           // "London Plane Tree"
    pub category: String,        // "tree", "structure", "hardscape", "utility"
    pub species: Option<String>,
    pub confidence: f64,         // 0.0–1.0
    pub reasoning: String,
    pub landscape_notes: String,
}
```

Key gap: `ClassifiedFeature` does NOT carry spatial data (bbox/centroid). Must join with `FeatureCandidate` on `cluster_id` to get coordinates.

### `Cluster` — `crates/pt-scan/src/cluster.rs:31-41`

Has `centroid: [f32; 3]` and `bbox: BoundingBox` — same spatial data as `FeatureCandidate`, but in f32. Either source works for annotation.

## Font Rendering Dependencies

### Current State

pt-scan depends on `image = "0.25"` (PNG feature only). No text rendering crate exists.

### Options

| Crate | Size | Approach | image crate compat |
|-------|------|----------|-------------------|
| `imageproc` 0.25 | ~400KB | Full image processing, `draw_text_mut` | Direct `image` 0.25 integration |
| `ab_glyph` 0.2 | ~150KB | Font parsing + rasterization | Works with `imageproc` |
| `fontdue` 0.9 | ~100KB | Pure-Rust font rasterizer | Manual pixel blitting |
| `rusttype` | deprecated | — | — |

`imageproc` + `ab_glyph` is the standard combo for text-on-image in Rust. `imageproc::drawing::draw_text_mut` takes a font reference, scale, position, color, and string — exactly what we need.

### Font Embedding

DejaVu Sans Mono is Apache-2.0 licensed, ~350KB TTF. Embed as `include_bytes!` or ship in `assets/fonts/`. The ticket specifically suggests this font.

## Color Palette (from ticket)

| Category | Hex | RGB |
|----------|-----|-----|
| tree | #22c55e | (34, 197, 94) |
| structure | #6b7280 | (107, 114, 128) |
| hardscape | #d97706 | (217, 119, 6) |
| utility | #ef4444 | (239, 68, 68) |
| planting (unlisted) | #22c55e | Same as tree |
| unknown | #9ca3af | (156, 163, 175) — fallback |

## Label Collision Avoidance

The ticket calls for "simple collision avoidance (shift down if overlap)". Strategy:
- Maintain a list of placed label rectangles `Vec<(x_min, y_min, x_max, y_max)>`
- For each new label, check overlap with all placed labels
- If overlapping, shift down by label height + padding
- Cap iterations to prevent infinite loop on dense clusters

## API Integration Point

`plantastic-api/src/routes/scan.rs:219-239` — `run_scan_pipeline()` currently calls `generate_terrain()` and uploads the PNG. Annotated PNG integration would happen here, but that's outside this ticket's scope (this ticket is pt-scan + CLI only).

## Existing Tests

- `export::tests::test_png_magic_bytes` — validates PNG output has correct header
- `export::tests::test_generate_terrain_produces_outputs` — asserts non-empty outputs
- No tests for annotation overlay (new territory)

## Scenario Impact

No scenario directly tests annotated plan view. S.1.4 (plant identification) and S.2.1 (site assessment) are adjacent but test pipeline flow, not image rendering. This ticket is a visual/UX enhancement — verification is "does the PNG look right" which is inherently manual, but we can test that the annotated PNG is produced, is valid, and is larger than the plain one.
