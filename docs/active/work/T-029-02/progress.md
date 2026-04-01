# T-029-02 Progress: Proposal Mock Trait

## Completed

### Step 1: Workspace dependency
- Added `async-trait = "0.1"` to workspace `Cargo.toml`
- Added `async-trait`, `thiserror` to `pt-proposal/Cargo.toml`

### Step 2: Error module
- Created `crates/pt-proposal/src/error.rs` with `ProposalError` enum (Generation, InvalidInput)

### Step 3: Generator module
- Created `crates/pt-proposal/src/generator.rs`
- `ProposalInput` struct bundles company_name, project_name, project_address, tiers
- `ProposalNarrativeGenerator` trait with `#[async_trait]`, `Send + Sync` bounds
- `BamlProposalGenerator` calls `B.GenerateProposalNarrative.call()`, maps errors

### Step 4: Mock module
- Created `crates/pt-proposal/src/mock.rs`
- `MockProposalGenerator` — deterministic narratives referencing input fields
- `MockFailingGenerator` — always returns `ProposalError::Generation`

### Step 5: Tests
- Created `crates/pt-proposal/tests/generator_test.rs` — 6 tests, all passing
- Tests: realistic content, per-tier narratives, zone callouts, determinism, error path, empty input

### Step 6: AppState integration
- Added `pt-proposal` and `async-trait` deps to `plantastic-api/Cargo.toml`
- Modified `state.rs`: added `proposal_generator: Arc<dyn ProposalNarrativeGenerator>`, manual `Debug` impl
- Modified `error.rs`: added `From<ProposalError> for AppError`
- Modified `main.rs`: constructs `BamlProposalGenerator` in production
- Updated `tests/common/mod.rs`: both `test_router_full` and `test_router` use `MockProposalGenerator`

### Step 7: Scenario infrastructure
- Added `pt-proposal` dep to `tests/scenarios/Cargo.toml`
- Updated `api_helpers.rs`: router uses `MockProposalGenerator`
- Added milestone to `progress.rs`: "pt-proposal: trait abstraction + mock generator"

### Step 8: Quality gate
- `cargo fmt` — clean
- `cargo clippy` — no warnings
- `cargo test -p pt-proposal` — 6/6 pass
- `cargo run -p pt-scenarios` — milestone visible, 82.5 min / 240 min (34.4%)

## Deviations from Plan

- A hook auto-generated `crates/pt-proposal/src/claude_cli.rs` with a `ClaudeCliGenerator` implementation that routes LLM calls through the local `claude` CLI. This was not in the original plan but is a valid third implementation of the trait. Kept as-is.

## Remaining

None — all steps complete.
