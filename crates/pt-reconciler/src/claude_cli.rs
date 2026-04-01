//! [`SiteReconciler`] backed by the `claude` CLI.
//!
//! Routes LLM calls through the local Claude Code process (subscription),
//! then parses the response with BAML. Zero API token cost.

use async_trait::async_trait;
use std::process::Command;

use crate::baml_client::types::ReconciledSite;
use crate::baml_client::types::{ClassifiedFeature, SatelliteBaseline};
use crate::error::ReconcilerError;
use crate::reconciler::{ReconcilerInput, SiteReconciler};

/// Calls the `claude` CLI (using your subscription) and parses with BAML.
///
/// Strips `ANTHROPIC_API_KEY` and `CLAUDECODE` from the subprocess env
/// to force the CLI to use session auth rather than API tokens.
#[derive(Debug)]
pub struct ClaudeCliReconciler;

impl ClaudeCliReconciler {
    fn build_prompt(input: &ReconcilerInput) -> String {
        let features_text = format_features(&input.scan_features);
        let satellite_text = format_satellite(&input.satellite_baseline);
        let zones_text = format_zones(input);
        let observations_text = format_observations(input);

        format!(
            r#"You are a senior landscape architect with GIS expertise, reconciling
multiple data sources for a site assessment.

Address: {address}

=== SATELLITE BASELINE (pre-visit remote sensing) ===
{satellite_text}

=== LIDAR SCAN FEATURES (current ground truth) ===
{features_text}

=== PLAN-VIEW ANALYSIS ===
Suggested zones:
{zones_text}

Site observations:
{observations_text}

Your task: reconcile these three data sources into a unified site model.

Feature reconciliation:
- confirmed_features: seen in BOTH scan and satellite
- scan_only_features: detected by LiDAR but not satellite
- satellite_only_features: in satellite but NOT scan (possible removal)
- source: "scan", "satellite", or "both"

Temporal reasoning: satellite is historical, scan is current ground truth.

Discrepancies: flag conflicts with explanation and design implication.

Recommended zones: build on plan-view suggestions informed by all sources.
zone_type: patio/bed/edging/fill/lawn/seating/pathway/screening
sun_exposure_hours: estimated hours for this zone (null if unknown)
data_sources: which sources informed this (e.g., "scan, satellite, plan_view")

Respond with ONLY valid JSON matching this schema (no markdown fences):
{{
  "confirmed_features": [{{"label": "...", "category": "...", "source": "both", "confidence": 0.0, "reasoning": "..."}}],
  "scan_only_features": [...],
  "satellite_only_features": [...],
  "discrepancies": [{{"description": "...", "possible_explanation": "...", "design_implication": "..."}}],
  "recommended_zones": [{{"label": "...", "zone_type": "...", "rationale": "...", "approximate_area_sqft": 0.0, "sun_exposure_hours": null, "data_sources": "..."}}]
}}"#,
            address = input.address,
        )
    }

    fn call_cli(prompt: &str) -> Result<String, ReconcilerError> {
        let output = Command::new("claude")
            .args(["-p", prompt, "--output-format", "text"])
            .env_remove("ANTHROPIC_API_KEY")
            .env_remove("CLAUDECODE")
            .output()
            .map_err(|e| {
                ReconcilerError::Reconciliation(format!(
                    "failed to run `claude` CLI (is it installed?): {e}"
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReconcilerError::Reconciliation(format!(
                "claude CLI exited {}: {stderr}",
                output.status
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout).to_string();
        if raw.trim().is_empty() {
            return Err(ReconcilerError::Reconciliation(
                "claude CLI returned empty response".into(),
            ));
        }

        Ok(raw)
    }
}

#[async_trait]
impl SiteReconciler for ClaudeCliReconciler {
    async fn reconcile(&self, input: &ReconcilerInput) -> Result<ReconciledSite, ReconcilerError> {
        let prompt = Self::build_prompt(input);

        let raw = tokio::task::spawn_blocking(move || Self::call_cli(&prompt))
            .await
            .map_err(|e| ReconcilerError::Reconciliation(format!("task join error: {e}")))??;

        crate::B.ReconcileSiteData.parse(&raw).map_err(|e| {
            ReconcilerError::Reconciliation(format!(
                "BAML parse failed: {e}\n\nRaw response:\n{raw}"
            ))
        })
    }
}

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

fn format_satellite(baseline: &SatelliteBaseline) -> String {
    let trees: Vec<String> = baseline
        .trees
        .iter()
        .enumerate()
        .map(|(i, t)| {
            format!(
                "  Tree {}: {:.0} ft tall, {:.0} ft spread, confidence {:.2}",
                i + 1,
                t.height_ft,
                t.spread_ft,
                t.confidence,
            )
        })
        .collect();

    format!(
        "Lot area: {:.0} sq ft\nAverage sun exposure: {:.1} hours/day\nDetected trees:\n{}",
        baseline.lot_area_sqft,
        baseline.avg_sun_hours,
        trees.join("\n"),
    )
}

fn format_zones(input: &ReconcilerInput) -> String {
    input
        .plan_view_analysis
        .suggested_zones
        .iter()
        .map(|z| {
            format!(
                "- {} ({}): {} (~{:.0} sqft)",
                z.label, z.zone_type, z.rationale, z.approximate_area_sqft,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_observations(input: &ReconcilerInput) -> String {
    input
        .plan_view_analysis
        .site_observations
        .iter()
        .map(|o| format!("- {}", o.observation))
        .collect::<Vec<_>>()
        .join("\n")
}
