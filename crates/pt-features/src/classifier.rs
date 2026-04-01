//! Feature classifier trait and real (BAML) implementation.

use async_trait::async_trait;

use crate::baml_client::types::{ClassifiedFeature, FeatureCandidateInput};
use crate::error::ClassificationError;

/// Abstraction over LiDAR feature classification.
///
/// Implemented by [`BamlFeatureClassifier`] for real LLM calls,
/// [`ClaudeCliClassifier`](crate::ClaudeCliClassifier) for subscription dev,
/// and [`MockFeatureClassifier`](crate::MockFeatureClassifier) for tests.
#[async_trait]
pub trait FeatureClassifier: Send + Sync {
    async fn classify(
        &self,
        candidates: &[pt_scan::FeatureCandidate],
        address: &str,
        climate_zone: &str,
    ) -> Result<Vec<ClassifiedFeature>, ClassificationError>;
}

/// Real implementation that calls the BAML-generated client.
///
/// Requires `ANTHROPIC_API_KEY` (or equivalent) at runtime.
#[derive(Debug)]
pub struct BamlFeatureClassifier;

#[async_trait]
impl FeatureClassifier for BamlFeatureClassifier {
    async fn classify(
        &self,
        candidates: &[pt_scan::FeatureCandidate],
        address: &str,
        climate_zone: &str,
    ) -> Result<Vec<ClassifiedFeature>, ClassificationError> {
        let inputs: Vec<FeatureCandidateInput> = candidates.iter().map(to_baml_input).collect();

        crate::B
            .ClassifyFeatures
            .call(&inputs, address, climate_zone)
            .await
            .map_err(|e| ClassificationError::Classification(e.to_string()))
    }
}

/// Convert a domain `FeatureCandidate` to the BAML-generated input type.
#[allow(clippy::cast_possible_truncation)]
fn to_baml_input(c: &pt_scan::FeatureCandidate) -> FeatureCandidateInput {
    FeatureCandidateInput {
        cluster_id: c.cluster_id as i64,
        centroid_x: c.centroid[0],
        centroid_y: c.centroid[1],
        centroid_z: c.centroid[2],
        height_ft: c.height_ft,
        spread_ft: c.spread_ft,
        point_count: c.point_count as i64,
        dominant_color: c.dominant_color.clone(),
        vertical_profile: c.vertical_profile.clone(),
        density: c.density,
    }
}
