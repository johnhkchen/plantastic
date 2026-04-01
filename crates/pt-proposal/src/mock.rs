//! Mock implementations of [`ProposalNarrativeGenerator`] for tests.

use async_trait::async_trait;

use crate::error::ProposalError;
use crate::generator::{ProposalInput, ProposalNarrativeGenerator};
use crate::{ProposalContent, TierNarrative, ZoneCallout};

/// Returns deterministic, realistic proposal narratives derived from input.
///
/// Same input always produces the same output. The text references actual
/// zone labels, tier levels, and project details so screenshot tests look
/// plausible.
#[derive(Debug)]
pub struct MockProposalGenerator;

#[async_trait]
impl ProposalNarrativeGenerator for MockProposalGenerator {
    async fn generate(&self, input: &ProposalInput) -> Result<ProposalContent, ProposalError> {
        let tier_narratives: Vec<TierNarrative> = input
            .tiers
            .iter()
            .map(|tier| {
                let zone_names: Vec<&str> = tier.zones.iter().map(|z| z.label.as_str()).collect();
                let zones_list = zone_names.join(", ");

                TierNarrative {
                    tier_level: tier.tier_level.clone(),
                    headline: format!("{} Package for {}", tier.tier_level, input.project_name),
                    description: format!(
                        "This {} option for {} at {} transforms your outdoor space \
                         across {} zones ({}) with carefully selected materials \
                         and professional installation, totaling {}.",
                        tier.tier_level.to_lowercase(),
                        input.project_name,
                        input.project_address,
                        tier.zones.len(),
                        zones_list,
                        tier.total,
                    ),
                    differentiators: vec![
                        format!("Covers {} distinct zones", tier.zones.len()),
                        format!("All-inclusive {} pricing", tier.total),
                        "Professional installation and cleanup".to_string(),
                    ],
                }
            })
            .collect();

        let zone_callouts: Vec<ZoneCallout> = input
            .tiers
            .first()
            .map(|first_tier| {
                first_tier
                    .zones
                    .iter()
                    .map(|zone| ZoneCallout {
                        zone_label: zone.label.clone(),
                        note: format!(
                            "{} zone ({} sq ft) featuring {} — designed to complement \
                             the overall landscape vision for {}.",
                            zone.label,
                            zone.area_sqft,
                            zone.materials.join(", "),
                            input.project_name,
                        ),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(ProposalContent {
            intro_paragraph: format!(
                "Thank you for choosing {} for your {} project at {}. \
                 We are pleased to present this comprehensive landscape proposal \
                 tailored to your property's unique characteristics and your vision \
                 for a beautiful, functional outdoor living space.",
                input.company_name, input.project_name, input.project_address,
            ),
            tier_narratives,
            zone_callouts,
            closing_paragraph: format!(
                "We at {} look forward to bringing your vision for {} to life. \
                 Our team is committed to quality craftsmanship and attention to detail \
                 throughout every phase of the project. Please don't hesitate to reach \
                 out with any questions.",
                input.company_name, input.project_name,
            ),
        })
    }
}

/// Always returns a generation error. Use for testing error-handling paths.
#[derive(Debug)]
pub struct MockFailingGenerator;

#[async_trait]
impl ProposalNarrativeGenerator for MockFailingGenerator {
    async fn generate(&self, _input: &ProposalInput) -> Result<ProposalContent, ProposalError> {
        Err(ProposalError::Generation(
            "mock LLM failure: rate limit exceeded".to_string(),
        ))
    }
}
