# T-034-01 Design: BAML ClassifyFeatures

## Decision: New `pt-features` crate following the pt-proposal pattern

### Option A: Add classification to pt-scan

- Pro: No new crate, less workspace overhead
- Con: pt-scan is sync, BAML client is async. Would need tokio dep in pt-scan, which is a pure compute crate. Violates the clean separation between geometry (no I/O) and LLM (async I/O).
- **Rejected**: Wrong abstraction boundary.

### Option B: New `pt-features` crate

- Pro: Same pattern as pt-proposal. Clean separation: pt-scan does geometry, pt-features does LLM classification. Can depend on pt-scan for the `FeatureCandidate` type.
- Pro: Three implementations (BAML, CLI, mock) parallel pt-proposal exactly.
- Con: One more crate in the workspace.
- **Chosen**: Follows established patterns, correct boundary.

### Option C: Extend pt-proposal with classification

- Pro: BAML client already included.
- Con: Proposal and classification are different concerns. Would create a grab-bag crate.
- **Rejected**: Wrong cohesion.

## BAML Schema Design

### New file: `baml_src/classify.baml`

**Input**: Mirror `FeatureCandidate` fields exactly. BAML generates its own struct — we convert from `pt_scan::FeatureCandidate` to `baml_client::types::FeatureCandidateInput`.

**Output**: `ClassifiedFeature` per the ticket AC:
- `cluster_id`: int — ties back to input
- `label`: string — human-readable ("London Plane Tree", "Street Light Pole")
- `category`: string — enum-like ("tree", "structure", "hardscape", "planting", "utility")
- `species`: string? — botanical name, null for non-vegetation
- `confidence`: float — 0.0–1.0
- `reasoning`: string — the "wow" text explaining the classification
- `landscape_notes`: string — design implications

**Function**: `ClassifyFeatures(candidates, address, climate_zone) -> ClassifiedFeature[]`

**Client**: `ProposalFallback` — reuse the existing fallback client. No need for a separate one; classification needs the same intelligence level as proposal writing.

### Prompt Design

The prompt must:
1. Establish the LLM as a landscape design expert with arborist knowledge
2. Provide address and climate zone for regional inference
3. Present candidates as structured JSON (not raw points — already <5KB)
4. Request calibrated confidence: distinctive profiles get high confidence, ambiguous ones lower
5. Instruct species identification from geometric cues + regional knowledge
6. For Powell & Market specifically: vertical cylinders + brown color + SF Market Street = London Plane or Brisbane Box

### Type Conversion Strategy

`pt_scan::FeatureCandidate` → `baml_client::types::FeatureCandidateInput`:
- Simple field-by-field mapping
- Implement `From<&pt_scan::FeatureCandidate>` on the BAML input type
- Centroid and bbox arrays become BAML `float[]` (BAML handles this)

### Trait Design

```rust
#[async_trait]
pub trait FeatureClassifier: Send + Sync {
    async fn classify(
        &self,
        candidates: &[pt_scan::FeatureCandidate],
        address: &str,
        climate_zone: &str,
    ) -> Result<Vec<ClassifiedFeature>, ClassificationError>;
}
```

Three implementations:
1. **`BamlFeatureClassifier`**: Converts candidates to BAML types, calls `B.ClassifyFeatures.call()`, returns generated `ClassifiedFeature` vec
2. **`ClaudeCliClassifier`**: Builds JSON prompt manually, calls `claude` CLI, parses with `B.ClassifyFeatures.parse()`
3. **`MockFeatureClassifier`**: Deterministic mapping based on candidate geometry — tall+brown+columnar → tree, short+flat+gray → hardscape, etc.

### Mock Fixture Strategy

The mock should produce plausible classifications based on geometric cues:
- `height_ft > 8 && vertical_profile == "columnar"` → tree trunk
- `height_ft > 15 && spread_ft > 10` → mature tree
- `height_ft < 2 && vertical_profile == "flat"` → hardscape/curb
- `height_ft > 10 && spread_ft < 2` → utility pole
- Otherwise → structure with moderate confidence

This makes mock tests meaningful — they verify the pipeline works end-to-end with structurally plausible output.

### Testing Strategy

1. **Unit tests in pt-features**: MockFeatureClassifier with synthetic candidates, verify output structure and field constraints
2. **Integration test**: Run mock classifier on Powell & Market candidates (from real scan), verify the two largest clusters are classified as trees
3. **No real LLM calls in CI**: Only MockFeatureClassifier runs in `just test`
4. **Fixture capture**: Powell & Market classifications from a real BAML call saved as JSON fixture for future regression testing

### Powell & Market Validation

The ticket requires: "two trunk clusters → both classified as 'tree trunk' with high confidence."

With the mock: the two largest clusters from the scan are tall (>8 ft), columnar, brown — the mock rules will classify them as tree trunks with high confidence. This validates the pipeline geometry-to-classification flow.

With real BAML (manual/dev): the prompt includes "Powell & Market Streets, San Francisco" + "USDA 10b" — the LLM should identify London Plane or Brisbane Box from the columnar profile + Market Street context.

## Open Questions

1. **BAML array types for centroid/bbox**: Need to verify BAML handles `float[]` for fixed-length arrays. May need to flatten to individual fields (centroid_x, centroid_y, centroid_z) if BAML array support is limited.
2. **Category enum vs string**: BAML supports enums, but using a string with documented valid values keeps the prompt simpler and avoids LLM formatting errors. Starting with string, can tighten later.
