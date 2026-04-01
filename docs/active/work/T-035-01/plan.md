# T-035-01 Plan: BAML AnalyzePlanView

## Step 1: Create BAML source file

Write `baml_src/analyze.baml` with:
- `SuggestedZone` class (label, zone_type, rationale, approximate_area_sqft)
- `SiteObservation` class (observation)
- `SiteAnalysis` class (features: ClassifiedFeature[], suggested_zones: SuggestedZone[], site_observations: SiteObservation[])
- `AnalyzePlanView` function with `image` parameter (BAML native image type), lot_dimensions, address, classified_features
- Prompt emphasizing spatial reasoning with annotated plan view
- `test powell_market_analysis` block

**Verify:** BAML syntax is valid (no linter available, but follows existing patterns).

## Step 2: Regenerate BAML client

Run `baml-cli generate` from the project root (or `npx @boundaryml/baml generate`).

**Verify:** `baml_client/functions/async_client.rs` contains `AnalyzePlanView` struct. `baml_client/types/classes.rs` contains `SiteAnalysis`, `SuggestedZone`, `SiteObservation`.

## Step 3: Create pt-analyzer crate scaffolding

Create `crates/pt-analyzer/`:
- `Cargo.toml` with workspace deps
- `src/lib.rs` with baml_client include and module declarations
- `src/error.rs` with `AnalyzerError` enum
- `src/analyzer.rs` with `SiteAnalyzer` trait + `BamlSiteAnalyzer`
- `src/mock.rs` with `MockSiteAnalyzer` + `MockFailingAnalyzer` + fixture
- `src/claude_cli.rs` with `ClaudeCliAnalyzer`

**Verify:** `cargo check -p pt-analyzer` passes.

## Step 4: Add pt-analyzer dependency to scenarios

Add `pt-analyzer = { path = "../../crates/pt-analyzer" }` to `tests/scenarios/Cargo.toml`.

**Verify:** `cargo check -p pt-scenarios` passes.

## Step 5: Run quality gate

Run `just check` (fmt + lint + test + scenarios).

Fix any clippy warnings, formatting issues, or test failures.

**Verify:** All four gates pass.

## Testing Strategy

- **Mock correctness:** The mock fixture returns a deterministic `SiteAnalysis` with 4 zones, 3 observations, and 2 features. A unit test in `pt-analyzer` verifies the fixture shape.
- **No real LLM calls:** All tests use `MockSiteAnalyzer`. The `BamlSiteAnalyzer` and `ClaudeCliAnalyzer` are tested only via the shared trait interface with the mock.
- **Arithmetic independence:** SuggestedZone area values in the fixture are hand-computed (not derived from system under test).
- **Error path:** `MockFailingAnalyzer` exists for downstream error-handling tests.

## Commit Strategy

1. After Step 2: commit BAML source + regenerated client
2. After Step 3: commit pt-analyzer crate
3. After Step 5: commit any fixes from quality gate
