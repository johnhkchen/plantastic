//! Feature classification error types.

/// Errors that can occur during feature classification.
#[derive(Debug, thiserror::Error)]
pub enum ClassificationError {
    /// LLM classification failed (timeout, rate limit, bad response).
    #[error("LLM classification failed: {0}")]
    Classification(String),

    /// Input validation failed.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
