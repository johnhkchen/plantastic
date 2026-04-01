# T-034-02 Design: Annotated Plan View

## Decision Summary

Create a new public function `annotate_plan_view_png()` that takes a base plan-view image plus paired classification/candidate data and returns annotated PNG bytes. Keep `to_plan_view_png()` unchanged. Add `imageproc` + `ab_glyph` for text rendering. Embed DejaVu Sans Mono TTF.

## Options Evaluated

### Option A: Extend `to_plan_view_png()` with optional params

Add `Option<&[AnnotationOverlay]>` parameter to the existing function.

- **Pro:** Single rendering pass, no double PNG encode/decode
- **Con:** Bloats a clean function with annotation logic. Forces all callers to handle the option. Mixes concerns (terrain raster vs vector overlay).

### Option B: New function that takes base PNG + overlays (CHOSEN)

New `annotate_plan_view_png(base_png, features, bbox, config)` that decodes the base PNG, draws overlays, re-encodes.

- **Pro:** Clean separation. Base plan view is unchanged. Annotation is purely additive. Can annotate any PNG (not just freshly rendered ones). Easy to test independently.
- **Con:** One extra PNG decode/encode cycle. Negligible cost (<10ms for a 4096×4096 image).

### Option C: Render annotations directly onto the `ImageBuffer` before encoding

Pass the mutable `ImageBuffer` from `to_plan_view_png` to an annotation function before final encode.

- **Pro:** Avoids decode/re-encode. Slightly more efficient.
- **Con:** Requires restructuring `to_plan_view_png` to expose its internal `ImageBuffer`, or making the annotation function private and calling it from within export.rs. Tight coupling.

**Decision: Option B.** The decode/encode overhead is negligible, and the clean separation is worth it. The function can be used on any plan-view PNG — including ones loaded from S3 for re-annotation after reclassification. The function is pure (bytes in → bytes out) which makes testing trivial.

## Data Bridge: FeatureCandidate + ClassifiedFeature → AnnotationOverlay

`ClassifiedFeature` has labels but no spatial data. `FeatureCandidate` has spatial data but no labels. We need both. Rather than forcing callers to do the join, define a simple struct:

```rust
pub struct FeatureAnnotation {
    pub label: String,           // "London Plane (0.92)"
    pub category: String,        // "tree"
    pub bbox_min: [f64; 2],      // world XY
    pub bbox_max: [f64; 2],      // world XY
    pub centroid: [f64; 2],      // world XY
}
```

Plus a constructor `FeatureAnnotation::from_pair(candidate, classified)` that joins on cluster_id and formats the label string. This keeps the annotation function generic — it doesn't need to know about BAML types.

## Font Strategy

Embed DejaVu Sans Mono as `include_bytes!("../../assets/fonts/DejaVuSansMono.ttf")`. The TTF is ~335KB, well within binary size budget. Apache-2.0 license is compatible.

Use `ab_glyph::FontRef::try_from_slice()` to parse at runtime (cheap, ~1ms). Pass to `imageproc::drawing::draw_text_mut()`.

Font size: scale dynamically based on image resolution. Target ~12px at 30 ppm (standard), proportionally larger at higher resolutions. Minimum 8px.

## Annotation Rendering Strategy

Per feature, in order:
1. **Bounding box rectangle** — 2px outline in category color, 20% alpha fill
2. **Label background** — small opaque rectangle behind text for legibility
3. **Label text** — `"{label} ({confidence:.2})"` in white on the background
4. **Category icon** (optional) — circle for tree, square for structure at centroid

### Label Collision Avoidance

Greedy algorithm:
1. Compute default label position: centered above bbox top edge
2. Check against all previously placed labels
3. If overlapping, shift down by (label_height + 4px)
4. Repeat up to 5 times, then place anyway (rare edge case)

This is O(n²) in feature count but n is typically <20 — negligible.

## Output Integration

### `TerrainOutput` extension

Add `annotated_plan_view_png: Option<Vec<u8>>` to `TerrainOutput`. When classification data is available, this field is populated. When not (e.g., scan without classification), it's `None`.

### `generate_terrain()` stays unchanged

The existing function doesn't take classification data and shouldn't. Instead, annotation happens as a post-processing step: caller generates terrain, runs classification, then calls `annotate_plan_view_png()`.

### CLI example extension

Add an optional classification stage to `process_sample.rs`:
- After clustering + candidate extraction (already done)
- Run `MockFeatureClassifier` (no real LLM in CLI)
- Call `annotate_plan_view_png()` with results
- Write `{stem}-planview-annotated.png`

## Testing Strategy

1. **Unit test: annotation produces valid PNG** — create a small test image, annotate with known features, verify PNG header and size > base
2. **Unit test: label formatting** — verify `FeatureAnnotation::from_pair()` joins correctly and formats labels
3. **Unit test: category color mapping** — verify each category maps to the correct RGB
4. **Unit test: empty features** — annotating with no features returns a PNG identical to input
5. **Integration test in CLI example** — visual verification (not automated, but the "wow" screenshot goal)

## Rejected Alternatives

- **SVG overlay instead of raster**: Would be better for scalability but adds SVG rendering dependency and doesn't integrate with existing PNG pipeline
- **Canvas-based rendering in browser**: Out of scope — this is backend image generation
- **fontdue instead of ab_glyph**: fontdue is lower-level; `imageproc` already wraps `ab_glyph`, so using fontdue means reimplementing `draw_text_mut`
- **Storing spatial data in ClassifiedFeature**: Would require BAML schema change and break the clean separation between scan geometry and LLM classification
