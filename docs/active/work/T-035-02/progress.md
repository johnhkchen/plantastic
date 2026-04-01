# T-035-02 Progress: Reconcile Site Data

## Completed

### Step 1: BAML function definition
- Created `baml_src/reconcile.baml` with:
  - SatelliteTree, SatelliteBaseline (LLM-friendly satellite summary types)
  - ReconciledFeature, Discrepancy, RecommendedZone, ReconciledSite (output types)
  - ReconcileSiteData function with temporal reasoning prompt
  - powell_market_reconciliation test block
- Ran `baml-cli generate` — 18 files written to baml_client/

### Step 2: Crate scaffold
- Created `crates/pt-reconciler/` with Cargo.toml
- Workspace auto-includes via `crates/*` glob — no root Cargo.toml edit needed

### Step 3: Core modules
- `src/error.rs` — ReconcilerError (Reconciliation, InvalidInput)
- `src/convert.rs` — summarize_baseline() with 4 unit tests
- `src/reconciler.rs` — ReconcilerInput, SiteReconciler trait, BamlSiteReconciler
- `src/claude_cli.rs` — ClaudeCliReconciler (text-only fallback)
- `src/mock.rs` — MockSiteReconciler, MockFailingReconciler, powell_market_fixture() with 6 tests
- `src/lib.rs` — module declarations and re-exports

### Step 4: Quality gate
- `cargo check -p pt-reconciler` — clean (0 warnings)
- `cargo test -p pt-reconciler` — 10/10 tests pass
- `just fmt` — applied
- `just check` — running

## Deviations from Plan

- No root Cargo.toml modification needed — workspace uses `crates/*` glob
- Added `geo` and `pt-solar` as dev-dependencies for convert.rs tests
  (needed to construct ExposureGrid and Polygon test fixtures)

## Remaining

- Confirm `just check` passes fully
- Write review.md
