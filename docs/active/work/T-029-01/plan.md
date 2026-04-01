# Plan: T-029-01 BAML Client Integration

## Step 1: Add `baml` to workspace dependencies

**File:** `Cargo.toml` (workspace root)

Add `baml = "0.218.0"` to `[workspace.dependencies]`.

**Verify:** `cargo metadata` resolves the dependency.

## Step 2: Create pt-proposal crate scaffold

**Files:**
- `crates/pt-proposal/Cargo.toml`
- `crates/pt-proposal/src/lib.rs`

Cargo.toml follows the same pattern as pt-quote: workspace edition/license/rust-version, workspace deps, `[lints] workspace = true`.

lib.rs includes baml_client via `#[path]`, applies lint suppression on the module, and re-exports public types and the async client.

**Verify:** `cargo check -p pt-proposal` compiles.

## Step 3: Fix compilation issues

Generated code may have issues with:
- Workspace clippy deny rules vs generated patterns
- Missing `serde` derives expected by baml types
- Path resolution for `crate::baml_client::` references inside generated code

Iterate until `cargo check -p pt-proposal` succeeds cleanly.

**Verify:** Zero errors, zero warnings from `cargo check -p pt-proposal`.

## Step 4: Run full quality gate

Run `just check` to verify:
- No formatting issues (`just fmt-check`)
- No lint errors (`just lint`)
- All existing tests pass (`just test`)
- Scenario dashboard has no regressions (`just scenarios`)

**Verify:** All four gates pass.

## Step 5: Run scenario baseline

Run `cargo run -p pt-scenarios` and record output for review.md.

## Testing Strategy

This ticket is purely structural — it makes generated code compile and be importable. There are no new behaviors to test.

- **No new unit tests:** The generated code is BAML's responsibility. Testing that BamlEncode/BamlDecode work correctly is testing BAML, not Plantastic.
- **No integration tests:** Actually calling GenerateProposalNarrative requires an API key and makes real LLM calls. T-029-02 creates the mock boundary.
- **Compile-time verification only:** If `cargo check -p pt-proposal` passes, the types and function are importable and callable.
- **Regression check:** `just test` ensures nothing existing broke.

The first real test of proposal generation will come in T-029-02 with MockProposalGenerator.
