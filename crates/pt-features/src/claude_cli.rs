//! [`FeatureClassifier`] backed by the `claude` CLI.
//!
//! Routes LLM calls through the local Claude Code process (subscription),
//! then parses the response with BAML. Zero API token cost.

use async_trait::async_trait;
use std::process::Command;

use crate::baml_client::types::ClassifiedFeature;
use crate::classifier::FeatureClassifier;
use crate::error::ClassificationError;

/// Calls the `claude` CLI (using your subscription) and parses with BAML.
///
/// Strips `ANTHROPIC_API_KEY` and `CLAUDECODE` from the subprocess env
/// to force the CLI to use session auth rather than API tokens.
#[derive(Debug)]
pub struct ClaudeCliClassifier;

impl ClaudeCliClassifier {
    fn build_prompt(
        candidates: &[pt_scan::FeatureCandidate],
        address: &str,
        climate_zone: &str,
    ) -> Result<String, ClassificationError> {
        let candidates_json = serde_json::to_string_pretty(candidates)
            .map_err(|e| ClassificationError::InvalidInput(format!("JSON serialize: {e}")))?;

        Ok(format!(
            r#"You are a landscape design expert and certified arborist analyzing LiDAR scan data.
Classify each detected feature from its geometric profile.

Site: {address}
Climate: {climate_zone}

Candidates (JSON):
{candidates_json}

For each candidate, provide:
- cluster_id: echo back the input cluster_id
- label: Human-readable name (e.g., "London Plane Tree", "Street Light Pole")
- category: One of "tree", "structure", "hardscape", "planting", "utility"
- species: Botanical name for vegetation, or null for non-vegetation
- confidence: 0.0-1.0
- reasoning: WHY your classification matches the geometric profile
- landscape_notes: Design implications for a landscaper

Respond with ONLY valid JSON — an array of objects, no markdown fences."#
        ))
    }

    fn call_cli(prompt: &str) -> Result<String, ClassificationError> {
        let output = Command::new("claude")
            .args(["-p", prompt, "--output-format", "text"])
            .env_remove("ANTHROPIC_API_KEY")
            .env_remove("CLAUDECODE")
            .output()
            .map_err(|e| {
                ClassificationError::Classification(format!(
                    "failed to run `claude` CLI (is it installed?): {e}"
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ClassificationError::Classification(format!(
                "claude CLI exited {}: {stderr}",
                output.status
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout).to_string();
        if raw.trim().is_empty() {
            return Err(ClassificationError::Classification(
                "claude CLI returned empty response".into(),
            ));
        }

        Ok(raw)
    }
}

#[async_trait]
impl FeatureClassifier for ClaudeCliClassifier {
    async fn classify(
        &self,
        candidates: &[pt_scan::FeatureCandidate],
        address: &str,
        climate_zone: &str,
    ) -> Result<Vec<ClassifiedFeature>, ClassificationError> {
        let prompt = Self::build_prompt(candidates, address, climate_zone)?;

        let raw = tokio::task::spawn_blocking(move || Self::call_cli(&prompt))
            .await
            .map_err(|e| ClassificationError::Classification(format!("task join error: {e}")))??;

        crate::B.ClassifyFeatures.parse(&raw).map_err(|e| {
            ClassificationError::Classification(format!(
                "BAML parse failed: {e}\n\nRaw response:\n{raw}"
            ))
        })
    }
}
