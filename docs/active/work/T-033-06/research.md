# T-033-06 Research: ML-Ready Abstractions

## Current Pipeline Architecture

The scan segmentation pipeline lives in `crates/pt-scan/` and flows through
free functions — no trait abstractions at the scan layer:

```
process_scan() → PointCloud
  eigenvalue::compute_point_features(points, k) → Vec<PointFeatures>
  cluster::hdbscan_cluster(points, features, config) → ClusterResult
  feature::extract_candidates(clusters, points, plane) → Vec<FeatureCandidate>
```

Classification lives in a separate crate `crates/pt-features/` with an existing
async trait `FeatureClassifier` and three impls (BAML, ClaudeCliClassifier, Mock).

## Feature Extraction (eigenvalue.rs)

`compute_point_features(points: &[Point], k: usize) -> Vec<PointFeatures>`

- Builds a kiddo k-d tree, queries k-NN per point, computes 3x3 covariance,
  eigendecomposition via nalgebra. Returns 6D per-point features:
  planarity, linearity, sphericity, omnivariance, normal[3], curvature.
- Currently a standalone free function. No trait abstraction.
- The 6D feature vector is implicit — `PointFeatures` is a struct with named
  fields, not a generic vector. The HDBSCAN code cherry-picks 3 of 6 fields
  (planarity, linearity, sphericity) to build its augmented 6D vector.

## Clustering (cluster.rs)

Two clustering modes exist:
1. `cluster_obstacles(points, config)` — DBSCAN with fixed epsilon
2. `hdbscan_cluster(points, features, config)` — HDBSCAN in augmented feature space

`hdbscan_cluster` builds `Vec<Vec<f64>>` feature vectors internally via
`build_feature_vectors()`, combining normalized spatial coords with
planarity/linearity/sphericity. Output: `ClusterResult { clusters, noise_indices }`.

Key: `hdbscan_cluster` already accepts `&[PointFeatures]` alongside `&[Point]`,
so it's halfway to the trait-based design. But it hardcodes which features to use.

## Classification (pt-features crate)

`FeatureClassifier` trait already exists in `crates/pt-features/src/classifier.rs`:
```rust
#[async_trait]
pub trait FeatureClassifier: Send + Sync {
    async fn classify(&self, candidates, address, climate_zone)
        -> Result<Vec<ClassifiedFeature>, ClassificationError>;
}
```

Three impls: `BamlFeatureClassifier`, `ClaudeCliClassifier`, `MockFeatureClassifier`.

The `ClassifiedFeature` type is BAML-generated in `baml_client/types/classes.rs`.

## Pipeline Composition (scan_to_quote.rs example)

The `scan_to_quote` example manually wires:
1. `process_scan_timed()` → PointCloud
2. `cluster_obstacles()` → ClusterResult (uses DBSCAN, not HDBSCAN)
3. `extract_candidates()` → Vec<FeatureCandidate>
4. `classifier.classify()` → Vec<ClassifiedFeature>
5. `measure_gaps()` → Vec<Gap>
6. Zone/material/quote computation

No ScanPipeline struct — it's ad-hoc composition in the example binary.

## Data Logging

No classification logging exists today. BAML calls are fire-and-forget.
The `.gitignore` doesn't mention `data/` yet — needs adding.

## Key Constraints

- `FeatureVector` must be dimensionally consistent within a pipeline run.
  The ticket specifies `extractor.feature_dim()` checked at construction.
- `PointFeatureExtractor` is sync (compute-heavy, no I/O).
- `FeatureClusterer` is sync (same reason).
- `FeatureClassifier` is already async (LLM calls).
- Classification logging is opt-in (env var or feature flag).
- No ML crate deps (burn/candle/dfdx) — interfaces only.

## Files to Modify

| File | Change |
|------|--------|
| `crates/pt-scan/src/lib.rs` | Add pipeline module, re-exports |
| `crates/pt-scan/src/eigenvalue.rs` | Extract trait impl |
| `crates/pt-scan/src/cluster.rs` | Extract trait impl |
| `crates/pt-scan/src/feature.rs` | Add FeatureVector newtype |
| `crates/pt-scan/src/pipeline.rs` | New: ScanPipeline struct |
| `crates/pt-scan/src/logging.rs` | New: classification JSONL logger |
| `crates/pt-features/src/classifier.rs` | FeatureClassifier already exists |
| `.gitignore` | Add data/classification_log/ |

## Existing Tests

- eigenvalue.rs: 6 unit tests (flat, linear, spherical, degenerate, normal, perf)
- cluster.rs: 12 tests (DBSCAN + HDBSCAN variants)
- feature.rs: 9 tests (candidates, colors, profiles)
- integration.rs: full pipeline tests
- pt-features mock.rs: deterministic classifier tests

All existing tests must continue passing unchanged.
