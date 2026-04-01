# T-034-02 Structure: Annotated Plan View

## File Changes Overview

```
CREATED:
  crates/pt-scan/src/annotate.rs       — annotation module (~200 lines)
  assets/fonts/DejaVuSansMono.ttf      — embedded font file (~335KB)

MODIFIED:
  crates/pt-scan/Cargo.toml            — add imageproc, ab_glyph deps
  crates/pt-scan/src/lib.rs            — add `pub mod annotate;` + re-exports
  crates/pt-scan/src/export.rs         — make world_to_pixel() pub(crate)
  crates/pt-scan/examples/process_sample.rs — add annotation stage
```

## New Module: `crates/pt-scan/src/annotate.rs`

### Public Types

```rust
/// Spatial + label data for one feature annotation overlay.
pub struct FeatureAnnotation {
    pub label: String,        // formatted: "London Plane (0.92)"
    pub category: String,     // "tree", "structure", "hardscape", "utility"
    pub bbox_min: [f64; 2],   // world XY (from FeatureCandidate)
    pub bbox_max: [f64; 2],   // world XY
    pub centroid: [f64; 2],   // world XY
}

/// Configuration for annotation rendering.
pub struct AnnotationConfig {
    pub font_scale: f32,          // default: 14.0
    pub box_line_width: u32,      // default: 2
    pub label_padding: u32,       // default: 4
}
```

### Public Functions

```rust
/// Build a FeatureAnnotation by joining candidate geometry with classified labels.
pub fn feature_annotation(
    candidate: &FeatureCandidate,
    classified: &ClassifiedFeature,
) -> FeatureAnnotation

/// Overlay annotations on a base plan-view PNG.
/// Returns annotated PNG bytes.
pub fn annotate_plan_view_png(
    base_png: &[u8],
    features: &[FeatureAnnotation],
    bbox: &BoundingBox,
    config: &AnnotationConfig,
) -> Result<Vec<u8>, ScanError>
```

### Internal Functions

```rust
fn category_color(category: &str) -> Rgb<u8>
fn draw_bbox_outline(img, min_px, max_px, color, line_width)
fn draw_label(img, text, position, bg_color, font, scale, placed_labels) -> Rect
fn resolve_label_position(default_pos, label_size, placed_labels) -> (i32, i32)
```

### Module Boundary

- `annotate.rs` depends on: `image`, `imageproc`, `ab_glyph`, `crate::types::BoundingBox`, `crate::feature::FeatureCandidate`, `crate::error::ScanError`
- `annotate.rs` does NOT depend on: `baml_client` types directly (receives `FeatureAnnotation`, not `ClassifiedFeature`)
- `annotate.rs` uses `crate::export::world_to_pixel()` — requires making it `pub(crate)`

## Dependency Changes: `crates/pt-scan/Cargo.toml`

```toml
# Add:
imageproc = "0.25"
ab_glyph = "0.2"
```

`imageproc` 0.25 is compatible with `image` 0.25 (same major). `ab_glyph` is a transitive dep of `imageproc` but we need it directly for `FontRef`.

## Visibility Change: `export.rs`

Change `world_to_pixel()` from `fn` to `pub(crate) fn`. No external API change — it's only used within the pt-scan crate.

## Re-exports: `lib.rs`

```rust
pub mod annotate;
pub use annotate::{annotate_plan_view_png, feature_annotation, AnnotationConfig, FeatureAnnotation};
```

## Font Asset

`assets/fonts/DejaVuSansMono.ttf` — downloaded from DejaVu project. Loaded via:

```rust
const FONT_BYTES: &[u8] = include_bytes!("../../../assets/fonts/DejaVuSansMono.ttf");
```

The `include_bytes!` path is relative to `annotate.rs` location (`crates/pt-scan/src/`), going up 3 levels to repo root then into `assets/fonts/`.

## CLI Example Extension

In `examples/process_sample.rs`, after the existing Stage 6 (clustering + candidates):

```
Stage 8: Annotation
  - Import MockFeatureClassifier from pt-features
  - Classify candidates
  - Build Vec<FeatureAnnotation> from candidate/classified pairs
  - Call annotate_plan_view_png()
  - Write {stem}-planview-annotated.png
```

This requires adding `pt-features` as a dev-dependency of pt-scan (for the example only), or making pt-features an optional dependency. Better approach: since the example is just a demo, add pt-features to `[dev-dependencies]` — examples can use dev-deps.

## Test Location

Tests go in `annotate.rs` as `#[cfg(test)] mod tests {}`, following the crate convention. They use the existing `make_test_cloud()` pattern from `export.rs` tests but don't need real point cloud data — they can create synthetic images and annotations.

## Ordering of Changes

1. Add font asset file
2. Add dependencies to Cargo.toml
3. Change `world_to_pixel` visibility in export.rs
4. Create annotate.rs with types + functions + tests
5. Add module + re-exports to lib.rs
6. Extend CLI example
7. Run `just check`
