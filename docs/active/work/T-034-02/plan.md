# T-034-02 Plan: Annotated Plan View

## Step 1: Add Font Asset

- Download DejaVu Sans Mono TTF to `assets/fonts/DejaVuSansMono.ttf`
- Verify file exists and is valid TTF (~335KB)

**Verify:** `file assets/fonts/DejaVuSansMono.ttf` shows TrueType font

## Step 2: Add Dependencies

Edit `crates/pt-scan/Cargo.toml`:
- Add `imageproc = "0.25"`
- Add `ab_glyph = "0.2"`
- Add `pt-features` to `[dev-dependencies]` (for example only)

**Verify:** `cargo check -p pt-scan` passes

## Step 3: Make `world_to_pixel` Crate-Visible

In `crates/pt-scan/src/export.rs`, change:
```rust
fn world_to_pixel(...) -> (f32, f32)
```
to:
```rust
pub(crate) fn world_to_pixel(...) -> (f32, f32)
```

**Verify:** No change in external API; `cargo check -p pt-scan` passes

## Step 4: Create `annotate.rs` — Types

Create `crates/pt-scan/src/annotate.rs` with:
- `FeatureAnnotation` struct (label, category, bbox_min/max [f64;2], centroid [f64;2])
- `AnnotationConfig` struct with defaults (font_scale=14.0, box_line_width=2, label_padding=4)
- `feature_annotation()` constructor that joins `FeatureCandidate` + `ClassifiedFeature` on cluster_id
- `category_color()` mapping function

**Verify:** Types compile

## Step 5: Create `annotate.rs` — Rendering Functions

Add to `annotate.rs`:
- `annotate_plan_view_png(base_png, features, bbox, config) -> Result<Vec<u8>, ScanError>`
  - Decode base PNG to ImageBuffer
  - For each feature: draw bbox outline, then label with collision avoidance
  - Re-encode to PNG
- Internal helpers: `draw_bbox_rect()`, `draw_label_with_bg()`, `resolve_label_position()`
- Font loading via `include_bytes!` + `ab_glyph::FontRef`

**Verify:** `cargo check -p pt-scan` passes

## Step 6: Wire Module into `lib.rs`

Add to `crates/pt-scan/src/lib.rs`:
- `pub mod annotate;`
- Re-export key types and functions

**Verify:** `cargo check -p pt-scan` passes

## Step 7: Write Unit Tests

In `annotate.rs` `#[cfg(test)]` block:
1. `test_category_colors` — each category maps to correct RGB
2. `test_feature_annotation_from_pair` — correct label formatting, cluster_id join
3. `test_annotate_empty_features` — returns valid PNG, same dimensions as input
4. `test_annotate_with_features` — returns valid PNG larger than base, PNG header intact
5. `test_annotation_config_defaults` — default config has expected values

**Verify:** `cargo test -p pt-scan -- annotate` all pass

## Step 8: Extend CLI Example

In `examples/process_sample.rs`:
- Add Stage 8 after gap measurement
- Use `MockFeatureClassifier` to classify candidates
- Build `Vec<FeatureAnnotation>` from pairs
- Call `annotate_plan_view_png()` on the terrain plan_view_png
- Write `{stem}-planview-annotated.png`
- Print stage timing and output file info

**Verify:** `cargo build -p pt-scan --example process_sample` compiles

## Step 9: Quality Gate

- `just fmt` — format code
- `just lint` — clippy strict
- `just test` — all workspace tests pass
- `just scenarios` — no regressions

**Verify:** `just check` passes clean

## Testing Strategy

### Unit Tests (in annotate.rs)
- Category color mapping: exhaustive for all 4 categories + unknown fallback
- Label construction: verify format `"{label} ({confidence:.2})"`, cluster_id matching
- Empty annotation: no features → output PNG equals input PNG in dimensions
- Non-empty annotation: output PNG > input PNG in byte size (annotations add data)
- PNG validity: output starts with PNG magic bytes

### Integration (CLI example)
- Visual inspection of annotated output — this is the "wow" screenshot
- Not automated (image content is inherently visual), but the tests above ensure correctness of the rendering pipeline

### What we do NOT test
- Font rendering pixel-perfect output (too brittle, platform-dependent)
- Specific pixel colors at coordinates (fragile)
- Collision avoidance edge cases beyond basic correctness
