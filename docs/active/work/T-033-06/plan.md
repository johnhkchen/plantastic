# T-033-06 Plan: ML-Ready Abstractions

## Step 1: FeatureVector + Traits (pipeline.rs skeleton)

Create `crates/pt-scan/src/pipeline.rs` with:
- `FeatureVector` newtype
- `PointFeatureExtractor` trait
- `FeatureClusterer` trait
- `ClassifiedFeatureOutput` struct (local classification result type)
- `PipelineClassifier` trait (sync)

Unit tests: FeatureVector construction, dim, empty vector.

## Step 2: EigenvalueExtractor impl

Implement `PointFeatureExtractor` for `EigenvalueExtractor`:
- `feature_dim()` returns 6
- `extract()` calls `compute_point_features()`, converts `PointFeatures` → `FeatureVector`
  using [planarity, linearity, sphericity, omnivariance, curvature, normal_z]

Tests: extract from flat grid produces 6D vectors, dim matches.

## Step 3: HdbscanClusterer impl

Implement `FeatureClusterer` for `HdbscanClusterer`:
- Stores `HdbscanConfig`
- `cluster()` converts `&[FeatureVector]` back to `Vec<PointFeatures>` for
  the existing `hdbscan_cluster()` fn, or adapts `build_feature_vectors`
  to accept raw feature slices.

Actually, re-examining: `hdbscan_cluster()` takes `&[PointFeatures]`. The
trait takes `&[FeatureVector]`. The `HdbscanClusterer` needs to either:
(a) Build the HDBSCAN augmented vectors directly from `FeatureVector` slices
(b) Convert back to `PointFeatures`

Option (a) is cleaner — extract the `build_feature_vectors` logic to work
with raw `f32` slices. The HdbscanClusterer picks indices [0..3] as
planarity/linearity/sphericity from the FeatureVector, plus spatial coords.

Tests: two separated clusters with uniform features, same as existing.

## Step 4: ScanPipeline struct

- `ScanPipeline::new(extractor, clusterer, classifier)` — validates feature_dim consistency (logs warning, no panic — dim is checked against what the clusterer expects, but since the clusterer is generic, just store dim for metadata)
- `ScanPipeline::run(cloud, ground_plane, address, climate_zone)` → `PipelineResult`
- `PipelineResult`: candidates, classifications, gaps, timing

Tests: pipeline with mock classifier, end-to-end on synthetic data.

## Step 5: Classification logging (logging.rs)

- `ClassificationRecord` struct (timestamp, scan_id, candidates, classifications, context)
- `ClassificationLogger::new(scan_id)` — checks `PLANTASTIC_LOG_CLASSIFICATIONS` env var
- `ClassificationLogger::log(record)` — appends JSONL line
- `ClassificationContext` struct

Tests: writes valid JSONL, respects env var off, handles missing directory.

## Step 6: Wire into lib.rs + Cargo.toml

- Add `pub mod pipeline; pub mod logging;` to lib.rs
- Add re-exports
- Add `chrono` dependency to Cargo.toml
- Add `data/classification_log/` to .gitignore

## Step 7: MockClassifier for pt-scan tests

Add `MockPipelineClassifier` in pipeline.rs tests that returns deterministic
`ClassifiedFeatureOutput` values based on geometry (simplified version of
pt-features MockFeatureClassifier). This lets pt-scan test the full pipeline
without depending on pt-features.

## Step 8: Run quality gate

- `just fmt`
- `just lint`
- `just test`
- `just scenarios`
- Verify no regressions

## Verification Criteria

- [ ] `FeatureVector::new(vec![1.0, 2.0]).dim() == 2`
- [ ] `EigenvalueExtractor.feature_dim() == 6`
- [ ] `EigenvalueExtractor.extract()` returns vectors of dim 6
- [ ] `HdbscanClusterer` separates two spatial blobs
- [ ] `ScanPipeline::new()` composes all three components
- [ ] `ScanPipeline::run()` produces candidates + classifications
- [ ] Classification logger writes valid JSONL when enabled
- [ ] Classification logger does nothing when env var unset
- [ ] All existing tests pass
- [ ] `just check` passes
- [ ] Scenario dashboard same or higher
