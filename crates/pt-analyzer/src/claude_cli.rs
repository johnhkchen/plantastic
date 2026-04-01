//! [`SiteAnalyzer`] backed by the `claude` CLI.
//!
//! Routes LLM calls through the local Claude Code process (subscription),
//! then parses the response with BAML. Zero API token cost.
//!
//! Note: the CLI cannot send images as multimodal content blocks, so this
//! implementation describes the features textually without the plan view image.
//! Use [`BamlSiteAnalyzer`](crate::BamlSiteAnalyzer) for full multimodal analysis.

use async_trait::async_trait;
use std::process::Command;

use crate::analyzer::{SiteAnalyzer, SiteAnalyzerInput};
use crate::baml_client::types::{ClassifiedFeature, SiteAnalysis};
use crate::error::AnalyzerError;

/// Calls the `claude` CLI (using your subscription) and parses with BAML.
///
/// Strips `ANTHROPIC_API_KEY` and `CLAUDECODE` from the subprocess env
/// to force the CLI to use session auth rather than API tokens.
#[derive(Debug)]
pub struct ClaudeCliAnalyzer;

impl ClaudeCliAnalyzer {
    fn build_prompt(input: &SiteAnalyzerInput) -> Result<String, AnalyzerError> {
        let features_text = format_features(&input.classified_features);

        Ok(format!(
            r#"You are a senior landscape architect analyzing a site plan.

Address: {address}
Lot dimensions: {lot_dimensions}

(Note: plan view image not available in CLI mode — analyze from feature data only.)

Classified features from LiDAR scan:
{features_text}

Your task:
1. Echo back the classified features exactly
2. Suggest 3-5 zone placements that maximize landscape value
3. Provide 3-4 site observations demonstrating expert spatial reasoning

For each zone: label, zone_type (patio/bed/edging/fill/lawn/seating/pathway/screening),
rationale (reference specific features), approximate_area_sqft.

For each observation: a free-form insight about the site.

Respond with ONLY valid JSON matching this schema (no markdown fences):
{{
  "features": [...],
  "suggested_zones": [
    {{"label": "...", "zone_type": "...", "rationale": "...", "approximate_area_sqft": 0.0}}
  ],
  "site_observations": [
    {{"observation": "..."}}
  ]
}}"#,
            address = input.address,
            lot_dimensions = input.lot_dimensions,
        ))
    }

    fn call_cli(prompt: &str) -> Result<String, AnalyzerError> {
        let output = Command::new("claude")
            .args(["-p", prompt, "--output-format", "text"])
            .env_remove("ANTHROPIC_API_KEY")
            .env_remove("CLAUDECODE")
            .output()
            .map_err(|e| {
                AnalyzerError::Analysis(format!(
                    "failed to run `claude` CLI (is it installed?): {e}"
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AnalyzerError::Analysis(format!(
                "claude CLI exited {}: {stderr}",
                output.status
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout).to_string();
        if raw.trim().is_empty() {
            return Err(AnalyzerError::Analysis(
                "claude CLI returned empty response".into(),
            ));
        }

        Ok(raw)
    }
}

#[async_trait]
impl SiteAnalyzer for ClaudeCliAnalyzer {
    async fn analyze(&self, input: &SiteAnalyzerInput) -> Result<SiteAnalysis, AnalyzerError> {
        let prompt = Self::build_prompt(input)?;

        let raw = tokio::task::spawn_blocking(move || Self::call_cli(&prompt))
            .await
            .map_err(|e| AnalyzerError::Analysis(format!("task join error: {e}")))??;

        crate::B.AnalyzePlanView.parse(&raw).map_err(|e| {
            AnalyzerError::Analysis(format!("BAML parse failed: {e}\n\nRaw response:\n{raw}"))
        })
    }
}

/// Format classified features as readable text for the CLI prompt.
fn format_features(features: &[ClassifiedFeature]) -> String {
    features
        .iter()
        .map(|f| {
            format!(
                "- [{}] {} (category: {}, confidence: {:.2})\n  {}",
                f.cluster_id, f.label, f.category, f.confidence, f.reasoning,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
