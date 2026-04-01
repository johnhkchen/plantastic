//! Planter estimation via BAML.
//!
//! This crate provides a trait abstraction ([`PlanterEstimator`]) over
//! LLM-powered planter estimation, with a real BAML implementation
//! ([`BamlPlanterEstimator`]), a Claude CLI fallback ([`ClaudeCliEstimator`]),
//! and a deterministic mock ([`MockPlanterEstimator`]) for tests.
//!
//! The LLM picks plants and explains design reasoning. Code computes
//! quantities and costs via the [`compute`] module.

#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    non_snake_case,
    unused_imports,
    non_camel_case_types,
    dead_code,
    missing_debug_implementations,
    unused_must_use
)]
#[path = "../../../baml_client/mod.rs"]
mod baml_client;

pub mod claude_cli;
pub mod compute;
mod error;
mod estimator;
mod mock;

// Re-export generated types for downstream use.
pub use baml_client::types::{PlantSelection, PlanterEstimate, PlanterStyle};

// Re-export the async client singleton and function struct.
pub use baml_client::async_client::EstimatePlanter;
pub use baml_client::B;

// Re-export trait, implementations, and error types.
pub use claude_cli::ClaudeCliEstimator;
pub use compute::{compute_style, ComputedPlanting, ComputedStyle};
pub use error::PlanterError;
pub use estimator::{BamlPlanterEstimator, PlanterEstimator, PlanterInput};
pub use mock::{MockFailingEstimator, MockPlanterEstimator};
