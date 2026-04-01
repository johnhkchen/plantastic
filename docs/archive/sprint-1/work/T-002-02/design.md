# Design — T-002-02: pt-quote Engine

## Decision: Function-based API with material lookup via slice

### The Options

**Option A: Free function taking `(&[Zone], &Tier, &[Material]) -> Quote`**

The caller passes zones, a tier, and the full material catalog. pt-quote looks up each MaterialId in the slice. Simple, testable, no trait abstractions.

**Option B: Trait-based material resolver `trait MaterialLookup { fn get(&self, id: MaterialId) -> Option<&Material> }`**

Abstracts material lookup behind a trait. Allows HashMap, database-backed, or mock implementations.

**Option C: HashMap-based `&HashMap<MaterialId, Material>`**

Caller passes a pre-built lookup table. O(1) lookups. But forces the caller to construct a HashMap even if they already have a Vec.

### Decision: Option A (free function with slice)

Rationale:
- pt-quote is pure computation. There will never be a database-backed material lookup inside the quote engine — I/O lives at the edges (apps/api).
- A slice is the simplest input. The caller already has `Vec<Material>` from the catalog. Linear scan over a slice is fast for the expected catalog size (<1000 materials).
- No trait overhead. The function is easy to test with builder-constructed materials.
- If performance becomes an issue (very large catalogs), the caller can pre-build a HashMap and we add a parallel function. Not needed now.

Rejected: Option B adds unnecessary abstraction for a pure function. Option C forces allocation.

### Core Types

```rust
pub struct Quote {
    pub tier: TierLevel,
    pub line_items: Vec<LineItem>,
    pub subtotal: Decimal,
    pub tax: Option<Decimal>,
    pub total: Decimal,
}

pub struct LineItem {
    pub zone_id: ZoneId,
    pub zone_label: Option<String>,
    pub material_id: MaterialId,
    pub material_name: String,
    pub quantity: Decimal,
    pub unit: Unit,
    pub unit_price: Decimal,
    pub line_total: Decimal,
}
```

### Quantity Computation

Per the ticket's acceptance criteria and the Unit enum documentation:

| Unit | Quantity formula | Source |
|------|-----------------|--------|
| SqFt | `area_sqft(&zone.geometry)` | pt_geo::area |
| CuYd | `area_sqft(&zone.geometry) * (depth_inches / 12.0) / 27.0` | pt_geo::volume |
| LinearFt | `perimeter_ft(&zone.geometry)` | pt_geo::perimeter |
| Each | `1` | constant |

For CuYd, depth comes from `assignment.overrides.depth_override_inches` if present, else `material.depth_inches`, else error (cu_yd material without depth is a configuration error).

### Precision Strategy

1. Compute quantity as `f64` from geometry functions.
2. Convert quantity to `Decimal` via `Decimal::from_f64_retain()` — this preserves the full f64 mantissa.
3. Multiply `quantity * unit_price` to get `line_total`.
4. Round `line_total` to 2 decimal places (currency rounding).
5. Store rounded `quantity` to a reasonable precision (4 decimal places) for display.
6. `subtotal` = exact sum of all `line_total` values (already rounded).
7. `total` = `subtotal + tax.unwrap_or(Decimal::ZERO)`.

This matches the S.3.1 scenario: `1.9753... cu_yd × $45 = $88.888... → $88.89`.

### Error Handling

Two error conditions:
1. **Material not found** — `MaterialAssignment` references a `MaterialId` not in the catalog.
2. **Missing depth** — `CuYd` material with no `depth_inches` and no override.

Return `Result<Quote, QuoteError>` with a descriptive error enum.

### Function Signature

```rust
pub fn compute_quote(
    zones: &[Zone],
    tier: &Tier,
    materials: &[Material],
    tax: Option<Decimal>,
) -> Result<Quote, QuoteError>;
```

Simple. Caller provides all data. Function returns a complete Quote or an error.

### Scenario Coverage

This ticket must turn S.3.1 and S.3.2 green:
- **S.3.1**: Build a project with known geometry, assign materials, compute quote, verify exact line totals and subtotal.
- **S.3.2**: Same project, three tiers with different materials at different prices, verify Good < Better < Best, subtotal == sum(line_totals), no duplicate zone assignments.

The `pt-scenarios` crate needs `pt-geo`, `pt-project`, `pt-materials`, `pt-quote` added as dependencies to make these tests real.

### What Was Rejected

- **Stateful QuoteEngine struct**: No state to manage. A function is sufficient.
- **Builder pattern for Quote**: Quotes are computed output, not user-constructed. Direct struct construction is fine.
- **Async**: No I/O. Sync is correct.
- **Generic over material lookup**: Premature abstraction. See Option B rejection above.
- **Tax calculation logic**: Tax rate/rules vary by jurisdiction. pt-quote accepts a pre-computed tax amount. Tax computation belongs in the API layer.
