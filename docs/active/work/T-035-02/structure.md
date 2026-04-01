# T-035-02 Structure: Reconcile Site Data

## New Files

### `baml_src/reconcile.baml` (~80 lines)

New BAML types and function:

```
class SatelliteTree { height_ft, spread_ft, confidence }
class SatelliteBaseline { lot_area_sqft, trees: SatelliteTree[], avg_sun_hours }
class ReconciledFeature { label, category, source, confidence, reasoning }
class Discrepancy { description, possible_explanation, design_implication }
class RecommendedZone { label, zone_type, rationale, approximate_area_sqft, sun_exposure_hours?, data_sources }
class ReconciledSite {
  confirmed_features: ReconciledFeature[]
  scan_only_features: ReconciledFeature[]
  satellite_only_features: ReconciledFeature[]
  discrepancies: Discrepancy[]
  recommended_zones: RecommendedZone[]
}

function ReconcileSiteData(
  scan_features: ClassifiedFeature[],
  satellite_baseline: SatelliteBaseline,
  plan_view_analysis: SiteAnalysis,
  address: string,
) -> ReconciledSite

test powell_market_reconciliation { ... }
```

Cross-file references: ClassifiedFeature (classify.baml), SiteAnalysis/SuggestedZone/SiteObservation (analyze.baml).

### `crates/pt-reconciler/Cargo.toml`

Dependencies: async-trait, baml, serde, serde_json, thiserror, tokio, pt-scan (path), pt-satellite (path).
Dev-dependencies: pt-test-utils.
Lints: workspace.

### `crates/pt-reconciler/src/lib.rs`

Module root following pt-analyzer pattern:
- `#[path = "../../../baml_client/mod.rs"] mod baml_client;`
- Modules: reconciler, claude_cli, error, mock, convert
- Re-exports: generated BAML types, trait + impls, error, conversion function

### `crates/pt-reconciler/src/reconciler.rs`

**ReconcilerInput struct:**
```rust
pub struct ReconcilerInput {
    pub scan_features: Vec<ClassifiedFeature>,
    pub satellite_baseline: SatelliteBaseline,
    pub plan_view_analysis: SiteAnalysis,
    pub address: String,
}
```

**SiteReconciler trait:**
```rust
#[async_trait]
pub trait SiteReconciler: Send + Sync {
    async fn reconcile(&self, input: &ReconcilerInput) -> Result<ReconciledSite, ReconcilerError>;
}
```

**BamlSiteReconciler:** Calls `crate::B.ReconcileSiteData.call(...)`.

### `crates/pt-reconciler/src/claude_cli.rs`

ClaudeCliReconciler — text-only fallback. Formats all three data sources as text,
calls `claude` CLI, parses with `crate::B.ReconcileSiteData.parse()`.

### `crates/pt-reconciler/src/mock.rs`

MockSiteReconciler + MockFailingReconciler + `powell_market_fixture()`.

Fixture: 2 confirmed (London Planes), 1 scan-only (fire hydrant), 1 satellite-only
(removed tree), 1 discrepancy (height mismatch), 4 recommended zones with sun data.

### `crates/pt-reconciler/src/convert.rs`

```rust
pub fn summarize_baseline(baseline: &pt_satellite::ProjectBaseline) -> SatelliteBaseline
```

Converts ProjectBaseline → BAML SatelliteBaseline:
- lot_area_sqft from baseline.lot_boundary.area_sqft
- trees from baseline.trees mapped to SatelliteTree
- avg_sun_hours computed as mean of baseline.sun_grid.values

### `crates/pt-reconciler/src/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ReconcilerError {
    #[error("LLM reconciliation failed: {0}")]
    Reconciliation(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
```

## Modified Files

### `Cargo.toml` (workspace root)

Add `"crates/pt-reconciler"` to workspace members.

## Module Boundaries

- **pt-reconciler** depends on pt-satellite types (for convert.rs) and BAML generated types
- **pt-reconciler** does NOT depend on pt-analyzer or pt-features — it takes their output
  types via the shared baml_client types
- The convert module is the only place that bridges pt-satellite → BAML type system
- Downstream consumers construct ReconcilerInput from whatever pipeline they use

## Public Interface

```rust
// Types (from BAML)
pub use ReconciledSite, ReconciledFeature, Discrepancy, RecommendedZone;
pub use SatelliteBaseline, SatelliteTree;

// Trait + impls
pub use SiteReconciler, ReconcilerInput;
pub use BamlSiteReconciler, ClaudeCliReconciler;
pub use MockSiteReconciler, MockFailingReconciler;

// Conversion
pub use summarize_baseline;

// Error
pub use ReconcilerError;
```
