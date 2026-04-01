//! Site analysis error types.

/// Errors that can occur during site analysis.
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    /// LLM analysis failed (timeout, rate limit, bad response).
    #[error("LLM analysis failed: {0}")]
    Analysis(String),

    /// Input validation failed.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
