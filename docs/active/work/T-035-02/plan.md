# T-035-02 Plan: Reconcile Site Data

## Step 1: Create `baml_src/reconcile.baml`

Define all BAML types and the ReconcileSiteData function with prompt. Add the
powell_market_reconciliation test block.

**Verify:** `npx @boundaryml/baml-cli generate` succeeds and new types appear in
`baml_client/types/classes.rs`.

## Step 2: Create `crates/pt-reconciler/` scaffold

Create Cargo.toml, src/lib.rs, src/error.rs. Add to workspace members in root Cargo.toml.

**Verify:** `cargo check -p pt-reconciler` compiles (even if empty trait).

## Step 3: Implement core modules

In order:
1. `src/error.rs` — ReconcilerError enum
2. `src/convert.rs` — summarize_baseline function
3. `src/reconciler.rs` — ReconcilerInput struct, SiteReconciler trait, BamlSiteReconciler
4. `src/claude_cli.rs` — ClaudeCliReconciler
5. `src/mock.rs` — MockSiteReconciler, MockFailingReconciler, powell_market_fixture
6. `src/lib.rs` — module declarations and re-exports

**Verify:** `cargo check -p pt-reconciler` compiles clean.

## Step 4: Run quality gate

Run `just check` (fmt + lint + test + scenarios).

Fix any clippy warnings, formatting issues, or test failures.

**Verify:** `just check` passes with zero warnings.

## Step 5: Write progress.md

Document what was completed, any deviations from plan.

## Testing Strategy

- **Unit tests in convert.rs:** Verify summarize_baseline correctly computes avg_sun_hours
  and maps trees. Use hand-constructed ProjectBaseline, assert independently computed
  expected values.

- **Mock fixture shape test:** Verify powell_market_fixture returns expected counts
  (2 confirmed, 1 scan-only, 1 satellite-only, 1 discrepancy, 4 zones).

- **Trait mock test:** Call MockSiteReconciler.reconcile() through trait interface,
  verify output shape.

- **Error path test:** Call MockFailingReconciler.reconcile(), verify error variant.

- **No real LLM calls in tests** (per CLAUDE.md: mock BAML/LLM in tests).

## Commit Strategy

- Single atomic commit after Step 4 passes (all files are interdependent — the BAML
  types must exist before the crate compiles).
