//! Planter estimation error types.

/// Errors that can occur during planter estimation.
#[derive(Debug, thiserror::Error)]
pub enum PlanterError {
    /// LLM estimation failed (timeout, rate limit, bad response).
    #[error("LLM estimation failed: {0}")]
    Estimation(String),

    /// Input validation failed.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
