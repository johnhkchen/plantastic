# T-034-01 Structure: BAML ClassifyFeatures

## Files to Create

### BAML Schema
- `baml_src/classify.baml` — ClassifyFeatures function, FeatureCandidateInput/ClassifiedFeature types, test case

### pt-features Crate
```
crates/pt-features/
├── Cargo.toml
└── src/
    ├── lib.rs             # Module declarations, re-exports, baml_client include
    ├── classifier.rs      # FeatureClassifier trait + BamlFeatureClassifier
    ├── claude_cli.rs      # ClaudeCliClassifier (subscription dev)
    ├── mock.rs            # MockFeatureClassifier (CI/test)
    └── error.rs           # ClassificationError enum
```

### Test Fixture
- `crates/pt-features/tests/fixtures/powell_market_candidates.json` — serialized FeatureCandidate[] from real scan (captured once, committed)

### Integration Tests
- `crates/pt-features/tests/classify.rs` — mock classifier tests + Powell & Market pipeline test

## Files to Modify

### BAML Generated Code
- `baml_client/` — regenerated after adding `classify.baml` (adds ClassifyFeatures function + types)

### Workspace Config
- `Cargo.toml` — no change needed (`crates/*` glob already includes new crate)

## Module Boundaries

### `baml_src/classify.baml`

Defines:
- `class FeatureCandidateInput` — mirrors pt-scan's FeatureCandidate for BAML
- `class ClassifiedFeature` — LLM output schema
- `function ClassifyFeatures(candidates: FeatureCandidateInput[], address: string, climate_zone: string) -> ClassifiedFeature[]`
- `test powell_market_features` — canned test data

### `crates/pt-features/src/lib.rs`

```rust
// Include generated BAML client (same pattern as pt-proposal)
#[allow(clippy::all, ...)]
#[path = "../../../baml_client/mod.rs"]
mod baml_client;

pub mod claude_cli;
mod classifier;
mod error;
mod mock;

// Re-export generated types
pub use baml_client::types::ClassifiedFeature;
pub use baml_client::B;

// Re-export trait + implementations
pub use classifier::{BamlFeatureClassifier, FeatureClassifier};
pub use claude_cli::ClaudeCliClassifier;
pub use error::ClassificationError;
pub use mock::MockFeatureClassifier;
```

### `crates/pt-features/src/classifier.rs`

Public interface:
```rust
#[async_trait]
pub trait FeatureClassifier: Send + Sync {
    async fn classify(
        &self,
        candidates: &[pt_scan::FeatureCandidate],
        address: &str,
        climate_zone: &str,
    ) -> Result<Vec<ClassifiedFeature>, ClassificationError>;
}

pub struct BamlFeatureClassifier;
// impl FeatureClassifier: converts candidates → BAML types, calls B.ClassifyFeatures.call()
```

Internal: `fn to_baml_input(c: &pt_scan::FeatureCandidate) -> FeatureCandidateInput` conversion

### `crates/pt-features/src/mock.rs`

```rust
pub struct MockFeatureClassifier;
// impl FeatureClassifier: deterministic classification from geometry heuristics
```

Rules:
- tall + columnar/spreading + brown/green → tree (high confidence)
- tall + columnar + gray → utility pole
- short + flat → hardscape
- fallback → structure (moderate confidence)

### `crates/pt-features/src/claude_cli.rs`

```rust
pub struct ClaudeCliClassifier;
// impl FeatureClassifier: builds prompt, calls `claude` CLI, parses with BAML
```

Same subprocess pattern as pt-proposal's `ClaudeCliGenerator`.

### `crates/pt-features/src/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClassificationError {
    #[error("LLM classification failed: {0}")]
    Classification(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
```

### `crates/pt-features/Cargo.toml`

Dependencies:
- `async-trait.workspace = true`
- `baml.workspace = true`
- `serde.workspace = true`
- `serde_json.workspace = true`
- `thiserror.workspace = true`
- `tokio.workspace = true`
- `pt-scan = { path = "../pt-scan" }` (for FeatureCandidate type)

Dev dependencies:
- `pt-test-utils = { path = "../pt-test-utils" }`

## Ordering

1. Create `baml_src/classify.baml`
2. Run `baml-cli generate` to regenerate `baml_client/`
3. Create `crates/pt-features/` crate structure
4. Implement error → mock → classifier → claude_cli → lib.rs
5. Create test fixture (serialize Powell & Market candidates to JSON)
6. Write integration tests
7. Run `just check`
