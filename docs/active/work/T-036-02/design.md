# T-036-02 Design: BAML Planter Estimation

## Decision: New `pt-planter` Crate

### Option A: Extend pt-features
- Pro: Less crate proliferation
- Con: Feature classification and planter estimation are different concerns. pt-features classifies *what* exists; pt-planter estimates *what to plant*. Mixing them violates SRP and creates a muddled public API. Also forces pt-features to depend on pt-scan's Gap type.

### Option B: New pt-planter crate (CHOSEN)
- Pro: Clean separation. Trait + 3 impls pattern is self-contained. Dependencies are clear: pt-planter depends on pt-scan (for Gap type) but not on pt-features directly.
- Pro: Follows the established pattern exactly — pt-features, pt-proposal, pt-planter each own one BAML function.
- Con: One more crate. But the workspace already has 14 crates; this is consistent.

### BAML Schema Design

The BAML function `EstimatePlanter` takes gap dimensions + context and returns 3 style options. The LLM's job is to pick appropriate plants and explain why — it does NOT compute quantities or costs.

**Input parameters** (flat, not nested — matches existing BAML patterns):
- `gap_width_ft: float` — clear width of the gap
- `gap_length_ft: float` — perpendicular extent
- `area_sqft: float` — rectangular area approximation
- `adjacent_features: string[]` — labels of features bordering the gap
- `sun_hours: int?` — optional estimated daily sun hours
- `climate_zone: string` — e.g. "USDA 10b / Sunset 17"
- `address: string` — for regional plant palette context

**Output types:**
- `PlanterEstimate` with `styles: PlanterStyle[3]`
- `PlanterStyle`: `style_name`, `description`, `plant_selections[]`, `soil_depth_inches`, `design_rationale`
- `PlantSelection`: `common_name`, `botanical_name`, `spacing_inches`, `why_this_plant`

### Code-Computed Quantities

After the LLM returns styles, Rust code computes for each style:
```
For each PlantSelection in style:
  plant_count = floor(area_sqft / (spacing_inches / 12)^2)

soil_volume_cuyd = area_sqft * soil_depth_inches / (12 * 27)

For each PlantSelection:
  plant_cost = plant_count * unit_price_for_category
soil_cost = soil_volume_cuyd * 45.0

estimated_total = sum(plant_costs) + soil_cost
```

Default pricing by inferred category (from common_name patterns):
- Grasses: $15/gal
- Annuals: $8/4"
- Succulents: $12/gal
- Default: $12/gal (safe middle ground)

The category inference is simple keyword matching on common_name — not worth an LLM call. If a plant name contains "grass", "sedge", "carex" → grass pricing. "Succulent", "sedum", "echeveria" → succulent pricing. Otherwise → default.

### Three Styles Map to Three Tiers

The 3 styles should naturally map to Good/Better/Best:
- Style 1 (Good): Low-maintenance, drought-tolerant, budget-friendly
- Style 2 (Better): Mixed perennials, moderate maintenance, mid-range
- Style 3 (Best): Premium specimen plants, designed impact, highest cost

This mapping is done by the Rust code after estimation, not by the LLM.

### Mock Strategy

The mock returns a realistic fixture for the Powell & Market gap:
- Partial shade, high foot traffic, SF maritime climate
- Style 1: Drought-tolerant groundcover (Dymondia, Carex)
- Style 2: Mixed perennial bed (Salvia, Heuchera, Festuca)
- Style 3: Statement planting (Japanese Maple, Hakonechloa, Mondo Grass)

These are real plants appropriate for SF USDA 10b. The fixture is hand-crafted, not LLM-generated, to ensure determinism and correctness.

### Trait Design

```rust
#[async_trait]
pub trait PlanterEstimator: Send + Sync {
    async fn estimate(
        &self,
        input: &PlanterInput,
    ) -> Result<PlanterEstimate, PlanterError>;
}
```

Where `PlanterInput` wraps the gap dimensions + context into a single struct (same pattern as `ProposalInput` in pt-proposal).

### Rejected Alternatives

**Having the LLM compute quantities**: Violates the core discipline. LLMs hallucinate numbers. Code computes math; LLMs pick plants and explain reasoning.

**Using the pt-quote engine for planter costing**: Premature. pt-quote works on Zone/Material/Tier — planter estimation is pre-zone (we're figuring out what goes in the gap). Once the user approves a style, it converts to a Zone + Materials for pt-quote. That integration is a follow-up ticket.

**Single fixture JSON loaded at compile time**: The proposal mock uses `include_str!`. For planter, a hand-built Rust struct is simpler and more maintainable — the fixture is small (3 styles × 2-3 plants each). No JSON file needed.

### Testing Strategy

1. **Unit tests**: Quantity computation (plant_count, soil_volume, cost) with known inputs
2. **Mock integration**: Run estimate through MockPlanterEstimator, verify output structure
3. **Scenario**: New S.4.1 "Planter estimation from measured gaps" in design.rs suite
