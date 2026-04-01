# T-030-01 Progress — Typst Template & PDF Rendering Pipeline

## Completed

### Step 1: Dependencies
- Added `typst`, `typst-as-lib` (with `typst-kit-fonts` + `typst-kit-embed-fonts`), `typst-pdf` to workspace deps
- Added `rust_decimal`, `serde_json`, `typst`, `typst-as-lib`, `typst-pdf`, `pt-materials`, `pt-quote` to pt-proposal Cargo.toml
- Added `pt-project` as dev-dependency for tests

### Step 2: Error variant
- Added `ProposalError::Render(String)` to error.rs
- Updated `plantastic-api/src/error.rs` match arm for exhaustiveness

### Step 3: render.rs types and function
- Created `crates/pt-proposal/src/render.rs` with:
  - `TenantBranding` struct (Serialize, Deserialize)
  - `ProposalDocument` struct (Debug, Clone) — no Serialize/Deserialize since ProposalContent is BAML-generated without serde
  - Private `TemplateData` hierarchy with pre-formatted dollar strings
  - `format_dollars()` with thousand separators
  - `format_unit()` for display strings
  - `render_proposal(&ProposalDocument)` using typst-as-lib with embedded fonts
- Wired into lib.rs with `mod render;` + re-exports

### Step 4: Typst template
- Created `crates/pt-proposal/templates/proposal.typ`
- Parses JSON from `sys.inputs.data`
- Sections: header, project info, intro, 3-tier comparison table, tier narratives, zone callouts, closing + CTA, footer
- Uses embedded fonts via typst-kit

### Step 5: Tests
- Created `crates/pt-proposal/tests/render_test.rs`
- `render_produces_valid_pdf` — verifies %PDF- magic bytes (passes)
- `render_pdf_size_reasonable` — verifies >10KB <5MB (passes)
- 4 unit tests for `format_dollars` in render.rs (passes)

### Step 6: Quality gate
- `cargo fmt` — passes
- `cargo clippy -p pt-proposal --all-targets -- -D warnings` — passes clean
- `cargo test -p pt-proposal` — 12 tests pass (4 format + 6 generator + 2 render)
- Scenario dashboard — no regressions, S.3.3 still NotImplemented (T-030-02 scope)

## Deviations from plan
- Used `&ProposalDocument` (reference) instead of owned value per clippy suggestion
- `ProposalDocument` does not derive `Serialize`/`Deserialize` because `ProposalContent` (BAML-generated) lacks serde impls. Serialization goes through the private `TemplateData` intermediary instead.
- `typst-kit-fonts` + `typst-kit-embed-fonts` features used for font embedding (no manual font file management)

## Known issue (not from this ticket)
- `plantastic-api` doesn't compile due to missing `scenes` module (from T-031 in-progress work). This is pre-existing and unrelated to T-030-01.
