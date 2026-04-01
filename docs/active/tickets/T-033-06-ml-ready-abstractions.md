---
id: T-033-06
story: S-033
title: ml-ready-abstractions
type: task
status: open
priority: medium
phase: ready
depends_on: [T-033-04]
---

## Context

The segmentation pipeline should be extensible for future ML research without requiring ML crates today. A 2026 lab working on this project would want to swap feature extraction, clustering, and classification without rewriting the pipeline. They'd also want structured training data from every BAML call.

This ticket sets up the trait abstractions and data logging that make ML experimentation plug-and-play.

## Acceptance Criteria

### Trait abstractions

- `PointFeatureExtractor` trait:
  ```rust
  trait PointFeatureExtractor: Send + Sync {
      fn extract(&self, points: &[Point3], k: usize) -> Vec<FeatureVector>;
      fn feature_dim(&self) -> usize;  // eigenvalue=6, learned=N
  }
  ```
- `FeatureClusterer` trait:
  ```rust
  trait FeatureClusterer: Send + Sync {
      fn cluster(&self, points: &[Point3], features: &[FeatureVector], config: &ClusterConfig) -> ClusterResult;
  }
  ```
- `EigenvalueExtractor` implements `PointFeatureExtractor` (current code, refactored)
- `HdbscanClusterer` implements `FeatureClusterer` (current code, refactored)
- `FeatureVector` is a newtype over `Vec<f32>` with dimensionality check

### Structured classification logging

- Every BAML/ClaudeCli classification call logs a JSON record:
  ```json
  {
    "timestamp": "...",
    "scan_id": "powell-market-downsampled",
    "candidates": [...],        // FeatureCandidate input
    "classifications": [...],   // ClassifiedFeature output
    "context": { "address": "...", "climate_zone": "..." }
  }
  ```
- Logged to `data/classification_log/` (gitignored)
- Each record is a potential training example for distillation
- Log format is append-only JSONL (one JSON object per line)

### Pipeline composition

- `ScanPipeline` struct composes extractor + clusterer + classifier:
  ```rust
  struct ScanPipeline {
      extractor: Box<dyn PointFeatureExtractor>,
      clusterer: Box<dyn FeatureClusterer>,
      classifier: Box<dyn FeatureClassifier>,
  }
  ```
- Default pipeline: `EigenvalueExtractor` + `HdbscanClusterer` + `BamlClassifier`
- Tests use: `EigenvalueExtractor` + `HdbscanClusterer` + `MockClassifier`
- Future ML: `LearnedExtractor` + `HdbscanClusterer` + `DistilledClassifier`

## Implementation Notes

- Don't pull in burn/candle/dfdx — this ticket is about interfaces, not implementations
- The `FeatureVector` dimensionality must be consistent within a pipeline run (extractor.feature_dim() checked at pipeline construction)
- Classification logging should be opt-in (feature flag or env var) to avoid disk writes in production
- The log format is designed for: `cat data/classification_log/*.jsonl | python train.py`
- A future ticket can add `burn` as an optional dep behind a feature flag for the learned extractor
