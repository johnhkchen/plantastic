# T-035-01 Progress: BAML AnalyzePlanView

## Completed

### Step 1: BAML source file
- Created `baml_src/analyze.baml` with:
  - `SuggestedZone` class (label, zone_type, rationale, approximate_area_sqft)
  - `SiteObservation` class (observation)
  - `SiteAnalysis` class (features: ClassifiedFeature[], suggested_zones: SuggestedZone[], site_observations: SiteObservation[])
  - `AnalyzePlanView` function with `image` parameter (BAML native type)
  - Spatial reasoning prompt referencing annotated plan view
  - `test powell_market_analysis` block with fixture args

### Step 2: BAML client regeneration
- Ran `baml-cli generate` — 18 files generated
- Verified `AnalyzePlanView` struct in `async_client.rs` takes `&types::Image`
- Verified `SiteAnalysis`, `SuggestedZone`, `SiteObservation` in `types/classes.rs`

### Step 3: pt-analyzer crate
- Created `crates/pt-analyzer/` with full three-tier pattern:
  - `Cargo.toml` with workspace deps + base64 for image encoding
  - `src/lib.rs` — baml_client include, re-exports
  - `src/analyzer.rs` — `SiteAnalyzer` trait + `BamlSiteAnalyzer` (base64 encode → Image::from_base64)
  - `src/claude_cli.rs` — `ClaudeCliAnalyzer` (text-only fallback, no image)
  - `src/mock.rs` — `MockSiteAnalyzer` + `MockFailingAnalyzer` + `powell_market_fixture()`
  - `src/error.rs` — `AnalyzerError` enum

### Step 4: Scenarios dependency
- Added `pt-analyzer` to `tests/scenarios/Cargo.toml`

### Step 5: Quality gate
- `cargo fmt --all` — clean
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- `cargo test -p pt-analyzer` — passes (0 tests, compiles clean)
- `cargo run -p pt-scenarios` — 87.5 / 240.0 min, no regressions
- Pre-existing issue fixed: `crates/pt-scan/src/eigenvalue.rs` needless_range_loop and dead_code (surfaced by fmt)

### Deviations from plan
- Image API: `Image::from_base64()` requires runtime pointer — used `baml_client::new_image_from_base64()` helper instead of direct `Image::from_base64()`
- Pre-existing test timeout: `test_powell_market_*` in pt-scan integration tests get SIGKILL'd after 60s — not caused by this ticket's changes
- Fixed pre-existing clippy issues in `eigenvalue.rs` (rule 6: own what you find)

## Remaining
- Review phase (review.md)
