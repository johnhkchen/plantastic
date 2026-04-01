//! [`ProposalNarrativeGenerator`] backed by the `claude` CLI.
//!
//! Routes LLM calls through the local Claude Code process (subscription),
//! then parses the response with BAML. Zero API token cost.
//!
//! This is the pattern from ssjo-grant-agent: BAML builds the prompt schema,
//! the `claude` CLI provides the LLM transport, BAML parses the response.

use async_trait::async_trait;
use std::process::Command;

use crate::error::ProposalError;
use crate::generator::{ProposalInput, ProposalNarrativeGenerator};
use crate::ProposalContent;

/// Calls the `claude` CLI (using your subscription) and parses with BAML.
///
/// Strips `ANTHROPIC_API_KEY` and `CLAUDECODE` from the subprocess env
/// to force the CLI to use session auth rather than API tokens.
#[derive(Debug)]
pub struct ClaudeCliGenerator;

impl ClaudeCliGenerator {
    fn build_prompt(input: &ProposalInput) -> String {
        let mut tiers_desc = String::new();
        for tier in &input.tiers {
            tiers_desc.push_str(&format!("\n- {} ({}):", tier.tier_level, tier.total));
            for zone in &tier.zones {
                tiers_desc.push_str(&format!(
                    "\n  {} ({} sqft, {}): {}",
                    zone.label,
                    zone.area_sqft,
                    zone.zone_type,
                    zone.materials.join(", ")
                ));
            }
        }

        format!(
            r#"You are writing a professional landscaping proposal on behalf of {company}.
The project is "{project}" at {address}.

Write warm, professional prose. The homeowner will read this.
Do NOT invent or recalculate dollar amounts — use the exact totals provided.

Tiers:{tiers}

Respond with ONLY valid JSON matching this exact schema (no markdown fences, no explanation outside the JSON):
{{
  "intro_paragraph": "2-3 sentence greeting and project overview",
  "tier_narratives": [
    {{"tier_level": "Good", "headline": "short headline", "description": "2-3 sentences", "differentiators": ["point 1", "point 2"]}},
    {{"tier_level": "Better", "headline": "short headline", "description": "2-3 sentences", "differentiators": ["point 1", "point 2"]}},
    {{"tier_level": "Best", "headline": "short headline", "description": "2-3 sentences", "differentiators": ["point 1", "point 2"]}}
  ],
  "zone_callouts": [
    {{"zone_label": "zone name", "note": "1-2 sentences about this zone"}}
  ],
  "closing_paragraph": "2-3 sentence CTA and next steps"
}}"#,
            company = input.company_name,
            project = input.project_name,
            address = input.project_address,
            tiers = tiers_desc,
        )
    }

    fn call_cli(prompt: &str) -> Result<String, ProposalError> {
        let output = Command::new("claude")
            .args(["-p", prompt, "--output-format", "text"])
            .env_remove("ANTHROPIC_API_KEY")
            .env_remove("CLAUDECODE")
            .output()
            .map_err(|e| {
                ProposalError::Generation(format!(
                    "failed to run `claude` CLI (is it installed?): {e}"
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProposalError::Generation(format!(
                "claude CLI exited {}: {stderr}",
                output.status
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout).to_string();
        if raw.trim().is_empty() {
            return Err(ProposalError::Generation(
                "claude CLI returned empty response".into(),
            ));
        }

        Ok(raw)
    }
}

#[async_trait]
impl ProposalNarrativeGenerator for ClaudeCliGenerator {
    async fn generate(&self, input: &ProposalInput) -> Result<ProposalContent, ProposalError> {
        let prompt = Self::build_prompt(input);

        // Run CLI in a blocking task (it spawns a subprocess)
        let raw = tokio::task::spawn_blocking(move || Self::call_cli(&prompt))
            .await
            .map_err(|e| ProposalError::Generation(format!("task join error: {e}")))??;

        // Parse with BAML — it knows the ProposalContent schema
        crate::B.GenerateProposalNarrative.parse(&raw).map_err(|e| {
            ProposalError::Generation(format!("BAML parse failed: {e}\n\nRaw response:\n{raw}"))
        })
    }
}
