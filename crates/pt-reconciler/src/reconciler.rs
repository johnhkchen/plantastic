//! Site reconciler trait and real (BAML) implementation.

use async_trait::async_trait;

use crate::baml_client::types::{
    ClassifiedFeature, ReconciledSite, SatelliteBaseline, SiteAnalysis,
};
use crate::error::ReconcilerError;

/// Bundled input for site data reconciliation.
///
/// Mirrors the parameters of the BAML `ReconcileSiteData` function.
#[derive(Debug, Clone)]
pub struct ReconcilerInput {
    pub scan_features: Vec<ClassifiedFeature>,
    pub satellite_baseline: SatelliteBaseline,
    pub plan_view_analysis: SiteAnalysis,
    pub address: String,
}

/// Abstraction over LLM-powered site data reconciliation.
///
/// Implemented by [`BamlSiteReconciler`] for real LLM calls,
/// [`ClaudeCliReconciler`](crate::ClaudeCliReconciler) for subscription dev,
/// and [`MockSiteReconciler`](crate::MockSiteReconciler) for tests.
#[async_trait]
pub trait SiteReconciler: Send + Sync {
    async fn reconcile(&self, input: &ReconcilerInput) -> Result<ReconciledSite, ReconcilerError>;
}

/// Real implementation that calls the BAML-generated client.
///
/// Requires `ANTHROPIC_API_KEY` (or equivalent) at runtime.
#[derive(Debug)]
pub struct BamlSiteReconciler;

#[async_trait]
impl SiteReconciler for BamlSiteReconciler {
    async fn reconcile(&self, input: &ReconcilerInput) -> Result<ReconciledSite, ReconcilerError> {
        crate::B
            .ReconcileSiteData
            .call(
                &input.scan_features,
                &input.satellite_baseline,
                &input.plan_view_analysis,
                &input.address,
            )
            .await
            .map_err(|e| ReconcilerError::Reconciliation(e.to_string()))
    }
}
