//! Reconciler error types.

/// Errors that can occur during site data reconciliation.
#[derive(Debug, thiserror::Error)]
pub enum ReconcilerError {
    /// LLM reconciliation failed (timeout, rate limit, bad response).
    #[error("LLM reconciliation failed: {0}")]
    Reconciliation(String),

    /// Input validation failed.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
