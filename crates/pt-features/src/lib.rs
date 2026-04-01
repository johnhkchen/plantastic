// TODO(T-034-01): Complete feature classification implementation
#![allow(dead_code, unused_imports, unused_variables)]

//! LiDAR feature classification via BAML.
//!
//! This crate provides a trait abstraction ([`FeatureClassifier`]) over
//! LLM-powered feature classification, with a real BAML implementation
//! ([`BamlFeatureClassifier`]), a Claude CLI fallback ([`ClaudeCliClassifier`]),
//! and a deterministic mock ([`MockFeatureClassifier`]) for tests.

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
mod classifier;
mod error;
mod mock;

// Re-export generated types for downstream use.
pub use baml_client::types::{ClassifiedFeature, FeatureCandidateInput};

// Re-export the async client singleton and function struct.
pub use baml_client::async_client::ClassifyFeatures;
pub use baml_client::B;

// Re-export trait, implementations, and error types.
pub use classifier::{BamlFeatureClassifier, FeatureClassifier};
pub use claude_cli::ClaudeCliClassifier;
pub use error::ClassificationError;
pub use mock::{MockFailingClassifier, MockFeatureClassifier};
