//! Proposal narrative generator trait and real (BAML) implementation.

use async_trait::async_trait;

use crate::baml_client::types::TierInput;
use crate::error::ProposalError;
use crate::ProposalContent;

/// Bundled input for proposal narrative generation.
///
/// Mirrors the parameters of the BAML `GenerateProposalNarrative` function.
#[derive(Debug, Clone)]
pub struct ProposalInput {
    pub company_name: String,
    pub project_name: String,
    pub project_address: String,
    pub tiers: Vec<TierInput>,
}

/// Abstraction over proposal narrative generation.
///
/// Implemented by [`BamlProposalGenerator`] for real LLM calls and by
/// [`MockProposalGenerator`](crate::MockProposalGenerator) for tests.
#[async_trait]
pub trait ProposalNarrativeGenerator: Send + Sync {
    async fn generate(&self, input: &ProposalInput) -> Result<ProposalContent, ProposalError>;
}

/// Real implementation that calls the BAML-generated client.
///
/// Requires `ANTHROPIC_API_KEY` (or equivalent) at runtime.
#[derive(Debug)]
pub struct BamlProposalGenerator;

#[async_trait]
impl ProposalNarrativeGenerator for BamlProposalGenerator {
    async fn generate(&self, input: &ProposalInput) -> Result<ProposalContent, ProposalError> {
        crate::B
            .GenerateProposalNarrative
            .call(
                &input.company_name,
                &input.project_name,
                &input.project_address,
                &input.tiers,
            )
            .await
            .map_err(|e| ProposalError::Generation(e.to_string()))
    }
}
