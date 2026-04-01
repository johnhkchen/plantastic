# T-034-01 Plan: BAML ClassifyFeatures

## Step 1: BAML Schema

Create `baml_src/classify.baml` with:
- `FeatureCandidateInput` class (mirrors pt-scan FeatureCandidate)
- `ClassifiedFeature` class (output schema)
- `ClassifyFeatures` function with prompt tuned for SF Bay Area
- Test case with Powell & Market example data

Verification: file parses correctly, no BAML syntax errors.

## Step 2: Regenerate BAML Client

Run `baml-cli generate` from project root.

Verification: `baml_client/` updated with new types and function. Confirm `ClassifyFeatures` struct and `ClassifiedFeature`/`FeatureCandidateInput` types in generated code.

## Step 3: Create pt-features Crate Skeleton

Create `crates/pt-features/Cargo.toml` and `src/` directory with:
- `error.rs` — ClassificationError enum
- `lib.rs` — module declarations, baml_client include, re-exports

Verification: `cargo check -p pt-features` compiles.

## Step 4: Implement MockFeatureClassifier

Create `src/mock.rs`:
- Heuristic classification from geometry: tall+columnar → tree, short+flat → hardscape, etc.
- Deterministic, no randomness
- Species inference from height/spread ranges for trees

Verification: unit tests in mock.rs pass.

## Step 5: Implement FeatureClassifier Trait + BamlFeatureClassifier

Create `src/classifier.rs`:
- `FeatureClassifier` trait with `classify()` method
- `to_baml_input()` conversion function
- `BamlFeatureClassifier` calling generated BAML client

Verification: compiles, trait is object-safe.

## Step 6: Implement ClaudeCliClassifier

Create `src/claude_cli.rs`:
- Build JSON prompt with candidate summaries
- Call `claude` CLI subprocess
- Parse response with BAML

Verification: compiles (runtime test requires claude CLI).

## Step 7: Create Powell & Market Test Fixture

Write a small binary/script that:
1. Loads the Powell & Market scan
2. Runs scan processing + clustering + feature extraction
3. Serializes candidates to JSON
4. Saves to `tests/fixtures/powell_market_candidates.json`

Or: generate the fixture inline during the first test run and commit it.

Verification: fixture file exists with realistic candidate data.

## Step 8: Integration Tests

Create `crates/pt-features/tests/classify.rs`:
1. **test_mock_classifies_synthetic_candidates**: Create hand-crafted FeatureCandidates, run MockFeatureClassifier, verify output structure
2. **test_mock_tall_columnar_is_tree**: Verify tall columnar brown candidate → tree category
3. **test_mock_short_flat_is_hardscape**: Verify short flat gray candidate → hardscape
4. **test_mock_classifies_powell_market**: Load fixture, classify with mock, verify two largest candidates are trees with confidence > 0.7
5. **test_classification_error_handling**: MockFailingClassifier returns error

Verification: `cargo test -p pt-features` passes.

## Step 9: Quality Gate

Run `just check`:
- `just fmt` — format new code
- `just lint` — clippy strict on pt-features
- `just test` — all workspace tests pass
- `just scenarios` — no regressions

Verification: all four pass.

## Step 10: Capture Real Classification Fixture (optional, dev-only)

If ANTHROPIC_API_KEY available, run real BAML classification on Powell & Market candidates, save output as `tests/fixtures/powell_market_classified.json` for future regression testing.

This step is manual/dev-only, not part of CI.
