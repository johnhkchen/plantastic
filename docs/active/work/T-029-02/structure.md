# T-029-02 Structure: Proposal Mock Trait

## File Changes

### Modified: `Cargo.toml` (workspace root)
- Add `async-trait = "0.1"` to `[workspace.dependencies]`

### Modified: `crates/pt-proposal/Cargo.toml`
- Add deps: `async-trait.workspace = true`, `thiserror.workspace = true`

### Modified: `crates/pt-proposal/src/lib.rs`
- Keep existing BAML re-exports
- Add new modules: `mod generator;`, `mod mock;`, `mod error;`
- Re-export: `ProposalNarrativeGenerator`, `ProposalInput`, `ProposalError`, `BamlProposalGenerator`, `MockProposalGenerator`, `MockFailingGenerator`

### New: `crates/pt-proposal/src/error.rs`
```
ProposalError enum:
  - Generation(String)
  - InvalidInput(String)
```

### New: `crates/pt-proposal/src/generator.rs`
```
ProposalInput struct:
  - company_name: String
  - project_name: String
  - project_address: String
  - tiers: Vec<TierInput>

ProposalNarrativeGenerator trait:
  - async fn generate(&self, input: &ProposalInput) -> Result<ProposalContent, ProposalError>

BamlProposalGenerator struct (unit):
  - impl ProposalNarrativeGenerator — calls B.GenerateProposalNarrative
```

### New: `crates/pt-proposal/src/mock.rs`
```
MockProposalGenerator struct (unit):
  - impl ProposalNarrativeGenerator — deterministic canned narratives from input
  - Templates reference company_name, project_name, tiers, zones

MockFailingGenerator struct (unit):
  - impl ProposalNarrativeGenerator — always returns ProposalError::Generation
```

### Modified: `crates/plantastic-api/Cargo.toml`
- Add: `pt-proposal = { path = "../pt-proposal" }`
- Add: `async-trait.workspace = true` (needed for trait bound in state)

### Modified: `crates/plantastic-api/src/state.rs`
- Add `proposal_generator: Arc<dyn ProposalNarrativeGenerator>` to AppState
- Manual `Debug` impl (replaces derive) to handle trait object field

### Modified: `crates/plantastic-api/src/error.rs`
- Add `From<pt_proposal::ProposalError> for AppError`

### Modified: `crates/plantastic-api/src/main.rs`
- Construct `Arc::new(BamlProposalGenerator)` and pass to AppState

### Modified: `crates/plantastic-api/src/lib.rs`
- Re-export pt_proposal types if needed for test construction

### Modified: `tests/scenarios/Cargo.toml`
- Add: `pt-proposal = { path = "../../crates/pt-proposal" }`

### Modified: `tests/scenarios/src/progress.rs`
- Add milestone: "pt-proposal: trait abstraction + mock generator"
- `delivered_by: Some("T-029-02")`, unlocks: `["S.3.3"]`

### Modified: `tests/scenarios/src/suites/quoting.rs` or `api_helpers.rs`
- Update `router()` helper to include `proposal_generator: Arc::new(MockProposalGenerator)` in AppState

### New: `crates/pt-proposal/tests/generator_test.rs`
- Test MockProposalGenerator returns expected content for given input
- Test MockFailingGenerator returns error
- Test determinism: same input → same output
- Test BamlProposalGenerator construction (unit only, no LLM call)

## Module Boundary

```
pt-proposal (crate)
├── lib.rs          — re-exports
├── error.rs        — ProposalError
├── generator.rs    — ProposalInput, trait, BamlProposalGenerator
└── mock.rs         — MockProposalGenerator, MockFailingGenerator

plantastic-api (crate)
├── state.rs        — AppState with Arc<dyn ProposalNarrativeGenerator>
├── error.rs        — From<ProposalError>
└── main.rs         — constructs BamlProposalGenerator
```

## Public API of pt-proposal after this ticket

```rust
// Types
pub struct ProposalInput { ... }
pub enum ProposalError { ... }

// Trait
pub trait ProposalNarrativeGenerator: Send + Sync { ... }

// Implementations
pub struct BamlProposalGenerator;
pub struct MockProposalGenerator;
pub struct MockFailingGenerator;

// Re-exports from BAML (unchanged)
pub use ProposalContent, TierInput, TierNarrative, ZoneCallout, ZoneSummary;
pub use GenerateProposalNarrative, B;
pub type Error = baml::BamlError;
```
