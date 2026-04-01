# T-029-02 Design: Proposal Mock Trait

## Decision: Use `async-trait` crate for dynamic dispatch

### Options Considered

**Option A: `async-trait` crate**
- Add `async-trait` workspace dependency
- `#[async_trait]` on trait and impls
- Compiles to `Pin<Box<dyn Future>>` under the hood
- Well-established pattern, used by axum/tower internally
- Trivial to write and read

**Option B: Manual `-> Pin<Box<dyn Future>>` return type**
- No extra dependency
- Verbose signature: `fn generate(&self, input: ProposalInput) -> Pin<Box<dyn Future<Output = Result<ProposalContent, ProposalError>> + Send + '_>>`
- Error-prone, less readable
- Matches what async-trait generates but with manual boilerplate

**Option C: `trait_variant` crate (experimental)**
- Rust's official approach for Send-able async trait methods
- Still experimental, API not stable
- Would need `#[trait_variant::make(SendProposalNarrativeGenerator: Send)]`
- Overkill for a single trait

**Decision: Option A.** `async-trait` is the idiomatic choice for dyn-dispatchable async traits in Rust 1.75. The acceptance criteria explicitly show `#[async_trait]` syntax. Minimal dep, zero risk.

## Trait Design

```rust
#[async_trait]
pub trait ProposalNarrativeGenerator: Send + Sync {
    async fn generate(&self, input: &ProposalInput) -> Result<ProposalContent, ProposalError>;
}
```

Key decisions:
- **`&ProposalInput` not owned** — the generator doesn't need ownership; avoids cloning when calling multiple generators or retrying
- **`Send + Sync` bounds** — required for `Arc<dyn T>` in async Axum handlers
- **Single method** — matches the BAML function signature; streaming can be added later as a separate method with a default impl

## ProposalInput type

Bundle the four BAML function parameters into a struct:
```rust
pub struct ProposalInput {
    pub company_name: String,
    pub project_name: String,
    pub project_address: String,
    pub tiers: Vec<TierInput>,
}
```
This struct lives in pt-proposal and is the contract between the API layer and the generator. It mirrors the BAML function params.

## ProposalError type

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProposalError {
    #[error("LLM generation failed: {0}")]
    Generation(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
```
Two variants cover the error space:
- `Generation` — LLM call failed (timeout, rate limit, bad response)
- `InvalidInput` — bad input (empty tiers, missing company name)
The mock uses `InvalidInput` for simulated failures; the real impl wraps `baml::BamlError` into `Generation`.

## BamlProposalGenerator (real impl)

```rust
pub struct BamlProposalGenerator;
```
Zero-size struct. Calls `B.GenerateProposalNarrative.call(...)` inside `generate()`. Maps `baml::BamlError` to `ProposalError::Generation`.

## MockProposalGenerator (test impl)

```rust
pub struct MockProposalGenerator;
```
Returns deterministic, realistic narrative text that references the input:
- `intro_paragraph` references `company_name`, `project_name`, `project_address`
- `tier_narratives` — one per tier, headline/description reference tier level and zone labels
- `zone_callouts` — one per zone in first tier, note references zone type and area
- `closing_paragraph` — generic professional sign-off
- Deterministic: `format!()` templates, no randomness

For error simulation: a separate `MockFailingGenerator` struct that always returns `ProposalError::Generation`. This keeps `MockProposalGenerator` simple (no config flags or interior mutability).

## AppState Integration

```rust
pub struct AppState {
    pub pool: PgPool,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_bucket: String,
    pub scan_jobs: Arc<ScanJobTracker>,
    pub proposal_generator: Arc<dyn ProposalNarrativeGenerator>,
}
```

`Debug` — manual impl that prints `"<ProposalNarrativeGenerator>"` for the generator field rather than requiring the trait to extend `Debug`.

In production (`main.rs`): `Arc::new(BamlProposalGenerator)`
In tests: `Arc::new(MockProposalGenerator)`

## What This Ticket Does NOT Do

- No proposal API route (that's a follow-up ticket)
- No streaming support (trait can be extended later)
- No PDF integration (separate S.3.3 path)
- No scenario flip — this is infrastructure, so we claim a milestone

## Milestone

Add a milestone: "pt-proposal: trait abstraction + mock generator" delivered by T-029-02, unlocking future proposal narrative scenarios.
