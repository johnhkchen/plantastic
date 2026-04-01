# T-033-06 Review: ML-Ready Abstractions

## Summary

Added trait abstractions and structured logging to make the scan segmentation
pipeline extensible for ML experimentation. Future implementations (learned
feature extractors, distilled classifiers) can be swapped in without rewriting
the pipeline.

## Files Changed

| File | Change |
|------|--------|
| `crates/pt-scan/src/pipeline.rs` | **New.** `FeatureVector`, `PointFeatureExtractor`, `FeatureClusterer`, `PipelineClassifier` traits. `EigenvalueExtractor`, `HdbscanClusterer`, `ScanPipeline` impls. 6 tests. |
| `crates/pt-scan/src/logging.rs` | **New.** `ClassificationLogger` for JSONL training data. `ClassificationRecord`, `ClassificationContext`. 3 tests. |
| `crates/pt-scan/src/lib.rs` | Added `pub mod pipeline; pub mod logging;` and re-exports. |
| `.gitignore` | Added `data/classification_log/`. |

## What Was Delivered

### Trait Abstractions

- **`PointFeatureExtractor`**: `extract(points, k) → Vec<FeatureVector>` + `feature_dim() → usize`
- **`FeatureClusterer`**: `cluster(points, features, config) → ClusterResult`
- **`PipelineClassifier`**: `classify(candidates, address, climate_zone) → Result<Vec<ClassifiedFeatureOutput>>`
- **`FeatureVector`**: Newtype over `Vec<f32>` with dimensionality tracking
- **`EigenvalueExtractor`**: Wraps existing `compute_point_features()`, 6D output
- **`HdbscanClusterer`**: Wraps existing `hdbscan_cluster()`, stores `HdbscanConfig`

### Pipeline Composition

- **`ScanPipeline`**: Composes extractor + clusterer + classifier via trait objects
- **`ScanPipeline::run()`**: Orchestrates the full segmentation flow
- Tests use `MockPipelineClassifier` — no LLM calls

### Structured Classification Logging

- **`ClassificationLogger`**: Append-only JSONL to `data/classification_log/`
- Each record contains candidates, classifications, and context (address, climate zone)
- Opt-in via `PLANTASTIC_LOG_CLASSIFICATIONS=1` env var
- Format designed for `cat *.jsonl | python train.py`

## Test Coverage

| Area | Tests | Notes |
|------|-------|-------|
| `FeatureVector` | 2 | dim, empty |
| `EigenvalueExtractor` | 2 | feature_dim, output dim from flat grid |
| `HdbscanClusterer` | 1 | separates two spatial blobs |
| `ScanPipeline` | 1 | end-to-end with mock classifier |
| `ClassificationLogger` | 3 | disabled by default, writes valid JSONL, noop |
| **Total new** | **9** | All existing 67 lib tests still pass |

## Scenario Dashboard

- **Before**: 87.5 min / 240.0 min (36.5%)
- **After**: 87.5 min / 240.0 min (36.5%)
- **No regression.** This ticket is infrastructure (trait abstractions + logging),
  not a new customer-facing capability.

## Open Concerns

1. **Pre-existing SIGKILL on powell_market integration tests.** The real PLY
   processing tests (test_powell_market_*) are killed by the 120s test timeout
   in debug mode. Not introduced by this ticket — they process the full 11MB PLY
   file which is too slow without `--release`. These tests pass in release mode.

2. **FeatureVector dimensionality is not enforced at the pipeline level.** The
   `ScanPipeline` stores the extractor's `feature_dim()` but doesn't validate
   that the clusterer can handle that dimensionality. This is by design — the
   clusterer adapts to whatever features it receives. If a future extractor
   produces wildly different dims, the clusterer should handle it gracefully
   (or the `HdbscanClusterer` could be enhanced with dim-awareness).

3. **`ClassifiedFeatureOutput` mirrors BAML `ClassifiedFeature`.** The local
   type keeps pt-scan independent of BAML codegen, but means conversion is
   needed at the integration point. This is a deliberate tradeoff — coupling
   pt-scan to BAML-generated types would create a fragile dependency.

4. **Logging timestamp uses manual epoch arithmetic.** Avoids a chrono dependency
   for one function. If chrono is added to the workspace later, this could be
   simplified.

## Not Addressed (Future Tickets)

- Actual ML implementations (LearnedExtractor, DistilledClassifier) — needs burn/candle
- Integration of logging into scan_to_quote or API handler — caller's responsibility
- Milestone claim — no existing milestone maps to this work; a new one could be
  added if this ticket is considered foundational enough
