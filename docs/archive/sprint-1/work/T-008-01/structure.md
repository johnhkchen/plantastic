# Structure — T-008-01: Quote API Route

## Files Modified

### `crates/plantastic-api/Cargo.toml`
- Add `pt-quote = { path = "../pt-quote" }` to `[dependencies]`

### `crates/plantastic-api/src/routes/mod.rs`
- Add `pub mod quotes;`
- Add `.merge(quotes::routes())` to the router

### `crates/plantastic-api/src/routes/tiers.rs`
- Change `fn parse_tier` from private to `pub(crate) fn parse_tier`

### `crates/plantastic-api/src/routes/quotes.rs` (NEW)

```
//! Quote computation routes.

routes()           → Router<AppState>
  GET /projects/{id}/quote/{tier}

get_quote()        → async handler
  1. verify_project_tenant
  2. parse_tier
  3. Load zones (zone::list_by_project)
  4. Load assignments (tier_assignment::get_by_project_and_tier)
  5. Load materials (material::list_by_tenant) — needs tenant_id from project row
  6. Convert repo types → domain types
  7. compute_quote(zones, tier, materials, None)
  8. Return Json<Quote>

zone_rows_to_zones(rows: Vec<ZoneRow>) -> Vec<Zone>
  ZoneRow.id (Uuid) → ZoneId(Uuid)
  ZoneRow.geometry → geometry (already Polygon<f64>)
  ZoneRow.zone_type → zone_type (already ZoneType)
  ZoneRow.label → label

material_rows_to_materials(rows: Vec<MaterialRow>) -> Vec<Material>
  All fields map directly; MaterialRow.id (Uuid) → MaterialId(Uuid)

build_tier(level, rows: Vec<TierAssignmentRow>) -> Result<Tier, AppError>
  For each row:
    zone_id: ZoneId(row.zone_id)
    material_id: MaterialId(row.material_id)
    overrides: parse row.overrides (Option<Value>) → Option<AssignmentOverrides>
  Returns Tier { level, assignments }
```

### `crates/plantastic-api/tests/crud_test.rs`
- Add `quote_route_integration` test function
  - Creates project, zones, materials, tier assignments via API
  - Fetches quote via GET /projects/:id/quote/good
  - Asserts correct line items, subtotal, total (independently computed)
  - Tests 404 for missing project
  - Tests 400 for invalid tier
  - Tests empty quote (tier with no assignments)

### `tests/scenarios/src/progress.rs`
- Claim the "Quote API route" milestone or add one if absent
  - Note: this is a NEW milestone since existing milestones don't cover the API route
    specifically for quoting

## Module boundaries

```
plantastic-api
  ├── routes/quotes.rs    ← NEW: handler + conversion functions
  ├── routes/tiers.rs     ← MODIFIED: parse_tier visibility
  └── routes/mod.rs       ← MODIFIED: register quotes

pt-quote (unchanged)      ← compute_quote() called from handler
pt-repo (unchanged)       ← queries called from handler
pt-project (unchanged)    ← Zone, Tier, MaterialAssignment types
pt-materials (unchanged)  ← Material, MaterialId types
```

## Public interface changes

- New route: `GET /projects/{id}/quote/{tier}` → `200 Json<Quote>` | `400` | `404`
- `tiers::parse_tier` changes from private to `pub(crate)` (internal only)

## Ordering constraints

1. `Cargo.toml` dependency must be added before `quotes.rs` can import pt-quote
2. `tiers.rs` visibility change must precede `quotes.rs` (which calls parse_tier)
3. Route registration in `mod.rs` after `quotes.rs` exists
4. Integration test after route is functional
5. Milestone claim after tests pass
