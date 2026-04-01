# T-036-02 Structure: BAML Planter Estimation

## New Files

### baml_src/planter.baml
BAML function and types for planter estimation:
- Types: `PlantSelection`, `PlanterStyle`, `PlanterEstimate`
- Function: `EstimatePlanter` with gap dimensions + context → 3 styles
- Prompt: landscape designer selecting plants for a measured gap
- Test case: `powell_market_gap` with two adjacent tree trunks

### crates/pt-planter/Cargo.toml
New crate with dependencies:
- `async-trait`, `baml`, `serde`, `serde_json`, `thiserror`, `tokio` (workspace)
- `rust_decimal` (workspace) — for cost computation
- `pt-scan` (path) — for `Gap` type reference in input construction
- Dev: `pt-test-utils` (path)

### crates/pt-planter/src/lib.rs
Module root following pt-features/pt-proposal pattern:
- `#[path]` include of `baml_client`
- Modules: `estimator`, `claude_cli`, `mock`, `error`, `compute`
- Re-exports: BAML types (PlanterEstimate, PlanterStyle, PlantSelection), trait, impls, error

### crates/pt-planter/src/estimator.rs
Trait + BAML implementation:
- `PlanterInput` struct: gap_width_ft, gap_length_ft, area_sqft, adjacent_features, sun_hours, climate_zone, address
- `PlanterEstimator` trait: `async fn estimate(&self, input: &PlanterInput) -> Result<PlanterEstimate, PlanterError>`
- `BamlPlanterEstimator`: calls `B.EstimatePlanter.call()`

### crates/pt-planter/src/claude_cli.rs
Claude CLI implementation:
- `ClaudeCliEstimator` struct
- `build_prompt(input)` — manual prompt construction
- `call_cli(prompt)` — subprocess execution with env stripping
- `B.EstimatePlanter.parse()` for response parsing

### crates/pt-planter/src/mock.rs
Mock implementation:
- `MockPlanterEstimator` — returns hand-built Powell & Market fixture
- `MockFailingEstimator` — returns error for error path testing
- Fixture: 3 styles with real SF Bay Area plants, realistic spacing/depth values

### crates/pt-planter/src/error.rs
Error types:
- `PlanterError::Estimation(String)` — LLM failure
- `PlanterError::InvalidInput(String)` — validation failure

### crates/pt-planter/src/compute.rs
Code-computed quantities (the "code does math" module):
- `ComputedPlanting` struct: plant_count, unit_price, plant_cost per plant selection
- `ComputedStyle` struct: plantings vec, soil_volume_cuyd, soil_cost, total_cost
- `compute_style(style: &PlanterStyle, area_sqft: f64) -> ComputedStyle`
- `infer_price_category(common_name: &str) -> Decimal` — keyword matching for pricing
- Constants: `GRASS_PRICE`, `ANNUAL_PRICE`, `SUCCULENT_PRICE`, `DEFAULT_PRICE`, `SOIL_PRICE_PER_CUYD`

## Modified Files

### baml_src/clients.baml
Add a `PlanterDesigner` client (or reuse ProposalFallback — same Haiku→Sonnet strategy).
Decision: reuse `ProposalFallback` — no need for a separate client for the same model pair.

### tests/scenarios/src/suites/design.rs
Add scenario S.4.1: "Planter estimation from measured gaps"
- Creates gap fixture (Powell & Market dimensions)
- Runs MockPlanterEstimator
- Verifies 3 styles returned, each with valid plant selections
- Verifies code-computed quantities match independent arithmetic

### tests/scenarios/src/suites/mod.rs
Register design suite if not already registered.

## File Organization

```
baml_src/
  planter.baml                    # NEW — BAML types + function

crates/pt-planter/
  Cargo.toml                      # NEW
  src/
    lib.rs                        # NEW — module root
    estimator.rs                  # NEW — trait + BamlImpl
    claude_cli.rs                 # NEW — CLI impl
    mock.rs                       # NEW — mock with fixture
    error.rs                      # NEW — error types
    compute.rs                    # NEW — quantity/cost computation

tests/scenarios/src/suites/
  design.rs                       # MODIFIED — add S.4.1
```

## Public API Surface

```rust
// Trait
pub trait PlanterEstimator: Send + Sync
pub struct PlanterInput { ... }

// Implementations
pub struct BamlPlanterEstimator;
pub struct ClaudeCliEstimator;
pub struct MockPlanterEstimator;
pub struct MockFailingEstimator;

// Error
pub enum PlanterError { Estimation(String), InvalidInput(String) }

// Computation
pub struct ComputedPlanting { ... }
pub struct ComputedStyle { ... }
pub fn compute_style(style: &PlanterStyle, area_sqft: f64) -> ComputedStyle

// Re-exported BAML types
pub use PlanterEstimate, PlanterStyle, PlantSelection;
```

## Ordering
1. `planter.baml` → `baml-cli generate` → regenerated `baml_client/`
2. `pt-planter` crate (error → compute → estimator → mock → claude_cli → lib)
3. Tests within pt-planter
4. Scenario S.4.1 in design suite
5. `just check` validation
