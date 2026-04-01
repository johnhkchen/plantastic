//! Site data reconciliation via BAML.
//!
//! This crate provides a trait abstraction ([`SiteReconciler`]) over
//! LLM-powered reconciliation of scan features, satellite baseline, and
//! plan-view analysis, with a real BAML implementation
//! ([`BamlSiteReconciler`]), a Claude CLI fallback
//! ([`ClaudeCliReconciler`]), and a deterministic mock
//! ([`MockSiteReconciler`]) for tests.
//!
//! The LLM merges all three data sources into a [`ReconciledSite`] with
//! confirmed/scan-only/satellite-only features, discrepancies, and
//! zone recommendations informed by sun exposure data.

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
pub mod convert;
mod error;
mod mock;
mod reconciler;

// Re-export generated types for downstream use.
pub use baml_client::types::{
    ClassifiedFeature, Discrepancy, RecommendedZone, ReconciledFeature, ReconciledSite,
    SatelliteBaseline, SatelliteTree, SiteAnalysis,
};

// Re-export the async client singleton and function struct.
pub use baml_client::async_client::ReconcileSiteData;
pub use baml_client::B;

// Re-export trait, implementations, and error types.
pub use claude_cli::ClaudeCliReconciler;
pub use error::ReconcilerError;
pub use mock::{MockFailingReconciler, MockSiteReconciler};
pub use reconciler::{BamlSiteReconciler, ReconcilerInput, SiteReconciler};
