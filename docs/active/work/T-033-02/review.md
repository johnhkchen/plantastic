# T-033-02 Review: Feature Candidates

## Summary

Added `FeatureCandidate` extraction from DBSCAN clusters in the pt-scan crate.
Each cluster is summarized into a structured geometric description suitable for
BAML LLM classification — the LLM never sees raw point data.

## Files Changed

### Created
- `crates/pt-scan/src/feature.rs` (290 lines) — new module with:
  - `FeatureCandidate` struct (10 fields matching ticket AC, serde Serialize)
  - `extract_candidates(clusters, points, ground_plane)` public function
  - Internal helpers: height, spread, density, color classification, vertical profile
  - Conical taper detection (upper/lower half XY spread comparison)
  - 11 unit tests

### Modified
- `crates/pt-scan/src/lib.rs` — added `pub mod feature` + re-exports
- `crates/pt-scan/examples/process_sample.rs` — added stage 6: clustering +
  feature candidate table with formatted output
- `crates/pt-scan/tests/integration.rs` — added 2 integration tests:
  - `test_feature_candidates_synthetic`: end-to-end with synthetic PLY
  - `test_powell_market_candidates`: real 294MB scan → 13 candidates validated

## Test Coverage

| Category | Tests | Status |
|----------|-------|--------|
| FeatureCandidate fields (known geometry) | 1 | Pass |
| Color classification (5 categories + unknown) | 5 | Pass |
| Vertical profile (4 types) | 4 | Pass |
| Empty input edge case | 1 | Pass |
| Synthetic integration (full pipeline) | 1 | Pass |
| Powell & Market real scan | 1 | Pass |
| **Total new tests** | **13** | **All pass** |

## Scenario Dashboard

- Before: 83.5 min / 240.0 min (34.8%)
- After: 83.5 min / 240.0 min (34.8%)
- No regression. This ticket is foundational infrastructure for BAML feature
  classification (future ticket), not a scenario-flipping deliverable.

## Quality Gate

- `just fmt` — clean
- `just lint` — clean (clippy strict, warnings=errors)
- `just test` — all 54 pt-scan tests pass (41 unit + 13 integration)
- `just scenarios` — no regression

## Acceptance Criteria Verification

- [x] `FeatureCandidate` struct with all 10 specified fields, serde Serialize
- [x] `extract_candidates(clusters, points, ground_plane)` function
- [x] Color classification: RGB histogram → dominant color name
- [x] Vertical profile: Z distribution shape analysis
- [x] Unit tests with synthetic clusters → expected candidates
- [x] CLI example extended with feature candidates table
- [x] Powell & Market produces 13 candidates (within 5-20 range)

## Design Decisions

1. **New `feature.rs` module** rather than extending cluster.rs — clean separation
   of spatial grouping (DBSCAN) from geometric summarization (feature extraction).

2. **f64 for FeatureCandidate fields** — matches BAML/JSON number precision
   requirements, even though internal computation uses f32 positions.

3. **Height = max ground-plane distance** — uses full plane equation (not raw Z)
   for correctness with tilted ground planes.

4. **Conical detection** — splits cluster into upper/lower halves by Z, compares
   XY spread. Taper ratio < 0.7 → conical. Works well for evergreen tree shapes.

## Open Concerns

1. **Color classification is coarse** — the five-bucket approach works for the
   "green/brown/gray" categories common in LiDAR scans, but unusual materials
   (e.g., painted surfaces) may consistently classify as "mixed". Acceptable per
   ticket: "Color names are intentionally coarse — the LLM interprets fine
   distinctions."

2. **Powell & Market test takes ~260s in debug mode** — processing 294MB PLY file.
   Runs fine in CI with release builds but slows local iteration. The test already
   existed for clustering; feature extraction adds negligible overhead.

3. **No BAML schema yet** — the FeatureCandidate struct is designed to match a
   future BAML class. A subsequent ticket should add the corresponding BAML
   schema with identical field names.

4. **Ground plane assumed roughly horizontal** — the conical taper detection
   splits by raw Z coordinate, not by distance-from-plane. For severely tilted
   scans this could misclassify. In practice, iPhone LiDAR scans are near-vertical
   so this is acceptable.
