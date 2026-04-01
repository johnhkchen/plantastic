# T-035-01 Structure: BAML AnalyzePlanView

## New Files

### `baml_src/analyze.baml`
BAML function definition with types:
- `SuggestedZone` class: label, zone_type, rationale, approximate_area_sqft
- `SiteObservation` class: observation string
- `SiteAnalysis` class: features (ClassifiedFeature[]), suggested_zones (SuggestedZone[]), site_observations (SiteObservation[])
- `AnalyzePlanView` function: image, lot_dimensions, address, classified_features → SiteAnalysis
- `test powell_market_analysis` block with fixture args

### `crates/pt-analyzer/Cargo.toml`
Workspace member. Dependencies: async-trait, baml, base64, serde, serde_json, thiserror, tokio. Path deps: pt-scan. Dev-deps: pt-test-utils.

### `crates/pt-analyzer/src/lib.rs`
Module root. Includes `baml_client` via `#[path]` (same pattern as pt-features/pt-planter). Declares modules: analyzer, claude_cli, error, mock. Re-exports: generated types (SiteAnalysis, SuggestedZone, SiteObservation, ClassifiedFeature), B singleton, AnalyzePlanView function struct, trait, implementations, errors.

### `crates/pt-analyzer/src/analyzer.rs`
- `SiteAnalyzerInput` struct: plan_view_png (Vec<u8>), lot_dimensions (String), address (String), classified_features (Vec<ClassifiedFeature>)
- `SiteAnalyzer` trait with `async fn analyze(&self, input: &SiteAnalyzerInput) -> Result<SiteAnalysis, AnalyzerError>`
- `BamlSiteAnalyzer` struct: converts PNG bytes to `baml::Image::from_base64()`, calls `B.AnalyzePlanView.call()`

### `crates/pt-analyzer/src/claude_cli.rs`
- `ClaudeCliAnalyzer` struct
- `build_prompt()`: formats classified features as JSON, includes lot dimensions and address. Note: image cannot be passed via CLI text prompt, so this implementation base64-encodes and instructs Claude to analyze the described features without the image, or includes the base64 inline (limited utility — CLI fallback is for dev convenience, not production).
- `call_cli()`: runs `claude -p ... --output-format text`
- `PlanViewAnalyzer` trait impl: prompt → CLI → parse with `B.AnalyzePlanView.parse()`

### `crates/pt-analyzer/src/mock.rs`
- `MockSiteAnalyzer` struct: returns `powell_market_fixture()`
- `MockFailingAnalyzer` struct: returns `AnalyzerError::Analysis("mock LLM failure")`
- `powell_market_fixture()` function: hand-crafted SiteAnalysis with 4 suggested zones, 3 site observations, and 2 classified features echoed back

### `crates/pt-analyzer/src/error.rs`
- `AnalyzerError` enum with `Analysis(String)` and `InvalidInput(String)` variants (matches pt-features/pt-planter pattern)

## Modified Files

### `baml_client/` (auto-generated)
After `baml-cli generate`:
- `functions/async_client.rs`: new `AnalyzePlanView` function struct, added to `BamlAsyncClient`
- `types/classes.rs`: new `SiteAnalysis`, `SuggestedZone`, `SiteObservation` structs
- `types/mod.rs`: new variants in `Types` enum
- `stream_types/`: corresponding streaming types

### `tests/scenarios/Cargo.toml`
Add `pt-analyzer` path dependency.

### `tests/scenarios/src/suites/design.rs`
No scenario array change. Add a comment noting T-035-01 delivered pt-analyzer capability, referencing future scenario integration.

## Module Boundaries

```
baml_src/analyze.baml
    ↓ (baml-cli generate)
baml_client/ (generated types + function struct)
    ↓ (#[path] include)
crates/pt-analyzer/
    ├── SiteAnalyzer trait (public interface)
    ├── BamlSiteAnalyzer (real LLM via baml_client::B)
    ├── ClaudeCliAnalyzer (subscription dev via claude CLI)
    └── MockSiteAnalyzer (deterministic fixture)
```

Downstream consumers (API, scenarios) depend on `pt-analyzer` and use the trait.

## Ordering

1. Write `baml_src/analyze.baml` (defines types + function)
2. Run `baml-cli generate` (regenerates baml_client/)
3. Create `crates/pt-analyzer/` (all source files)
4. Add pt-analyzer to `tests/scenarios/Cargo.toml`
5. Run `just check`
