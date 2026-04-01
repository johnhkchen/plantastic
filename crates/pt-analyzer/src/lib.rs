//! Site analysis via BAML.
//!
//! This crate provides a trait abstraction ([`SiteAnalyzer`]) over
//! LLM-powered site analysis from annotated plan views, with a real BAML
//! implementation ([`BamlSiteAnalyzer`]), a Claude CLI fallback
//! ([`ClaudeCliAnalyzer`]), and a deterministic mock ([`MockSiteAnalyzer`])
//! for tests.
//!
//! The LLM analyzes an annotated plan-view PNG alongside classified features
//! to produce zone suggestions and site observations.

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

mod analyzer;
pub mod claude_cli;
mod error;
mod mock;

// Re-export generated types for downstream use.
pub use baml_client::types::{ClassifiedFeature, SiteAnalysis, SiteObservation, SuggestedZone};

// Re-export the async client singleton and function struct.
pub use baml_client::async_client::AnalyzePlanView;
pub use baml_client::B;

// Re-export trait, implementations, and error types.
pub use analyzer::{BamlSiteAnalyzer, SiteAnalyzer, SiteAnalyzerInput};
pub use claude_cli::ClaudeCliAnalyzer;
pub use error::AnalyzerError;
pub use mock::{MockFailingAnalyzer, MockSiteAnalyzer};
