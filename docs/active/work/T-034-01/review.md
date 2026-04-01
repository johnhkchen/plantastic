# T-034-01 Review: BAML ClassifyFeatures

## Summary

Implemented the BAML `ClassifyFeatures` function and `pt-features` crate, providing LLM-powered classification of LiDAR feature candidates. Three classifier implementations follow the established pt-proposal pattern: `BamlFeatureClassifier` (production), `ClaudeCliClassifier` (subscription dev), and `MockFeatureClassifier` (CI/tests).

## Files Created

| File | Purpose |
|------|---------|
| `baml_src/classify.baml` | BAML schema: FeatureCandidateInput, ClassifiedFeature types, ClassifyFeatures function with SF Bay Area prompt |
| `crates/pt-features/Cargo.toml` | Crate manifest |
| `crates/pt-features/src/lib.rs` | Module declarations, baml_client include, re-exports |
| `crates/pt-features/src/classifier.rs` | FeatureClassifier trait + BamlFeatureClassifier |
| `crates/pt-features/src/mock.rs` | MockFeatureClassifier (geometry heuristics) + MockFailingClassifier |
| `crates/pt-features/src/claude_cli.rs` | ClaudeCliClassifier (claude CLI subprocess) |
| `crates/pt-features/src/error.rs` | ClassificationError enum |
| `crates/pt-features/tests/classify.rs` | 8 integration tests |

## Files Modified

| File | Change |
|------|--------|
| `baml_client/` | Regenerated (18 files) — adds ClassifyFeatures + types |
| `crates/pt-scan/tests/integration.rs` | Fixed pre-existing `cast_sign_loss` lint (line 795) |

## Test Coverage

### pt-features tests (8 tests, all pass)

| Test | What it verifies |
|------|------------------|
| `test_mock_classifies_tree_trunk` | 25ft columnar brown → tree, confidence > 0.7, species present |
| `test_mock_classifies_hardscape` | 0.5ft flat gray → hardscape, no species |
| `test_mock_classifies_utility_pole` | 33ft columnar narrow gray → utility, no species |
| `test_mock_classifies_mixed_candidates` | 3 different types → correct categories, cluster_ids preserved |
| `test_mock_powell_market_two_trees` | Two trunk clusters (25ft, 23ft, brown, columnar) → both tree, >0.7 confidence, species present |
| `test_mock_empty_input` | Empty input → empty output |
| `test_failing_classifier` | MockFailingClassifier → ClassificationError::Classification |
| `test_classified_feature_fields_valid` | Category in valid set, confidence 0.0–1.0, non-empty labels/reasoning |

### Powell & Market validation

The `test_mock_powell_market_two_trees` test validates the core AC: two trunk clusters with Powell & Market-like geometry (25.1 ft and 22.6 ft, columnar, brown, 1247 and 983 points) are both classified as trees with confidence > 0.7 and species identification. The mock classifies them as "Street Tree Trunk" (Lophostemon confertus) since spread < 5ft (trunk-only scan, no canopy). With a real LLM, the prompt's regional context ("Powell & Market Streets, San Francisco" + "USDA 10b / Sunset 17") would enable London Plane / Brisbane Box identification.

## Scenario Dashboard

Before: 83.5 min / 240.0 min (34.8%)
After: 83.5 min / 240.0 min (34.8%)

No regression. This ticket provides infrastructure (BAML classification pipeline) — it does not directly flip a scenario. It unblocks S.1.4 (plant identification) which requires pt-plants + this classification layer.

## Quality Gate

- `just fmt` — pass
- `just lint` — pass
- `cargo test -p pt-features` — 8/8 pass
- `just scenarios` — no regressions

## Open Concerns

1. **Real LLM validation not run**: The BamlFeatureClassifier and ClaudeCliClassifier compile and follow the proven pt-proposal pattern, but have not been tested with a real API key. First real run should be manual (dev-only) to capture output as a fixture.

2. **Powell & Market fixture data is approximated**: The centroid/bbox/height values in tests are plausible estimates, not from an actual scan run. When T-033-02's real scan processing is available, the fixture should be updated from actual output.

3. **No scenario milestone claimed**: This ticket delivers infrastructure for S.1.4 (plant identification) but doesn't independently unlock any scenario. A milestone could be added when the classification pipeline is wired into the scan upload flow.

4. **Async test compatibility**: Tests use `#[tokio::test]` directly (no `pt_test_utils::timed()` wrapper) because the mock classifier is async. These tests complete in <1ms so timeout enforcement isn't needed, but this is a pattern deviation from the rest of the codebase.
