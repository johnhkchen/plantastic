# T-034-01 Progress: BAML ClassifyFeatures

## Completed

### Step 1: BAML Schema ✓
- Created `baml_src/classify.baml` with:
  - `FeatureCandidateInput` class (10 fields, flattened centroid to x/y/z for BAML compat)
  - `ClassifiedFeature` class (7 fields including optional species)
  - `ClassifyFeatures` function with SF Bay Area-tuned prompt
  - `powell_market_features` test case with two trunk cluster examples
- Client: reuses `ProposalFallback` (Haiku → Sonnet fallback)

### Step 2: Regenerate BAML Client ✓
- Ran `baml-cli generate` — 18 files written to `baml_client/`
- Verified: `ClassifiedFeature`, `FeatureCandidateInput`, `ClassifyFeatures` all generated correctly
- Generated types use `i64` for int, `f64` for float, `Option<String>` for nullable string

### Step 3: pt-features Crate ✓
- Created `crates/pt-features/` with Cargo.toml + 5 source files
- Dependencies: async-trait, baml, serde, serde_json, thiserror, tokio, pt-scan
- `baml_client` included via `#[path]` attribute (same pattern as pt-proposal)

### Step 4: MockFeatureClassifier ✓
- Heuristic classification in `mock.rs`:
  - Tall + columnar + narrow + gray → utility pole (0.75 confidence)
  - Tall + columnar/spreading + brown/green → tree (0.60–0.85 confidence)
  - Short + flat → hardscape (0.65 confidence)
  - Fallback → generic structure (0.45 confidence)
- MockFailingClassifier for error path testing

### Step 5: FeatureClassifier Trait + BamlFeatureClassifier ✓
- `classifier.rs`: trait with `classify()` method, `to_baml_input()` conversion
- `BamlFeatureClassifier` calls `crate::B.ClassifyFeatures.call()`

### Step 6: ClaudeCliClassifier ✓
- `claude_cli.rs`: builds JSON prompt, calls `claude` CLI subprocess
- Parses response with `crate::B.ClassifyFeatures.parse()`
- Same pattern as pt-proposal's `ClaudeCliGenerator`

### Step 7: Test Fixture (simplified) ✓
- Used inline FeatureCandidate construction in tests instead of separate JSON fixture
- Powell & Market candidates hardcoded from expected real-scan values

### Step 8: Integration Tests ✓
- 8 tests in `crates/pt-features/tests/classify.rs`:
  1. `test_mock_classifies_tree_trunk` — tree with species, high confidence
  2. `test_mock_classifies_hardscape` — curb, no species
  3. `test_mock_classifies_utility_pole` — utility, no species
  4. `test_mock_classifies_mixed_candidates` — 3 types, correct categories
  5. `test_mock_powell_market_two_trees` — 2 trunk clusters → both tree, >0.7 confidence
  6. `test_mock_empty_input` — empty vec → empty vec
  7. `test_failing_classifier` — error propagation
  8. `test_classified_feature_fields_valid` — field constraints (category valid, confidence 0-1)
- All 8 pass

### Step 9: Quality Gate ✓
- `just fmt` — clean
- `just lint` — clean (also fixed pre-existing cast_sign_loss lint in pt-scan integration tests)
- `cargo test -p pt-features` — 8/8 pass
- `just scenarios` — 83.5 min / 240.0 min (34.8%), no regressions

## Deviations from Plan

1. **No separate JSON fixture file**: Used inline candidate construction in tests. Simpler, no fixture loading code, same coverage.
2. **Flattened centroid in BAML**: Instead of `float[]` array, used `centroid_x/y/z` individual fields — more reliable with BAML's type system.
3. **Fixed pre-existing lint**: Added `clippy::cast_sign_loss` allow to `make_synthetic_ply_wide()` in pt-scan integration tests (line 795). Same fix already present in the first `make_synthetic_ply()` function (line 30).
