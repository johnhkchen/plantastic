# T-034-02 Progress: Annotated Plan View

## Completed Steps

### Step 1: Add Font Asset
- Downloaded DejaVu Sans Mono TTF to `assets/fonts/DejaVuSansMono.ttf` (340KB)
- Verified: TrueType font, Apache-2.0 license

### Step 2: Add Dependencies
- Added `ab_glyph = "0.2"` and `imageproc = "0.25"` to pt-scan deps
- Added `pt-features` and `tokio` to pt-scan dev-deps (for CLI example)
- `cargo check -p pt-scan` passes

### Step 3: Make `world_to_pixel` Crate-Visible
- Changed `fn` to `pub(crate) fn` in `export.rs:373`
- No external API change

### Step 4: Create `annotate.rs` — Types
- `FeatureAnnotation` struct with label, category, bbox, centroid
- `AnnotationConfig` with defaults (font_scale=14, box_line_width=2, label_padding=4)
- `ClassifiedFeatureRef` trait to avoid direct baml_client dependency
- `feature_annotation()` constructor joining candidate + classified
- `category_color()` mapping: tree=#22c55e, structure=#6b7280, hardscape=#d97706, utility=#ef4444

### Step 5: Create `annotate.rs` — Rendering Functions
- `annotate_plan_view_png()`: decode base → draw overlays → re-encode
- Per feature: tinted bbox fill (15% alpha), outline (2px), label with bg, category icon
- Label collision avoidance: greedy shift-down, up to 5 attempts
- Category icons: circle for tree, square for structure/hardscape, diamond for utility

### Step 6: Wire Module into `lib.rs`
- Added `pub mod annotate;`
- Re-exported all public types and functions

### Step 7: Write Unit Tests
- 7 tests, all passing:
  - `test_category_colors` — all 5 categories + fallback
  - `test_feature_annotation_from_pair` — label formatting, bbox/centroid extraction
  - `test_annotate_empty_features` — valid PNG, same dimensions
  - `test_annotate_with_features` — valid PNG, different from base, dimensions preserved
  - `test_annotation_config_defaults` — expected default values
  - `test_resolve_label_no_collision` — position unchanged with no existing labels
  - `test_resolve_label_with_collision` — position shifted down correctly

### Step 8: Extend CLI Example
- Added Stage 8: mock classification → annotation → write annotated PNG
- Uses `MockFeatureClassifier` from pt-features
- `ClassifiedFeatureAdapter` bridges pt-features types to `ClassifiedFeatureRef` trait
- Outputs `{stem}-planview-annotated.png` alongside plain `{stem}-planview.png`
- Prints classified features table and timing

### Step 9: Quality Gate
- `cargo fmt -p pt-scan` — clean
- `cargo clippy -p pt-scan -- -D warnings` — clean
- `cargo test -p pt-scan --lib` — 55/55 pass (0.08s)
- Scenario dashboard: 83.5 min / 240.0 min (34.8%) — no regression

### Deviation: pt-planter stub
- `crates/pt-planter/src/lib.rs` was missing (pre-existing issue, broke workspace compilation)
- Created minimal stub with `mod error;` to unblock build

## Remaining
None — all steps complete.
