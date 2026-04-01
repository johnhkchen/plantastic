# T-034-02 Review: Annotated Plan View

## Summary

Added annotated plan-view PNG generation to pt-scan. Given classified feature data (labels, categories, bounding boxes), produces a professional site-plan overlay with color-coded bounding boxes, labels with confidence scores, and category icons. The annotation function is decoupled from the base plan-view renderer — it takes any PNG as input, overlays annotations, and returns annotated PNG bytes.

## Files Created

| File | Purpose | Lines |
|------|---------|-------|
| `crates/pt-scan/src/annotate.rs` | Annotation module: types, rendering, tests | ~500 |
| `assets/fonts/DejaVuSansMono.ttf` | Embedded monospace font for labels | 340KB |
| `crates/pt-planter/src/lib.rs` | Stub to fix pre-existing broken workspace | 2 |

## Files Modified

| File | Change |
|------|--------|
| `crates/pt-scan/Cargo.toml` | Added `ab_glyph`, `imageproc` deps; `pt-features`, `tokio` dev-deps |
| `crates/pt-scan/src/lib.rs` | Added `pub mod annotate;` + re-exports |
| `crates/pt-scan/src/export.rs` | Changed `world_to_pixel()` from `fn` to `pub(crate) fn` |
| `crates/pt-scan/examples/process_sample.rs` | Added Stage 8: mock classification + annotation + write annotated PNG |

## Architecture Decisions

1. **Separate function, not extended existing**: `annotate_plan_view_png()` is a new public function that takes base PNG bytes + overlay data. This avoids modifying `to_plan_view_png()` or coupling terrain generation to classification. Any PNG can be annotated — including ones loaded from S3 for re-annotation.

2. **`ClassifiedFeatureRef` trait**: The annotation module defines a trait instead of depending on baml_client types directly. This keeps pt-scan free of BAML dependencies while allowing any classified feature type to implement the trait.

3. **`FeatureAnnotation` as bridge type**: Combines spatial data (from `FeatureCandidate`) with display data (from `ClassifiedFeature`) into a single struct. Callers do the join; the renderer stays generic.

4. **Font embedded via `include_bytes!`**: DejaVu Sans Mono is bundled in the binary (~340KB). No runtime font discovery needed. Works in Lambda, CI, and local dev.

## Test Coverage

### Unit Tests (7 tests in annotate.rs)

| Test | What it verifies |
|------|------------------|
| `test_category_colors` | All 5 categories + unknown fallback map to correct RGB |
| `test_feature_annotation_from_pair` | Label format `"Name (0.92)"`, bbox/centroid extraction |
| `test_annotate_empty_features` | Empty features → valid PNG with same dimensions |
| `test_annotate_with_features` | Non-empty → valid PNG, different from base, dimensions preserved |
| `test_annotation_config_defaults` | Default config: scale=14, line_width=2, padding=4 |
| `test_resolve_label_no_collision` | Label placed at default position when no conflicts |
| `test_resolve_label_with_collision` | Label shifted down by height+4 when overlapping |

### Gaps

- No pixel-level rendering verification (intentional — font rendering is platform-dependent and brittle to test)
- No visual regression test framework (would need a baseline image and perceptual diff)
- CLI example not tested in CI (requires PLY input file)

## Scenario Dashboard

Before: 83.5 min / 240.0 min (34.8%)
After: 83.5 min / 240.0 min (34.8%)

No regression. This ticket is a visual/UX enhancement — it produces annotated PNGs for sales decks and README. No scenario directly measures "annotated plan view exists" but the infrastructure supports future S.1.4 (plant identification) visualization.

## Quality Gate

- `cargo fmt -p pt-scan` — pass
- `cargo clippy -p pt-scan -- -D warnings` — pass (zero warnings)
- `cargo test -p pt-scan --lib` — 55/55 pass
- `cargo run -p pt-scenarios` — no regressions

## Acceptance Criteria Status

| AC | Status |
|----|--------|
| Extend pt-scan to accept ClassifiedFeature[] | Done — `annotate_plan_view_png()` accepts `&[FeatureAnnotation]` |
| Bounding box outline (color-coded by category) | Done — 2px outline + 15% tinted fill |
| Label text: feature name + confidence | Done — format: `"London Plane (0.92)"` |
| Optional category icon | Done — circle for tree, square for structure, diamond for utility |
| Output annotated PNG alongside plain one | Done — separate function, caller controls both outputs |
| CLI example extended | Done — Stage 8 in process_sample.rs |
| Professional site survey quality | Implemented — full visual verification requires running with real PLY data |

## Open Concerns

1. **Binary size**: `imageproc` + `ab_glyph` + font add ~700KB to the pt-scan binary. Acceptable for Lambda but worth noting.

2. **pt-planter stub**: Created `crates/pt-planter/src/lib.rs` as a minimal stub to fix a pre-existing broken workspace. This is outside ticket scope but was required to compile.

3. **Font licensing**: DejaVu Sans Mono is Apache-2.0 + Bitstream Vera license (permissive). Compatible with project license. The font file should be tracked in git (340KB is fine).

4. **Real PLY demo**: The "wow" screenshot goal requires running the CLI example with a real scan. Not automated, but the pipeline is complete: `cargo run -p pt-scan --example process_sample --release -- path/to/scan.ply` will produce both plain and annotated plan views.
