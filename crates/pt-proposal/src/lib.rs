//! Proposal narrative generation via BAML.
//!
//! This crate provides a trait abstraction ([`ProposalNarrativeGenerator`]) over
//! LLM-powered proposal narrative generation, with a real BAML implementation
//! ([`BamlProposalGenerator`]) and a deterministic mock ([`MockProposalGenerator`])
//! for tests.

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
mod error;
mod generator;
mod mock;

// Re-export generated types for downstream use.
pub use baml_client::types::{ProposalContent, TierInput, TierNarrative, ZoneCallout, ZoneSummary};

// Re-export the async client singleton and function struct.
pub use baml_client::async_client::GenerateProposalNarrative;
pub use baml_client::B;

// Re-export trait, implementations, and error types.
pub use claude_cli::ClaudeCliGenerator;
pub use error::ProposalError;
pub use generator::{BamlProposalGenerator, ProposalInput, ProposalNarrativeGenerator};
pub use mock::{MockFailingGenerator, MockProposalGenerator};

/// Convenience alias for BAML errors.
pub type Error = baml::BamlError;
