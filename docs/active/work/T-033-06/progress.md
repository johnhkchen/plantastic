# T-033-06 Progress: ML-Ready Abstractions

## Completed

### Step 1-2: FeatureVector + EigenvalueExtractor
- Created `crates/pt-scan/src/pipeline.rs` with `FeatureVector` newtype
- Implemented `PointFeatureExtractor` trait
- `EigenvalueExtractor` wraps `compute_point_features()`, produces 6D vectors:
  [planarity, linearity, sphericity, omnivariance, curvature, normal_z]
- Tests: dim check, correct output dimensionality

### Step 3: HdbscanClusterer
- Implemented `FeatureClusterer` trait
- `HdbscanClusterer` converts `FeatureVector` → `PointFeatures` for existing
  HDBSCAN code path
- Test: separates two spatial blobs (with spatial_weight=5.0 to dominate over
  feature variation within blobs)

### Step 4: ScanPipeline
- `PipelineClassifier` trait (sync) with `ClassifiedFeatureOutput` result type
- `ScanPipeline::new(extractor, clusterer, classifier)` composes all three
- `ScanPipeline::run()` orchestrates extract → cluster → candidates → classify
- `PipelineError` enum for error propagation
- Test: end-to-end with `MockPipelineClassifier`

### Step 5: Classification logging
- Created `crates/pt-scan/src/logging.rs`
- `ClassificationLogger` writes append-only JSONL to `data/classification_log/`
- `ClassificationRecord` struct: timestamp, scan_id, candidates, classifications, context
- Opt-in via `PLANTASTIC_LOG_CLASSIFICATIONS` env var
- Tests: disabled by default, writes valid JSONL, noop when disabled

### Step 6: Wiring
- Added `pub mod logging; pub mod pipeline;` to lib.rs
- Added all re-exports
- Added `data/classification_log/` to .gitignore
- No new Cargo.toml deps needed (used std::time instead of chrono)

### Step 7: Quality gate
- `just fmt` — clean
- `just lint` — clean (all clippy warnings fixed)
- All 76 lib tests pass (6 new)
- All workspace tests pass (excluding pre-existing powell_market SIGKILL from debug-mode memory)
- Scenario dashboard: 87.5 min / 240.0 min (36.5%) — no regression

## Deviations from Plan

1. **No chrono dependency.** Used `std::time::SystemTime` + manual epoch arithmetic
   for ISO 8601 timestamps. Avoids adding a new dependency for one function.

2. **`PipelineClassifier` is sync, not async.** Keeps pt-scan free of async_trait.
   The BAML classifier is wrapped with `block_on()` at the call site (as
   `scan_to_quote` already does).

3. **`ClassifiedFeatureOutput` local type.** Instead of depending on BAML-generated
   types in the pipeline's public API, defined a local mirror struct. Keeps pt-scan
   independent of BAML codegen.
