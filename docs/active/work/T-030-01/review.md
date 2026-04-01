# T-030-01 Review — Typst Template & PDF Rendering Pipeline

## Summary

Created the Typst-based PDF rendering pipeline in the existing `pt-proposal` crate. The `render_proposal()` function takes a `ProposalDocument` (3 quotes + narrative + branding) and returns branded PDF bytes. Template and fonts are embedded at compile time — no filesystem access at runtime, Lambda-ready.

## Files Created

| File | Purpose |
|------|---------|
| `crates/pt-proposal/src/render.rs` | Types (`TenantBranding`, `ProposalDocument`), dollar formatting, Typst rendering pipeline |
| `crates/pt-proposal/templates/proposal.typ` | Typst template: header, 3-tier comparison table, narratives, callouts, CTA, footer |
| `crates/pt-proposal/tests/render_test.rs` | Integration tests for PDF output validity and size |
| `docs/active/work/T-030-01/{research,design,structure,plan,progress,review}.md` | RDSPI artifacts |

## Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` (workspace) | Added `typst`, `typst-as-lib`, `typst-pdf` workspace deps |
| `crates/pt-proposal/Cargo.toml` | Added typst deps + pt-materials, pt-quote, pt-project |
| `crates/pt-proposal/src/lib.rs` | Added `mod render;` + re-exports |
| `crates/pt-proposal/src/error.rs` | Added `ProposalError::Render(String)` variant |
| `crates/plantastic-api/src/error.rs` | Added match arm for `Render` variant |

## Public API

```rust
pub struct TenantBranding {
    pub company_name: String,
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

pub struct ProposalDocument {
    pub project_name: String,
    pub project_address: String,
    pub date: String,
    pub branding: TenantBranding,
    pub narrative: ProposalContent,
    pub good_quote: Quote,
    pub better_quote: Quote,
    pub best_quote: Quote,
}

pub fn render_proposal(data: &ProposalDocument) -> Result<Vec<u8>, ProposalError>
```

## Test Coverage

| Test | What it verifies |
|------|-----------------|
| `format_dollars_basic` | $1,530.00 formatting |
| `format_dollars_small` | $8.50 formatting |
| `format_dollars_large` | $1,234,567.89 thousand separators |
| `format_dollars_zero` | $0.00 edge case |
| `render_produces_valid_pdf` | Full pipeline → PDF starts with `%PDF-` |
| `render_pdf_size_reasonable` | PDF is >10KB and <5MB |

All 12 pt-proposal tests pass (6 generator + 4 format + 2 render).

## Scenario Dashboard

Before: S.3.3 = NotImplemented
After: S.3.3 = NotImplemented (API route is T-030-02's scope)
No regressions. Dashboard total unchanged.

## Design Decisions

1. **typst-as-lib** wraps Typst's `World` trait boilerplate. `typst-kit-fonts` + `typst-kit-embed-fonts` features embed default fonts for Lambda (no system font dependency).

2. **ProposalDocument doesn't derive Serialize/Deserialize** because BAML-generated `ProposalContent` lacks serde impls. A private `TemplateData` intermediary handles JSON serialization with pre-formatted dollar strings.

3. **Dollar formatting in Rust** (not Typst) ensures precision from `rust_decimal::Decimal` is preserved with commas and 2 decimal places. Template receives strings like `"$1,530.00"`.

4. **Template receives JSON via `sys.inputs.data`**, parsed in Typst with `json(bytes(inputs.data))`. This avoids needing `derive_typst_intoval` or manual Dict construction.

## Open Concerns

1. **Binary size**: Typst + embedded fonts add significant weight to the Lambda deployment package. Monitor after packaging. If problematic, could use lazy font loading or stripped font subsets.

2. **Template limitations**: The template uses `dedup()` for zone labels which assumes adjacent duplicates. This is correct for our data (line items are grouped by zone) but could silently drop zones if ordering changes.

3. **plantastic-api pre-existing break**: The API crate doesn't compile due to a missing `scenes` module from T-031 (in-progress). This is unrelated to T-030-01 but blocks workspace-wide `just check`. Filed as known issue.

4. **Material photos**: Not included in this iteration (ticket notes say "consider" with photo_ref). Future enhancement for visual richness.

5. **Custom brand fonts**: Using Typst's default fonts. Brand-specific fonts would need embedding via the builder's `.fonts([])` API.

## What T-030-02 Needs

T-030-02 (proposal-api-route) can now call:
```rust
let pdf = pt_proposal::render_proposal(&ProposalDocument { ... })?;
```
to get PDF bytes for the API response. It needs to:
- Load project, compute 3 quotes, generate narrative, build TenantBranding from TenantRow
- Return with `Content-Type: application/pdf`
- Implement S.3.3 scenario test
