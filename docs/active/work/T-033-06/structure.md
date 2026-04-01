# T-033-06 Structure: ML-Ready Abstractions

## New Files

### `crates/pt-scan/src/pipeline.rs`

Core module. Contains:

- `FeatureVector` — newtype over `Vec<f32>` with `new()`, `dim()`, `as_slice()`
- `PointFeatureExtractor` trait — `extract(&self, points, k) -> Vec<FeatureVector>` + `feature_dim(&self) -> usize`
- `FeatureClusterer` trait — `cluster(&self, points, features, config) -> ClusterResult`
- `PipelineClassifier` trait — sync classifier for pipeline composition, `classify(&self, candidates, address, climate_zone) -> Result<Vec<ClassifiedFeature>, ClassificationError>`
- `EigenvalueExtractor` — implements `PointFeatureExtractor`, wraps `compute_point_features()`
- `HdbscanClusterer` — implements `FeatureClusterer`, wraps `hdbscan_cluster()`
- `ScanPipeline` struct — composes `Box<dyn PointFeatureExtractor>`, `Box<dyn FeatureClusterer>`, `Box<dyn PipelineClassifier>`
- `ScanPipeline::new()` — validates `feature_dim()` consistency
- `ScanPipeline::default()` — `EigenvalueExtractor` + `HdbscanClusterer` + requires classifier
- `ScanPipeline::run()` — orchestrates extract → cluster → classify, returns structured result

### `crates/pt-scan/src/logging.rs`

Classification logging module. Contains:

- `ClassificationRecord` — serde struct: timestamp, scan_id, candidates, classifications, context
- `ClassificationLogger` — writes JSONL to `data/classification_log/`
- `ClassificationLogger::new(scan_id)` — creates logger, checks env var
- `ClassificationLogger::log()` — appends JSON record + newline
- `ClassificationContext` — struct: address, climate_zone
- Enabled by `PLANTASTIC_LOG_CLASSIFICATIONS` env var

## Modified Files

### `crates/pt-scan/src/lib.rs`

Add:
```rust
pub mod logging;
pub mod pipeline;

pub use pipeline::{
    EigenvalueExtractor, FeatureClusterer, FeatureVector,
    HdbscanClusterer, PipelineClassifier, PointFeatureExtractor, ScanPipeline,
};
pub use logging::ClassificationLogger;
```

### `crates/pt-scan/Cargo.toml`

Add dependency: `chrono = "0.4"` for timestamp generation in logging.
Add `serde_json` is already present.

### `.gitignore`

Add: `data/classification_log/`

## Module Boundaries

```
pt-scan/
├── pipeline.rs       # Traits + impls + ScanPipeline compositor
├── logging.rs        # JSONL classification logger
├── eigenvalue.rs     # Unchanged — compute_point_features() stays
├── cluster.rs        # Unchanged — hdbscan_cluster() stays
├── feature.rs        # Unchanged — extract_candidates() stays
└── lib.rs            # New re-exports

pt-features/
├── classifier.rs     # Unchanged — async FeatureClassifier stays
├── mock.rs           # Unchanged — MockFeatureClassifier stays
└── ...
```

The existing free functions remain. The trait impls delegate to them.
No breaking changes — all existing call sites continue working.

## Public Interface Changes

New public types from pt-scan:
- `FeatureVector`
- `PointFeatureExtractor` (trait)
- `FeatureClusterer` (trait)
- `PipelineClassifier` (trait)
- `EigenvalueExtractor` (struct)
- `HdbscanClusterer` (struct)
- `ScanPipeline` (struct)
- `ClassificationLogger` (struct)
- `ClassificationRecord` (struct)

No removals. No signature changes to existing functions.

## Type Relationships

```
PointFeatureExtractor::extract() → Vec<FeatureVector>
    ↓
FeatureClusterer::cluster(points, features) → ClusterResult
    ↓
extract_candidates(clusters, points, plane) → Vec<FeatureCandidate>
    ↓
PipelineClassifier::classify(candidates, addr, zone)
    → Vec<ClassifiedFeature>
    ↓
ClassificationLogger::log(candidates, classified, context)
    → data/classification_log/*.jsonl
```

The `ClassifiedFeature` type is from `baml_client::types`. pt-scan already
has a `#[path]` include of baml_client for the `annotate` module's
`ClassifiedFeatureRef` trait. We'll use the same pattern for the pipeline
classifier output, or define a simpler return type. Actually, looking at
`annotate.rs`, it defines its own `ClassifiedFeatureRef` trait. For the
pipeline classifier we can return a generic struct or use the BAML type.

Decision: `PipelineClassifier` returns `Vec<ClassifiedFeatureOutput>` — a
local struct mirroring `ClassifiedFeature` fields. This keeps pt-scan
independent of BAML-generated types in its public API. The BAML type can
impl `From<ClassifiedFeatureOutput>` at the pt-features level.

Actually, simplest path: just use a `serde_json::Value` for the classification
output in the logging, and define `ClassifiedFeatureOutput` as a simple struct
in pipeline.rs with the same fields. The MockClassifier in pt-scan tests
returns this type. For real usage, pt-features wraps its async classifier
in a sync adapter that converts BAML `ClassifiedFeature` → `ClassifiedFeatureOutput`.
