# T-029-02 Plan: Proposal Mock Trait

## Step 1: Add workspace dependency

- Add `async-trait = "0.1"` to `[workspace.dependencies]` in root `Cargo.toml`
- Verify: `cargo check -p pt-proposal` still compiles

## Step 2: Create pt-proposal error module

- Create `crates/pt-proposal/src/error.rs` with `ProposalError` enum
- Add `thiserror.workspace = true` to `crates/pt-proposal/Cargo.toml`
- Wire into `lib.rs`: `mod error; pub use error::ProposalError;`
- Verify: `cargo check -p pt-proposal`

## Step 3: Create generator module with trait and BamlProposalGenerator

- Add `async-trait.workspace = true` to `crates/pt-proposal/Cargo.toml`
- Create `crates/pt-proposal/src/generator.rs`:
  - `ProposalInput` struct
  - `ProposalNarrativeGenerator` trait with `#[async_trait]`
  - `BamlProposalGenerator` struct + impl
- Wire into `lib.rs`: `mod generator; pub use generator::*;`
- Verify: `cargo check -p pt-proposal`

## Step 4: Create mock module

- Create `crates/pt-proposal/src/mock.rs`:
  - `MockProposalGenerator` — deterministic narratives referencing input fields
  - `MockFailingGenerator` — always returns error
- Wire into `lib.rs`: `mod mock; pub use mock::*;`
- Verify: `cargo check -p pt-proposal`

## Step 5: Write pt-proposal tests

- Create `crates/pt-proposal/tests/generator_test.rs`:
  - `mock_returns_realistic_content` — verify intro references company/project
  - `mock_returns_tier_narratives_per_tier` — one narrative per input tier
  - `mock_returns_zone_callouts` — one callout per zone
  - `mock_is_deterministic` — two calls with same input produce identical output
  - `failing_mock_returns_error` — MockFailingGenerator returns Generation error
- Verify: `cargo test -p pt-proposal`

## Step 6: Integrate into AppState

- Add `pt-proposal` and `async-trait` deps to `crates/plantastic-api/Cargo.toml`
- Modify `state.rs`: add `proposal_generator: Arc<dyn ProposalNarrativeGenerator>` field, manual Debug impl
- Modify `error.rs`: add `From<ProposalError> for AppError`
- Modify `main.rs`: construct `BamlProposalGenerator` in AppState
- Update any test helpers that construct AppState to include `MockProposalGenerator`
- Verify: `cargo check -p plantastic-api`

## Step 7: Update scenario infrastructure

- Add `pt-proposal` dep to `tests/scenarios/Cargo.toml`
- Update `api_helpers.rs` router construction to include mock generator
- Add milestone to `progress.rs`
- Verify: `cargo run -p pt-scenarios`

## Step 8: Quality gate

- `just fmt`
- `just lint`
- `just test`
- `just scenarios`

## Testing Strategy

- **Unit tests** (Step 5): mock behavior, determinism, error paths
- **Compile-time verification** (Steps 2-4): trait + impls compile correctly
- **Integration** (Step 6): AppState construction with trait object compiles
- **No LLM calls**: BamlProposalGenerator is tested only for construction; actual LLM calls are smoke-test-only (documented, not automated)
