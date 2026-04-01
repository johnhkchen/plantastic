# T-035-01 Design: BAML AnalyzePlanView

## Decision 1: Image parameter type

### Option A: BAML `image` native type
The BAML runtime has a native `image` type that handles multimodal API calls. The Rust type is `baml::Image`, constructed via `Image::from_base64("image/png", base64_data)`. The runtime sends the image as a proper content block in the API request.

**Pros:** Idiomatic BAML, runtime handles encoding, works with streaming.
**Cons:** The trait method signature needs `baml::Image` which couples the trait to BAML types.

### Option B: `string` (base64-encoded)
Pass the image as a raw base64 string. The prompt template references it inline.

**Pros:** Simpler trait signature, no BAML type dependency in trait.
**Cons:** Won't trigger multimodal API behavior — Claude needs the image as a proper content block, not inline text. Would require prompt engineering workarounds.

### Decision: Option A — BAML `image` native type

The whole point is multimodal analysis. BAML's `image` type is designed for this. The trait can accept `&[u8]` (raw PNG bytes) and the BAML implementation converts to `Image::from_base64()` internally, keeping the trait clean.

## Decision 2: Crate naming and location

### Option A: New `pt-analyzer` crate
Follows the terse naming convention (pt-features, pt-planter, pt-scan).

### Option B: Add to existing `pt-features` crate
AnalyzePlanView builds on feature classification — could extend that crate.

### Decision: Option A — New `pt-analyzer` crate

Site analysis is a distinct capability from feature classification. It takes classified features as *input* (not the same pipeline stage). Separate crate maintains single-responsibility and matches the existing pattern where each BAML function gets its own crate.

## Decision 3: Type definitions in BAML

### Approach
Define new types in `baml_src/analyze.baml`:
- `SuggestedZone`: label, zone_type, rationale, approximate_area_sqft
- `SiteObservation`: observation (free-form string)
- `SiteAnalysis`: features (ClassifiedFeature[]), suggested_zones (SuggestedZone[]), site_observations (SiteObservation[])

The `ClassifiedFeature` type is already defined in `classify.baml` and can be referenced directly — BAML resolves cross-file references within `baml_src/`.

The `features` field in `SiteAnalysis` echoes back classified features. This is useful for the API contract but the mock can simply return the same features passed in.

**Note:** The ticket says `features[]` but doesn't specify a new type for analyzed features vs classified features. Since the analysis builds on classification, reusing `ClassifiedFeature[]` is appropriate. If the analysis adds per-feature insights, we can add an `AnalyzedFeature` wrapper later.

## Decision 4: Trait method signature

```rust
#[async_trait]
pub trait SiteAnalyzer: Send + Sync {
    async fn analyze(
        &self,
        plan_view_png: &[u8],
        lot_dimensions: &str,
        address: &str,
        classified_features: &[ClassifiedFeature],
    ) -> Result<SiteAnalysis, AnalyzerError>;
}
```

The trait takes raw PNG bytes (`&[u8]`) rather than `baml::Image` so it's not coupled to BAML types. The BAML implementation converts to `Image::from_base64()` internally. The mock ignores the image entirely and returns a fixture.

## Decision 5: Mock fixture design

The mock returns a realistic Powell & Market site analysis:
- **Suggested zones** (4): Entry Patio, Shade Garden, Street Buffer Planting, Seating Alcove
- **Site observations** (3): shade corridor from London Planes, grade change near curb, high foot traffic pattern
- **Features**: echoes back a hardcoded set matching the classify mock output

This matches the pattern in `pt-planter/src/mock.rs` where the fixture is hand-crafted with real SF Bay Area context.

## Decision 6: Prompt design

The prompt must emphasize spatial reasoning — this is where Claude's multimodal capability shines. Key elements:
- Reference the annotated plan view image directly (BAML `image` type)
- List classified features with their locations for cross-referencing
- Ask for zone suggestions that reference spatial relationships ("between Feature 3 and Feature 7")
- Request site observations that demonstrate landscape expertise
- Lot dimensions provide scale context for area estimation

## Decision 7: Scenario test

No existing scenario covers AI site analysis. This is a new capability that belongs in the Design value area. We should add it as a new scenario test function in `design.rs` but NOT add it to the SCENARIOS array yet — that would require incrementing the array size and assigning a scenario ID, which should be done when the capability is fully integrated. Instead, we add a unit test in the crate itself.

Actually — the ticket's acceptance criteria say "Run on Powell & Market annotated plan view, capture fixture." This means we need a test that exercises the mock on realistic input. Following the S.2.3 pattern, we add this as a crate-level test and verify the mock produces correct output.

## Rejected alternatives

1. **Adding AnalyzePlanView to pt-features crate**: Violates single-responsibility. Site analysis is a consumer of feature classification, not part of it.
2. **Using string for image**: Would break multimodal API calls. Claude needs proper image content blocks.
3. **Skipping the ClaudeCliAnalyzer**: Consistency with the three-tier pattern is important for the codebase.
4. **Using a complex AnalyzedFeature wrapper type**: Over-engineering for the current ticket. ClassifiedFeature[] is sufficient.
