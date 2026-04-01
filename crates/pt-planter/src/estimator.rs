//! Planter estimator trait and real (BAML) implementation.

use async_trait::async_trait;

use crate::baml_client::types::PlanterEstimate;
use crate::error::PlanterError;

/// Bundled input for planter estimation.
///
/// Mirrors the parameters of the BAML `EstimatePlanter` function.
#[derive(Debug, Clone)]
pub struct PlanterInput {
    pub gap_width_ft: f64,
    pub gap_length_ft: f64,
    pub area_sqft: f64,
    pub adjacent_features: Vec<String>,
    pub sun_hours: Option<i64>,
    pub climate_zone: String,
    pub address: String,
}

/// Abstraction over LLM-powered planter estimation.
///
/// Implemented by [`BamlPlanterEstimator`] for real LLM calls,
/// [`ClaudeCliEstimator`](crate::ClaudeCliEstimator) for subscription dev,
/// and [`MockPlanterEstimator`](crate::MockPlanterEstimator) for tests.
#[async_trait]
pub trait PlanterEstimator: Send + Sync {
    async fn estimate(&self, input: &PlanterInput) -> Result<PlanterEstimate, PlanterError>;
}

/// Real implementation that calls the BAML-generated client.
///
/// Requires `ANTHROPIC_API_KEY` (or equivalent) at runtime.
#[derive(Debug)]
pub struct BamlPlanterEstimator;

#[async_trait]
impl PlanterEstimator for BamlPlanterEstimator {
    async fn estimate(&self, input: &PlanterInput) -> Result<PlanterEstimate, PlanterError> {
        let features: Vec<String> = input.adjacent_features.clone();

        crate::B
            .EstimatePlanter
            .call(
                input.gap_width_ft,
                input.gap_length_ft,
                input.area_sqft,
                &features,
                input.sun_hours,
                &input.climate_zone,
                &input.address,
            )
            .await
            .map_err(|e| PlanterError::Estimation(e.to_string()))
    }
}
