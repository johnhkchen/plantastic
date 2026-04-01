//! [`PlanterEstimator`] backed by the `claude` CLI.
//!
//! Routes LLM calls through the local Claude Code process (subscription),
//! then parses the response with BAML. Zero API token cost.

use async_trait::async_trait;
use std::process::Command;

use crate::baml_client::types::PlanterEstimate;
use crate::error::PlanterError;
use crate::estimator::{PlanterEstimator, PlanterInput};

/// Calls the `claude` CLI (using your subscription) and parses with BAML.
///
/// Strips `ANTHROPIC_API_KEY` and `CLAUDECODE` from the subprocess env
/// to force the CLI to use session auth rather than API tokens.
#[derive(Debug)]
pub struct ClaudeCliEstimator;

impl ClaudeCliEstimator {
    fn build_prompt(input: &PlanterInput) -> String {
        let features = input.adjacent_features.join(", ");
        let sun_line = input
            .sun_hours
            .map(|h| format!("\nEstimated sun hours: {h} hours/day"))
            .unwrap_or_default();

        format!(
            r#"You are an expert landscape designer selecting plants for a measured gap
between existing features in an urban landscape.

Site: {address}
Climate: {climate_zone}

Gap dimensions:
- Width: {width} ft
- Length: {length} ft
- Area: {area} sq ft

Adjacent features: {features}{sun_line}

Design exactly 3 planter styles ranging from budget-friendly to premium.
Each style should feel like a real landscape design option.

For each style, provide:
- style_name: Evocative name (e.g., "Mediterranean Groundcover", "Woodland Edge")
- description: 1-2 sentences on the look and feel
- plant_selections: 2-4 plants, each with common_name, botanical_name, spacing_inches, why_this_plant
- soil_depth_inches: Required soil depth (4-12 inches)
- design_rationale: Why this combination works here

All plants must suit the climate zone. Style 1 = lowest cost, Style 3 = premium.
Do NOT compute quantities or costs.

Respond with ONLY valid JSON matching this schema (no markdown fences):
{{
  "styles": [
    {{
      "style_name": "...",
      "description": "...",
      "plant_selections": [
        {{"common_name": "...", "botanical_name": "...", "spacing_inches": 8.0, "why_this_plant": "..."}}
      ],
      "soil_depth_inches": 6.0,
      "design_rationale": "..."
    }}
  ]
}}"#,
            address = input.address,
            climate_zone = input.climate_zone,
            width = input.gap_width_ft,
            length = input.gap_length_ft,
            area = input.area_sqft,
        )
    }

    fn call_cli(prompt: &str) -> Result<String, PlanterError> {
        let output = Command::new("claude")
            .args(["-p", prompt, "--output-format", "text"])
            .env_remove("ANTHROPIC_API_KEY")
            .env_remove("CLAUDECODE")
            .output()
            .map_err(|e| {
                PlanterError::Estimation(format!(
                    "failed to run `claude` CLI (is it installed?): {e}"
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PlanterError::Estimation(format!(
                "claude CLI exited {}: {stderr}",
                output.status
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout).to_string();
        if raw.trim().is_empty() {
            return Err(PlanterError::Estimation(
                "claude CLI returned empty response".into(),
            ));
        }

        Ok(raw)
    }
}

#[async_trait]
impl PlanterEstimator for ClaudeCliEstimator {
    async fn estimate(&self, input: &PlanterInput) -> Result<PlanterEstimate, PlanterError> {
        let prompt = Self::build_prompt(input);

        let raw = tokio::task::spawn_blocking(move || Self::call_cli(&prompt))
            .await
            .map_err(|e| PlanterError::Estimation(format!("task join error: {e}")))??;

        crate::B.EstimatePlanter.parse(&raw).map_err(|e| {
            PlanterError::Estimation(format!("BAML parse failed: {e}\n\nRaw response:\n{raw}"))
        })
    }
}
