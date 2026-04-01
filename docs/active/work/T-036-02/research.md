# T-036-02 Research: BAML Planter Estimation

## What Exists

### BAML Infrastructure
- `baml_src/` contains two functions: `ClassifyFeatures` (classify.baml) and `GenerateProposalNarrative` (proposal.baml)
- `baml_src/clients.baml` defines three LLM clients: ProposalWriter (Sonnet 4), ProposalWriterFast (Haiku 4.5), ProposalFallback (Haiku → Sonnet)
- `baml_src/generators.baml` outputs Rust async client to `baml_client/`
- Auto-generated `baml_client/` provides `B.FunctionName.call()` and `B.FunctionName.parse()` for each function
- Adding a new function to `baml_src/` requires running `baml-cli generate` to regenerate `baml_client/`

### Established Three-Implementation Pattern
Both `pt-features` and `pt-proposal` follow the same pattern:
1. **Trait**: `#[async_trait]` with single method (e.g. `FeatureClassifier::classify()`)
2. **BamlImpl**: Calls `B.FunctionName.call()` with BAML-generated types
3. **ClaudeCliImpl**: Builds prompt manually, calls `claude -p <prompt> --output-format text`, parses with `B.FunctionName.parse()`
4. **MockImpl**: Deterministic output — heuristic-based (pt-features) or fixture-based (pt-proposal)

Each crate:
- Includes `baml_client` via `#[path = "../../../baml_client/mod.rs"] mod baml_client;`
- Re-exports BAML types, async client singleton `B`, and function struct
- Has its own error enum with `thiserror`
- Uses `async-trait`, `baml`, `serde`, `serde_json`, `thiserror`, `tokio` deps

### Gap Measurement (T-036-01)
`crates/pt-scan/src/gap.rs`:
- `Gap` struct: `feature_a_id`, `feature_b_id`, `centroid_distance_ft`, `clear_width_ft`, `clear_length_ft`, `area_sqft`, `ground_elevation_ft`, `midpoint`
- `measure_gaps(candidates, ground_plane, config) -> Vec<Gap>`
- Clear width = centroid_distance - (spread_a/2 + spread_b/2)
- Clear length = min(spread_a, spread_b)
- Area = clear_width * clear_length

### Feature Classification (T-034-01)
`crates/pt-features/`:
- `ClassifiedFeature`: `cluster_id`, `label`, `category`, `species?`, `confidence`, `reasoning`, `landscape_notes`
- Mock uses geometry heuristics (height, spread, color, profile) for deterministic output
- Integration tests validate Powell & Market two-tree scenario

### Quote Engine
`crates/pt-quote/src/engine.rs`:
- `compute_quote(zones, tier, materials, tax) -> Quote`
- Computes quantity per unit type: SqFt, LinearFt, Each, CuYd
- CuYd formula: `area_sqft * depth_ft / (12 * 27)` — needs depth
- Returns `Quote` with `line_items: Vec<LineItem>`, `subtotal`, `tax`, `total`

### Scenario Dashboard
`tests/scenarios/src/suites/quoting.rs`:
- S.3.1–S.3.4 exist for quoting scenarios
- No planting/estimation scenarios yet
- `tests/scenarios/src/suites/design.rs` may be the natural home for planter estimation scenarios

## Key Integration Points

### Input Assembly
The planter estimator needs:
1. **From Gap** (T-036-01): `clear_width_ft`, `clear_length_ft`, `area_sqft`
2. **From ClassifiedFeature** (T-034-01): labels of adjacent features (e.g. "London Plane Tree")
3. **From context**: `sun_hours` (optional), `climate_zone`, `address`

### Output Shape
The LLM produces: plant selections with spacing and style rationale.
Code computes from LLM output:
- `plant_count = floor(area_sqft / (spacing_inches / 12)^2)`
- `soil_volume_cuyd = area_sqft * depth_inches / (12 * 27)`
- `estimated_cost = plant_count * unit_price + soil_volume * soil_price`

### Default Pricing (from ticket)
- $15/gal (grasses)
- $8/4" (annuals)
- $12/gal (succulents)
- $45/cuyd (soil)

### Powell & Market Test Fixture
The gap between two London Plane Tree trunks at Powell & Market Streets, SF:
- Partial shade (building shadow, no canopy in trunk-only scan)
- High foot traffic (Market Street sidewalk)
- SF Parks & Rec style guidelines for Market Street streetscape
- USDA 10b / Sunset 17 climate zone

## Constraints & Assumptions
- No new crate needed — this follows the same pattern as pt-features and pt-proposal
- A new crate `pt-planter` (or extension to pt-features) would be cleanest for the trait + 3 impls
- BAML function goes in a new `planter.baml` file per ticket AC
- The BAML function returns 3 style options; code then computes quantities/costs for each
- Mock should return realistic SF Bay Area plant selections, not random placeholders
- The fixture should be crafted to match the Powell & Market gap scenario
- `baml-cli generate` must run after adding `planter.baml` to regenerate the client

## Open Questions
1. Should this be a new crate `pt-planter` or added to `pt-features`? (Leaning: new crate — different concern)
2. How does this feed into the three-tier quote? (Each style maps to a tier: Good/Better/Best)
3. Do we need a scenario for this? (Yes — planter estimation is a new value-delivering capability)
