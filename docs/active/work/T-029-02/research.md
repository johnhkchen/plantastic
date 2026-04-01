# T-029-02 Research: Proposal Mock Trait

## Current State

### pt-proposal crate (`crates/pt-proposal/`)
- Minimal crate that re-exports BAML-generated types and client
- Re-exports: `ProposalContent`, `TierInput`, `TierNarrative`, `ZoneCallout`, `ZoneSummary`
- Re-exports: `GenerateProposalNarrative`, `B` (async client singleton)
- `Error` type alias for `baml::BamlError`
- Dependencies: `baml`, `serde`, `tokio`
- No trait abstraction, no mock, no `ProposalInput` bundle type

### BAML-generated types (`baml_client/types/classes.rs`)
- `ProposalContent` — `intro_paragraph`, `tier_narratives: Vec<TierNarrative>`, `zone_callouts: Vec<ZoneCallout>`, `closing_paragraph`
- `TierInput` — `tier_level: String`, `total: String`, `zones: Vec<ZoneSummary>`
- `TierNarrative` — `tier_level`, `headline`, `description`, `differentiators: Vec<String>`
- `ZoneCallout` — `zone_label`, `note`
- `ZoneSummary` — `label`, `zone_type`, `area_sqft: f64`, `materials: Vec<String>`
- All derive `Debug, Clone, Default, BamlEncode, BamlDecode` (no `serde` derives)

### BAML client function signature
```rust
GenerateProposalNarrative(
    company_name: impl AsRef<str>,
    project_name: impl AsRef<str>,
    project_address: impl AsRef<str>,
    tiers: &[TierInput]
) -> (stream_types::ProposalContent, types::ProposalContent)
```
Called via: `B.GenerateProposalNarrative.call(company, project, address, &tiers).await`

### AppState (`crates/plantastic-api/src/state.rs`)
```rust
pub struct AppState {
    pub pool: PgPool,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_bucket: String,
    pub scan_jobs: Arc<ScanJobTracker>,
}
```
- Uses `#[derive(Debug, Clone)]`
- No trait objects currently; adding `Arc<dyn Trait>` means `Debug` must be manually implemented or the trait must require `Debug`

### API route pattern (`crates/plantastic-api/src/routes/`)
- Routes use `State(state): State<AppState>` extractor
- No proposal routes exist yet
- Route assembly via `.merge()` in `routes/mod.rs`
- `plantastic-api` does NOT depend on `pt-proposal`

### Error pattern
- Domain crates define their own error enums with `thiserror`
- `AppError` in plantastic-api has `From` impls for each domain error
- Variants: `NotFound`, `BadRequest(String)`, `Conflict(String)`, `Internal(String)`

### Test infrastructure
- `pt-test-utils`: `timed()` (10s timeout), `run_with_timeout(duration, closure)`
- Scenario harness in `tests/scenarios/` — returns `ScenarioOutcome`
- `api_helpers.rs`: `scenario_pool()`, `setup_db()`, `router(pool)`, `api_call()`
- No proposal-related scenarios exist

### Dependency injection precedent
- `Arc<ScanJobTracker>` in AppState — concrete type behind Arc, not trait object
- No existing trait object DI pattern in AppState
- The codebase doesn't use `async-trait` crate (not in workspace deps)
- Rust 1.75+ supports `async fn` in traits natively, but `dyn Trait` with async fns requires `async-trait` or manual boxing

## Key Constraints

1. **BAML types lack serde derives** — `ProposalContent` has `BamlEncode`/`BamlDecode`, not `Serialize`/`Deserialize`. The mock needs to construct `ProposalContent` directly (no JSON round-trip).

2. **`async fn` in `dyn Trait`** — Rust 1.75 stabilized async-fn-in-traits but NOT for dynamic dispatch (`dyn Trait`). For `Arc<dyn ProposalNarrativeGenerator>`, we either need the `async-trait` crate or use manual `-> Pin<Box<dyn Future>>` return types.

3. **`Debug` on AppState** — Adding `Arc<dyn ProposalNarrativeGenerator>` means `AppState` can't derive `Debug` unless the trait requires `Debug` or we impl `Debug` manually.

4. **No proposal route yet** — The acceptance criteria mention "API route accepts the generator via AppState" but no specific route is defined. This ticket creates the trait + DI plumbing; the route handler itself may be in a follow-up.

## Files That Will Change

- `crates/pt-proposal/Cargo.toml` — add `async-trait`, `thiserror`
- `crates/pt-proposal/src/lib.rs` — add trait, ProposalInput, ProposalError, BamlProposalGenerator, MockProposalGenerator
- `crates/plantastic-api/Cargo.toml` — add `pt-proposal` dependency
- `crates/plantastic-api/src/state.rs` — add generator field to AppState
- `crates/plantastic-api/src/error.rs` — add `From<ProposalError>` for AppError
- `crates/plantastic-api/src/lib.rs` or `main.rs` — construct BamlProposalGenerator in prod
- `tests/scenarios/Cargo.toml` — add `pt-proposal` dependency
- `tests/scenarios/src/suites/` — potentially add proposal scenario
- `tests/scenarios/src/progress.rs` — claim milestone
