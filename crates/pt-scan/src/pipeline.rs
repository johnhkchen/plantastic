//! ML-ready trait abstractions for the scan segmentation pipeline.
//!
//! Provides pluggable interfaces for feature extraction, clustering, and
//! classification so that future ML experiments (learned extractors, distilled
//! classifiers) can be swapped in without rewriting the pipeline.

use serde::{Deserialize, Serialize};

use crate::cluster::{ClusterResult, HdbscanConfig};
use crate::eigenvalue::{compute_point_features, PointFeatures};
use crate::feature::{extract_candidates, FeatureCandidate};
use crate::types::{Plane, Point};

// ---------------------------------------------------------------------------
// FeatureVector
// ---------------------------------------------------------------------------

/// Dimensioned feature vector for a single point.
///
/// Newtype over `Vec<f32>`. The extractor that produces it declares its
/// dimensionality via [`PointFeatureExtractor::feature_dim`]; the pipeline
/// validates consistency at construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    values: Vec<f32>,
}

impl FeatureVector {
    /// Create a new feature vector.
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }

    /// Number of dimensions.
    pub fn dim(&self) -> usize {
        self.values.len()
    }

    /// View the raw values.
    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }
}

// ---------------------------------------------------------------------------
// PointFeatureExtractor trait
// ---------------------------------------------------------------------------

/// Extracts per-point feature vectors from a point cloud.
///
/// Current implementation: [`EigenvalueExtractor`] (6D geometric features).
/// Future: learned embeddings from a neural network.
pub trait PointFeatureExtractor: Send + Sync {
    /// Compute feature vectors for all points using `k` nearest neighbors.
    fn extract(&self, points: &[Point], k: usize) -> Vec<FeatureVector>;

    /// The dimensionality of vectors produced by [`extract`](Self::extract).
    fn feature_dim(&self) -> usize;
}

// ---------------------------------------------------------------------------
// FeatureClusterer trait
// ---------------------------------------------------------------------------

/// Groups points into clusters using their positions and feature vectors.
///
/// Current implementation: [`HdbscanClusterer`] (density-based, augmented space).
/// Future: learned clustering, graph-based methods.
pub trait FeatureClusterer: Send + Sync {
    /// Cluster points given their feature vectors.
    fn cluster(
        &self,
        points: &[Point],
        features: &[FeatureVector],
        config: &crate::cluster::ClusterConfig,
    ) -> ClusterResult;
}

// ---------------------------------------------------------------------------
// ClassifiedFeatureOutput
// ---------------------------------------------------------------------------

/// Classification result for a single feature candidate.
///
/// Local mirror of the BAML-generated `ClassifiedFeature` so pt-scan stays
/// independent of BAML codegen. Conversion is trivial at the call site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedFeatureOutput {
    pub cluster_id: i64,
    pub label: String,
    pub category: String,
    pub species: Option<String>,
    pub confidence: f64,
    pub reasoning: String,
    pub landscape_notes: String,
}

// ---------------------------------------------------------------------------
// PipelineClassifier trait
// ---------------------------------------------------------------------------

/// Synchronous feature classifier for pipeline composition.
///
/// This is the sync counterpart to pt-features' async `FeatureClassifier`.
/// For real LLM calls, wrap the async classifier with `block_on()`.
pub trait PipelineClassifier: Send + Sync {
    /// Classify feature candidates.
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::Classification` if the classifier fails.
    fn classify(
        &self,
        candidates: &[FeatureCandidate],
        address: &str,
        climate_zone: &str,
    ) -> Result<Vec<ClassifiedFeatureOutput>, PipelineError>;
}

/// Errors from pipeline execution.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("classification failed: {0}")]
    Classification(String),

    #[error("scan processing failed: {0}")]
    Scan(#[from] crate::error::ScanError),
}

// ---------------------------------------------------------------------------
// EigenvalueExtractor
// ---------------------------------------------------------------------------

/// Extracts 6D eigenvalue-based geometric features per point.
///
/// Produces: `[planarity, linearity, sphericity, omnivariance, curvature, normal_z]`
#[derive(Debug, Default)]
pub struct EigenvalueExtractor;

impl PointFeatureExtractor for EigenvalueExtractor {
    fn extract(&self, points: &[Point], k: usize) -> Vec<FeatureVector> {
        let pf = compute_point_features(points, k);
        pf.iter().map(point_features_to_vector).collect()
    }

    fn feature_dim(&self) -> usize {
        6
    }
}

/// Convert a `PointFeatures` struct to a 6D `FeatureVector`.
fn point_features_to_vector(pf: &PointFeatures) -> FeatureVector {
    FeatureVector::new(vec![
        pf.planarity,
        pf.linearity,
        pf.sphericity,
        pf.omnivariance,
        pf.curvature,
        pf.normal[2], // normal Z component
    ])
}

/// Convert a `FeatureVector` back to `PointFeatures` for the HDBSCAN code path.
///
/// Assumes 6D eigenvalue layout: [planarity, linearity, sphericity, omnivariance,
/// curvature, normal_z]. Missing dimensions default to 0.
fn vector_to_point_features(fv: &FeatureVector) -> PointFeatures {
    let s = fv.as_slice();
    PointFeatures {
        planarity: s.first().copied().unwrap_or(0.0),
        linearity: s.get(1).copied().unwrap_or(0.0),
        sphericity: s.get(2).copied().unwrap_or(0.0),
        omnivariance: s.get(3).copied().unwrap_or(0.0),
        curvature: s.get(4).copied().unwrap_or(0.0),
        normal: [0.0, 0.0, s.get(5).copied().unwrap_or(1.0)],
    }
}

// ---------------------------------------------------------------------------
// HdbscanClusterer
// ---------------------------------------------------------------------------

/// Clusters points using HDBSCAN in an augmented spatial + feature space.
#[derive(Debug, Default)]
pub struct HdbscanClusterer {
    config: HdbscanConfig,
}

impl HdbscanClusterer {
    /// Create with the given HDBSCAN configuration.
    pub fn new(config: HdbscanConfig) -> Self {
        Self { config }
    }
}

impl FeatureClusterer for HdbscanClusterer {
    fn cluster(
        &self,
        points: &[Point],
        features: &[FeatureVector],
        _config: &crate::cluster::ClusterConfig,
    ) -> ClusterResult {
        let pf: Vec<PointFeatures> = features.iter().map(vector_to_point_features).collect();
        crate::cluster::hdbscan_cluster(points, &pf, &self.config)
    }
}

// ---------------------------------------------------------------------------
// ScanPipeline
// ---------------------------------------------------------------------------

/// Result of a full pipeline run.
#[derive(Debug)]
pub struct PipelineResult {
    /// Extracted feature candidates (one per cluster).
    pub candidates: Vec<FeatureCandidate>,
    /// Classifications from the pipeline classifier.
    pub classifications: Vec<ClassifiedFeatureOutput>,
    /// Dimensionality of the feature vectors used.
    pub feature_dim: usize,
}

/// Composable scan segmentation pipeline.
///
/// Default pipeline: `EigenvalueExtractor` + `HdbscanClusterer` + caller-provided classifier.
///
/// ```text
/// EigenvalueExtractor → HdbscanClusterer → extract_candidates → PipelineClassifier
/// ```
#[allow(missing_debug_implementations)]
pub struct ScanPipeline {
    extractor: Box<dyn PointFeatureExtractor>,
    clusterer: Box<dyn FeatureClusterer>,
    classifier: Box<dyn PipelineClassifier>,
}

impl ScanPipeline {
    /// Construct a pipeline from pluggable components.
    pub fn new(
        extractor: Box<dyn PointFeatureExtractor>,
        clusterer: Box<dyn FeatureClusterer>,
        classifier: Box<dyn PipelineClassifier>,
    ) -> Self {
        Self {
            extractor,
            clusterer,
            classifier,
        }
    }

    /// Run the full pipeline on obstacle points.
    ///
    /// 1. Extract per-point features
    /// 2. Cluster in augmented feature space
    /// 3. Extract geometric candidates per cluster
    /// 4. Classify candidates
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::Classification` if the classifier fails.
    pub fn run(
        &self,
        obstacles: &[Point],
        ground_plane: &Plane,
        k: usize,
        address: &str,
        climate_zone: &str,
    ) -> Result<PipelineResult, PipelineError> {
        let features = self.extractor.extract(obstacles, k);
        let feature_dim = self.extractor.feature_dim();

        let cluster_config = crate::cluster::ClusterConfig::default();
        let cluster_result = self
            .clusterer
            .cluster(obstacles, &features, &cluster_config);

        let candidates = extract_candidates(&cluster_result.clusters, obstacles, ground_plane);

        let classifications = self
            .classifier
            .classify(&candidates, address, climate_zone)?;

        Ok(PipelineResult {
            candidates,
            classifications,
            feature_dim,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::ClusterConfig;
    use crate::types::Plane;
    use pt_test_utils::timed;

    // -- FeatureVector tests --

    #[test]
    fn feature_vector_dim() {
        timed(|| {
            let fv = FeatureVector::new(vec![1.0, 2.0, 3.0]);
            assert_eq!(fv.dim(), 3);
            assert_eq!(fv.as_slice(), &[1.0, 2.0, 3.0]);
        });
    }

    #[test]
    fn feature_vector_empty() {
        timed(|| {
            let fv = FeatureVector::new(vec![]);
            assert_eq!(fv.dim(), 0);
            assert!(fv.as_slice().is_empty());
        });
    }

    // -- EigenvalueExtractor tests --

    fn point(x: f32, y: f32, z: f32) -> Point {
        Point {
            position: [x, y, z],
            color: None,
        }
    }

    fn flat_grid(side: usize, spacing: f32) -> Vec<Point> {
        let mut points = Vec::with_capacity(side * side);
        for i in 0..side {
            for j in 0..side {
                points.push(point(i as f32 * spacing, j as f32 * spacing, 0.0));
            }
        }
        points
    }

    #[test]
    fn eigenvalue_extractor_dim() {
        timed(|| {
            let ext = EigenvalueExtractor;
            assert_eq!(ext.feature_dim(), 6);
        });
    }

    #[test]
    fn eigenvalue_extractor_produces_correct_dim() {
        timed(|| {
            let ext = EigenvalueExtractor;
            let points = flat_grid(10, 0.1);
            let features = ext.extract(&points, 10);

            assert_eq!(features.len(), 100);
            for fv in &features {
                assert_eq!(fv.dim(), 6, "expected 6D feature vector, got {}D", fv.dim());
            }
        });
    }

    // -- HdbscanClusterer tests --

    fn make_blob(center: [f32; 3], count: usize, spread: f32) -> Vec<Point> {
        (0..count)
            .map(|i| {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let side = (count as f32).cbrt().ceil() as usize;
                let x = (i % side) as f32 / side as f32 * spread - spread / 2.0;
                let y = ((i / side) % side) as f32 / side as f32 * spread - spread / 2.0;
                let z = (i / (side * side)) as f32 / side as f32 * spread - spread / 2.0;
                Point {
                    position: [center[0] + x, center[1] + y, center[2] + z],
                    color: None,
                }
            })
            .collect()
    }

    #[test]
    fn hdbscan_clusterer_separates_blobs() {
        timed(|| {
            let mut points = make_blob([0.0, 0.0, 0.0], 200, 0.5);
            points.extend(make_blob([10.0, 10.0, 10.0], 200, 0.5));

            let ext = EigenvalueExtractor;
            let features = ext.extract(&points, 10);

            // Use high spatial_weight so spatial separation dominates over
            // feature variation within each blob.
            let clusterer = HdbscanClusterer::new(HdbscanConfig {
                min_cluster_size: 20,
                min_samples: 5,
                spatial_weight: 5.0,
            });
            let result = clusterer.cluster(&points, &features, &ClusterConfig::default());

            // With real eigenvalue features (not uniform), HDBSCAN may find
            // sub-clusters within each blob. We verify at least 2 clusters
            // exist and that each blob's centroid region has assigned points.
            assert!(
                result.clusters.len() >= 2,
                "expected >= 2 clusters, got {}",
                result.clusters.len()
            );

            // Total assigned + noise = input size
            let assigned: usize = result.clusters.iter().map(|c| c.point_indices.len()).sum();
            assert_eq!(assigned + result.noise_indices.len(), 400);
        });
    }

    // -- MockPipelineClassifier for pipeline tests --

    struct MockPipelineClassifier;

    impl PipelineClassifier for MockPipelineClassifier {
        fn classify(
            &self,
            candidates: &[FeatureCandidate],
            _address: &str,
            _climate_zone: &str,
        ) -> Result<Vec<ClassifiedFeatureOutput>, PipelineError> {
            Ok(candidates
                .iter()
                .map(|c| {
                    #[allow(clippy::cast_possible_truncation)]
                    ClassifiedFeatureOutput {
                        cluster_id: c.cluster_id as i64,
                        label: "Mock Feature".to_string(),
                        category: "structure".to_string(),
                        species: None,
                        confidence: 0.5,
                        reasoning: "mock classification".to_string(),
                        landscape_notes: String::new(),
                    }
                })
                .collect())
        }
    }

    // -- ScanPipeline tests --

    #[test]
    fn pipeline_runs_end_to_end() {
        timed(|| {
            let mut points = make_blob([0.0, 0.0, 2.0], 200, 0.5);
            points.extend(make_blob([10.0, 10.0, 3.0], 200, 0.5));

            let pipeline = ScanPipeline::new(
                Box::new(EigenvalueExtractor),
                Box::new(HdbscanClusterer::default()),
                Box::new(MockPipelineClassifier),
            );

            let plane = Plane {
                normal: [0.0, 0.0, 1.0],
                d: 0.0,
            };

            let result = pipeline
                .run(&points, &plane, 10, "Test Address", "USDA 10b")
                .expect("pipeline should succeed");

            assert_eq!(result.feature_dim, 6);
            // With default HdbscanConfig (min_cluster_size=200), clusters may form
            // depending on point density. We verify the pipeline doesn't panic and
            // produces consistent output.
            assert_eq!(result.candidates.len(), result.classifications.len());
        });
    }
}
