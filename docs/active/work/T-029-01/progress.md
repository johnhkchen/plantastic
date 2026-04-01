# Progress: T-029-01 BAML Client Integration

## Completed

### Step 1: Add `baml` to workspace dependencies
- Added `baml = "0.218.0"` to `[workspace.dependencies]` in root Cargo.toml.
- Resolved cleanly — pulled in baml, baml-macros, baml-sys, prost, ureq, and 14 other transitive deps.

### Step 2: Create pt-proposal crate scaffold
- Created `crates/pt-proposal/Cargo.toml` with baml, serde, tokio workspace deps.
- Created `crates/pt-proposal/src/lib.rs` with `#[path]` include of baml_client.
- Re-exports: ProposalContent, TierInput, TierNarrative, ZoneCallout, ZoneSummary, B, GenerateProposalNarrative, Error.

### Step 3: Fix compilation issues
- Initial attempt with `#[path = "../../../baml_client"]` failed — `mod` expects a file, not directory.
- Fixed to `#[path = "../../../baml_client/mod.rs"]` — compiles.
- `cargo fmt` reformatted generated code (trailing whitespace, import grouping, etc.).
- Clippy pedantic lints (`missing_errors_doc`, `needless_pass_by_value`, `redundant_closure_for_method_calls`) fired on generated code despite `#[allow(clippy::all)]` — `clippy::all` doesn't cover pedantic.
- Added `clippy::pedantic` + specific pedantic lints to the allow list.

### Step 4: Quality gate
- `just check` passes: fmt, lint, test, scenarios all green.

### Step 5: Scenario baseline
- Before: 44.5/240.0 min (18.5%), 8 pass, 15/24 milestones.
- After: same (this ticket is structural, no scenario impact expected).

## Deviations from Plan
- Had to format generated code with `cargo fmt` — the generated files weren't formatted to Rust style. This is expected since BAML generates code, not formatted Rust.
- Lint suppression needed `clippy::pedantic` in addition to `clippy::all` because workspace config individually enables pedantic lints at warn/deny level.
