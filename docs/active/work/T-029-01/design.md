# Design: T-029-01 BAML Client Integration

## Decision: Create `pt-proposal` crate that hosts `baml_client/` as a submodule

### Options Considered

#### Option A: Include baml_client/ as a module in plantastic-api
- **Pro:** No new crate, minimal changes.
- **Con:** Couples generated LLM code to the API binary. pt-quote or future crates can't import proposal types without depending on the API crate. Violates the project's domain-crate-per-concern pattern.
- **Rejected:** Creates an architectural bottleneck.

#### Option B: Create a standalone `baml-client` workspace member at root
- **Pro:** baml_client/ stays where the generator puts it.
- **Con:** Need a Cargo.toml at workspace root next to baml_client/, which conflicts with the workspace Cargo.toml. Would require restructuring generator output. Naming (`baml-client`) doesn't match `pt-*` convention.
- **Rejected:** Naming/structural mismatch.

#### Option C: Create `pt-proposal` crate in `crates/`, symlink or move baml_client/ into it
- **Pro:** Follows existing `pt-*` pattern. T-029-02 needs pt-proposal anyway (trait + mock). Domain types live in a domain crate. Other crates can depend on it.
- **Con:** Must either change generator output_dir or use `#[path]` to reference baml_client/ at workspace root.
- **Selected.**

### Approach: `#[path]` attribute for generated code

Rather than moving baml_client/ (which would require changing generators.baml output_dir and re-running baml-cli), pt-proposal will reference it in place using:

```rust
// crates/pt-proposal/src/lib.rs
#[path = "../../../baml_client"]
mod baml_client;
```

This way:
- Generator output stays at workspace root (no baml-cli config changes needed).
- `baml-cli generate` continues to work without path adjustments.
- pt-proposal re-exports the types that other crates need.

### Dependency: `baml` crate

The generated code imports from the `baml` crate. We need:
- Add `baml = "0.218.0"` to `[workspace.dependencies]`.
- Add `baml.workspace = true` to pt-proposal's Cargo.toml.
- Also need `tokio` (already in workspace) and `serde` (already in workspace).

### Lint Handling

Generated code has `#![allow(non_snake_case, ...)]` at module level. Since this is an inner attribute in a submodule (not the crate root), it should work. However, workspace-level clippy lints at `deny` could override. If needed, we'll add targeted `#[allow(...)]` or configure clippy to skip generated files.

The practical approach: add `#![allow(clippy::all)]` to the `mod baml_client` declaration or use a wrapper module with appropriate allows.

### Re-exports from pt-proposal

pt-proposal will re-export:
- Types: `ProposalContent`, `TierNarrative`, `ZoneCallout`, `TierInput`, `ZoneSummary`
- Client: `B` (the async client singleton) — though T-029-02 will wrap this behind a trait
- Error: `baml::BamlError` as a convenience re-export

### What This Ticket Does NOT Do

- No trait abstraction (T-029-02)
- No mock implementation (T-029-02)
- No API route changes (T-029-02)
- No actual LLM calls in tests

### Verification

Acceptance is met when:
1. `pt-proposal` crate exists in `crates/` and is a workspace member (automatic via `crates/*` glob).
2. `baml` dependency resolves and compiles.
3. Generated types are importable: `use pt_proposal::types::ProposalContent`.
4. `GenerateProposalNarrative` is callable (compiles, not runtime-tested).
5. `just check` passes — no lint errors, no test regressions.
