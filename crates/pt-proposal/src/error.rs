//! Proposal generation error types.

/// Errors that can occur during proposal narrative generation.
#[derive(Debug, thiserror::Error)]
pub enum ProposalError {
    /// LLM generation failed (timeout, rate limit, bad response).
    #[error("LLM generation failed: {0}")]
    Generation(String),

    /// Input validation failed.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
