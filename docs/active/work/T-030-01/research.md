# T-030-01 Research — Typst Template & PDF Rendering Pipeline

## Codebase State

### pt-proposal crate (existing)
- `crates/pt-proposal/src/lib.rs` — re-exports BAML types + trait + mocks
- `src/generator.rs` — `ProposalNarrativeGenerator` trait, `ProposalInput`, `BamlProposalGenerator`
- `src/mock.rs` — `MockProposalGenerator` (deterministic), `MockFailingGenerator`
- `src/error.rs` — `ProposalError { Generation, InvalidInput }`
- `Cargo.toml` — depends on async-trait, baml, serde, thiserror, tokio
- No rendering logic, no template files, no `ProposalDocument` struct yet

### BAML-generated types (used by pt-proposal)
- `ProposalContent { intro_paragraph, tier_narratives, zone_callouts, closing_paragraph }`
- `TierNarrative { tier_level, headline, description, differentiators }`
- `ZoneCallout { zone_label, note }`
- `TierInput { tier_level, total, zones: Vec<ZoneSummary> }`
- `ZoneSummary { label, zone_type, area_sqft, materials }`

### pt-quote types
- `Quote { tier: TierLevel, line_items: Vec<LineItem>, subtotal, tax, total }` — all Decimal
- `LineItem { zone_id, zone_label, material_id, material_name, quantity, unit, unit_price, line_total }`
- `TierLevel { Good, Better, Best }` from pt-project
- `Unit { SqFt, CuYd, LinearFt, Each }` from pt-materials

### pt-repo tenant
- `TenantRow { id, name, logo_url, brand_color, contact (JSON), created_at, updated_at }`
- `contact` is `Option<serde_json::Value>` — could hold phone/email

### Scenario S.3.3
- Currently `ScenarioOutcome::NotImplemented` at line 662 of `quoting.rs`
- Worth 10 min savings, targets ★★☆ integration / ★☆☆ polish
- T-030-02 owns the API route and scenario test; T-030-01 owns the rendering

## Typst Ecosystem

### Available crates
- `typst` v0.14.2 — core compiler, requires implementing `World` trait
- `typst-pdf` v0.14.2 — takes compiled `Document` → PDF bytes
- `typst-as-lib` v0.15.4 — wrapper that handles `World` boilerplate
- `typst-bake` v0.1.9 — embeds templates + fonts into binary at build time

### typst-as-lib approach
- Handles font loading, caching, source management
- `TypstWrapperWorld::new(root, template_content)` → implements World
- `typst::compile(&world).output` → `Document`
- `typst_pdf::pdf(&document, &PdfOptions::default())` → `Vec<u8>`
- Data passed via `sys.inputs` (string key-value pairs)
- Template parses JSON: `#let data = json(bytes(sys.inputs.payload))`

### Data flow
1. Caller builds `ProposalDocument` (3 quotes + narrative + branding)
2. Serialize to JSON string
3. Pass as `sys.inputs.data` to Typst world
4. Template parses JSON, layouts sections
5. Compile → Document → PDF bytes

## Constraints

- **Lambda deployment**: Typst compiles in-process, no subprocesses — fits
- **Fonts**: Need to embed or bundle at least one sans-serif font (Typst bundles defaults)
- **Dollar amounts**: Must come from pt-quote Decimals, pre-formatted as strings
- **Template location**: `templates/proposal.typ` inside pt-proposal crate
- **Binary size**: Typst + fonts will add to Lambda package size; monitor

## Patterns from codebase

- Newtype IDs: `ZoneId(Uuid)`, `MaterialId(Uuid)` — need serde for JSON
- `rust_decimal::Decimal` with `serde-with-str` — serializes as strings, good for template
- Error types use `thiserror`
- Test utils: `pt_test_utils::timed()` for 10s timeout
- All crates use workspace lints and edition

## Open questions for Design
1. `typst-as-lib` vs raw `typst` + `typst-pdf`?
2. Where to define `TenantBranding` — pt-proposal or pt-repo?
3. Template: embed as `include_str!` or load from filesystem?
4. Font strategy: Typst built-in fonts vs custom brand fonts?
5. How to format Decimal → display strings (in Rust before passing, or in Typst)?
