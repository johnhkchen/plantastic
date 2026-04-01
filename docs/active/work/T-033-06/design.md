# T-033-06 Design: ML-Ready Abstractions

## Decision: Trait Abstractions in pt-scan + ScanPipeline Compositor

### Option A: Traits in pt-scan, pipeline in pt-scan (CHOSEN)

Put `PointFeatureExtractor`, `FeatureClusterer`, `FeatureVector`, and
`ScanPipeline` all in pt-scan. The classification trait already lives in
pt-features and stays there.

**Pros**: Single crate for the compute pipeline. No new crate. Downstream
consumers (scan_to_quote, pt-features) import from one place. The pipeline
struct composes all three stages.

**Cons**: pt-scan grows larger. But it's already the natural home — eigenvalue
and cluster code already live here.

### Option B: New pt-pipeline crate

**Rejected.** Would add a crate boundary for what's essentially composition of
things already in pt-scan. The only cross-crate dependency is FeatureClassifier
(in pt-features), which we reference via trait object — no compile-time dep
from pt-scan on pt-features.

### Option C: Traits in a shared pt-scan-traits crate

**Rejected.** Premature separation. If ML crates need to implement these traits
from outside the workspace, a traits crate makes sense then. Not now.

## FeatureVector Design

```rust
#[derive(Debug, Clone)]
pub struct FeatureVector {
    values: Vec<f32>,
}

impl FeatureVector {
    pub fn new(values: Vec<f32>) -> Self { Self { values } }
    pub fn dim(&self) -> usize { self.values.len() }
    pub fn as_slice(&self) -> &[f32] { &self.values }
}
```

Newtype over `Vec<f32>`. Dimensionality check at pipeline construction, not at
every vector — the extractor declares `feature_dim()` and the pipeline asserts
consistency once.

## PointFeatureExtractor Trait

```rust
pub trait PointFeatureExtractor: Send + Sync {
    fn extract(&self, points: &[Point], k: usize) -> Vec<FeatureVector>;
    fn feature_dim(&self) -> usize;
}
```

`EigenvalueExtractor` wraps the existing `compute_point_features()` and converts
`PointFeatures` to `FeatureVector` (6D: planarity, linearity, sphericity,
omnivariance, curvature, + a 6th from normal-z or just curvature). Actually,
the ticket says `eigenvalue=6` and the current code produces 6 scalar features
(planarity, linearity, sphericity, omnivariance, curvature) + normal[3] = 8
values total, but the HDBSCAN code only uses 3 (planarity, linearity,
sphericity). For the feature vector we'll use all 6 scalars: planarity,
linearity, sphericity, omnivariance, curvature, normal_z. This gives the
clusterer a richer signal while keeping dimensionality reasonable.

## FeatureClusterer Trait

```rust
pub trait FeatureClusterer: Send + Sync {
    fn cluster(
        &self,
        points: &[Point],
        features: &[FeatureVector],
        config: &ClusterConfig,
    ) -> ClusterResult;
}
```

`HdbscanClusterer` wraps `hdbscan_cluster()`. The existing `ClusterConfig` is
reused for DBSCAN params; HDBSCAN uses `HdbscanConfig`. The trait method takes
`ClusterConfig` for generality. `HdbscanClusterer` holds its own `HdbscanConfig`
and ignores the `ClusterConfig` argument (or we make the config generic — but
simpler to just carry the HDBSCAN config on the struct).

Actually, re-reading the ticket: the trait signature uses `&ClusterConfig`.
The simplest design: `HdbscanClusterer` stores `HdbscanConfig` internally and
the `ClusterConfig` parameter is the generic pipeline config. We can make it
work by having `HdbscanClusterer` ignore `config` or we can have a unified
config. The ticket shows `config: &ClusterConfig` so let's keep that — the
`HdbscanClusterer` carries its specific config and the pipeline-level config
is additional context.

## ScanPipeline

```rust
pub struct ScanPipeline {
    extractor: Box<dyn PointFeatureExtractor>,
    clusterer: Box<dyn FeatureClusterer>,
    classifier: Box<dyn FeatureClassifier>,
}
```

`FeatureClassifier` is the existing async trait from pt-features. But pt-scan
doesn't depend on pt-features at compile time. Solution: define a local
`FeatureClassifier` trait in pt-scan that mirrors the one in pt-features, or
accept `Box<dyn Any>` and downcast. Actually simpler: since the pipeline
orchestration that needs all three stages happens in the *example* (scan_to_quote)
or in the API layer, not in pt-scan itself, we should define ScanPipeline in
pt-scan with just extractor + clusterer, and let the API layer compose the
classifier. Or: we can put a sync `FeatureClassifier` trait in pt-scan.

**Decision**: The `ScanPipeline` in pt-scan holds extractor + clusterer. The
classifier is composed at the call site (scan_to_quote, API handler) since it's
async and lives in pt-features. This avoids pt-scan depending on async_trait
or pt-features. The ticket's acceptance criteria show `ScanPipeline` with all
three, but the classifier field can be `Box<dyn Any>` with a note that it's
plugged in by the caller. Actually, let's just make it generic or use a
separate trait. Simplest: add a thin `FeatureClassifier` re-export trait in
the pipeline module that is sync and optional. No — the ticket explicitly shows
the struct with `classifier: Box<dyn FeatureClassifier>`.

**Final decision**: Define a local sync `FeatureClassifier` trait in pt-scan's
pipeline module for the struct. The pt-features async trait stays separate.
The pipeline.run() method calls the sync version. For the async BAML classifier,
the caller wraps it. This keeps pt-scan free of async_trait deps.

Wait — looking at this more carefully, the ticket says "Tests use:
EigenvalueExtractor + HdbscanClusterer + MockClassifier". The mock classifier
is sync anyway. And the pipeline struct exists primarily for composition and
testing. Let's keep it simple: the classifier trait in the pipeline is **sync**
(blocking), and the BAML path uses `block_on()` as scan_to_quote already does.

## Classification Logging

New `logging.rs` module in pt-scan:
- `ClassificationLog` struct with JSONL append
- Records: timestamp, scan_id, candidates input, classifications output, context
- Enabled by env var `PLANTASTIC_LOG_CLASSIFICATIONS=1`
- Output dir: `data/classification_log/` (gitignored)
- Filename: `{scan_id}_{timestamp}.jsonl`

## Testing Strategy

- Unit tests for `FeatureVector` (dim, construction, empty)
- Unit tests for `EigenvalueExtractor` implements `PointFeatureExtractor`
- Unit tests for `HdbscanClusterer` implements `FeatureClusterer`
- Unit tests for `ScanPipeline` construction (dim mismatch panics)
- Integration test: pipeline with mock classifier end-to-end
- Unit tests for `ClassificationLog` (writes valid JSONL, respects env var)
- All existing tests must pass unchanged
