//! Site analyzer trait and real (BAML) implementation.

use async_trait::async_trait;
use base64::Engine;

use crate::baml_client::types::{ClassifiedFeature, SiteAnalysis};
use crate::error::AnalyzerError;

/// Bundled input for site analysis.
///
/// Mirrors the parameters of the BAML `AnalyzePlanView` function.
#[derive(Debug, Clone)]
pub struct SiteAnalyzerInput {
    pub plan_view_png: Vec<u8>,
    pub lot_dimensions: String,
    pub address: String,
    pub classified_features: Vec<ClassifiedFeature>,
}

/// Abstraction over LLM-powered site analysis.
///
/// Implemented by [`BamlSiteAnalyzer`] for real LLM calls,
/// [`ClaudeCliAnalyzer`](crate::ClaudeCliAnalyzer) for subscription dev,
/// and [`MockSiteAnalyzer`](crate::MockSiteAnalyzer) for tests.
#[async_trait]
pub trait SiteAnalyzer: Send + Sync {
    async fn analyze(&self, input: &SiteAnalyzerInput) -> Result<SiteAnalysis, AnalyzerError>;
}

/// Real implementation that calls the BAML-generated client.
///
/// Requires `ANTHROPIC_API_KEY` (or equivalent) at runtime.
#[derive(Debug)]
pub struct BamlSiteAnalyzer;

#[async_trait]
impl SiteAnalyzer for BamlSiteAnalyzer {
    async fn analyze(&self, input: &SiteAnalyzerInput) -> Result<SiteAnalysis, AnalyzerError> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&input.plan_view_png);
        let image = crate::baml_client::new_image_from_base64(&b64, Some("image/png"));

        crate::B
            .AnalyzePlanView
            .call(
                &image,
                &input.lot_dimensions,
                &input.address,
                &input.classified_features,
            )
            .await
            .map_err(|e| AnalyzerError::Analysis(e.to_string()))
    }
}
