# Research — T-008-01: Quote API Route

## Ticket Goal

Wire `pt_quote::compute_quote` to `GET /projects/:id/quote/:tier`. The computation
engine exists and passes S.3.1/S.3.2 at OneStar. This ticket makes it network-reachable,
advancing those scenarios toward TwoStar.

## Existing Components

### pt-quote crate (`crates/pt-quote/`)

Pure computation, no I/O. Key function:

```rust
pub fn compute_quote(
    zones: &[Zone],
    tier: &Tier,
    materials: &[Material],
    tax: Option<Decimal>,
) -> Result<Quote, QuoteError>
```

**Inputs needed from the database:**
- `zones: &[Zone]` — project's zone geometries (from PostGIS via pt-repo)
- `tier: &Tier` — tier level + material assignments (from tier_assignments table)
- `materials: &[Material]` — tenant's full material catalog (from materials table)
- `tax: Option<Decimal>` — not stored yet; pass `None` for v1

**Output:** `Quote { tier, line_items, subtotal, tax, total }` — already `Serialize`.

**Errors:** `QuoteError::MaterialNotFound`, `QuoteError::MissingDepth` — both should
map to 400 (bad data in the system, not a server error).

### Repository layer (`crates/pt-repo/`)

All needed queries exist:
- `zone::list_by_project(pool, project_id) -> Vec<ZoneRow>` — returns `Polygon<f64>` geometry
- `tier_assignment::get_by_project_and_tier(pool, project_id, tier) -> Vec<TierAssignmentRow>`
- `material::list_by_tenant(pool, tenant_id) -> Vec<MaterialRow>`
- `project::get_by_id(pool, id) -> ProjectRow` — needed for tenant verification

### Type bridging gap

The repo layer returns `ZoneRow`, `TierAssignmentRow`, `MaterialRow`. The quote engine
expects `Zone`, `Tier`, `Material` (domain types from pt-project/pt-materials). Conversion
is needed:

| Repo type | Domain type | Gap |
|-----------|------------|-----|
| `ZoneRow` → `Zone` | ZoneRow has `id: Uuid`, Zone needs `ZoneId(Uuid)` | Wrap UUID |
| `TierAssignmentRow` → `MaterialAssignment` | TierAssignmentRow has `overrides: Option<Value>`, MaterialAssignment needs `Option<AssignmentOverrides>` | Parse JSONB |
| `MaterialRow` → `Material` | MaterialRow has all fields but as repo types | Direct field mapping |

### API layer (`crates/plantastic-api/`)

**Router pattern** (`routes/mod.rs`): Modules register routes via `pub fn routes() -> Router<AppState>`,
merged in `router()`. Adding a new `quotes` module follows this exact pattern.

**Extractors:** `TenantId` from `X-Tenant-Id` header. `Path<(Uuid, String)>` for nested
resources (see tiers.rs pattern).

**Tenant verification:** `zones::verify_project_tenant(pool, project_id, tenant_id)` is
`pub(crate)` and reusable.

**Tier parsing:** `tiers::parse_tier(s: &str) -> Result<TierLevel, AppError>` exists but
is private. Either make it `pub(crate)` or duplicate the 5-line match (duplication is fine
for such a trivial function).

**Error mapping:** `AppError` already has `From<RepoError>`. Need to add `From<QuoteError>`
or convert inline.

**Cargo.toml:** Already depends on `pt-repo`, `pt-project`, `pt-materials`, `pt-geo`. Does
NOT depend on `pt-quote` — needs adding.

### Scenario impact

- S.3.1 and S.3.2 currently pass at OneStar (pure computation only)
- Adding an API route is the definition of TwoStar: "API exists, no UI"
- The scenario tests themselves won't change (they test the engine directly), but the
  integration rating can advance once the route exists
- A new integration test in `plantastic-api/tests/` will verify the route end-to-end

### Existing test infrastructure

`crates/plantastic-api/tests/common/mod.rs` provides:
- `test_pool()` / `setup_test_db()` — Postgres connection + migrations
- `create_test_tenant()` — test tenant creation
- `test_router()` — full router with real DB
- `send()` — HTTP request helper with tenant header

All integration tests are `#[ignore = "Requires Postgres ..."]`.

## Constraints and Assumptions

1. **No new migrations needed.** All tables exist (projects, zones, materials, tier_assignments).
2. **Tax is None for v1.** No tax rate stored per project yet.
3. **Empty quote for no assignments.** AC requires `$0 total, no line items` when tier
   has no assignments — `compute_quote` already handles this (empty tier → empty quote).
4. **404 for missing project** — handled by `verify_project_tenant`.
5. **400 for invalid tier** — handled by `parse_tier`.
6. **QuoteError maps to 400** — bad catalog data is a data-quality issue, not a 500.

## Files that will be touched

- `crates/plantastic-api/Cargo.toml` — add `pt-quote` dependency
- `crates/plantastic-api/src/routes/mod.rs` — register quotes module
- `crates/plantastic-api/src/routes/quotes.rs` — **new file**: handler + DTOs + conversion
- `crates/plantastic-api/src/routes/tiers.rs` — make `parse_tier` pub(crate)
- `crates/plantastic-api/tests/crud_test.rs` — add quote integration test
- `tests/scenarios/src/progress.rs` — claim milestone for quote API route
