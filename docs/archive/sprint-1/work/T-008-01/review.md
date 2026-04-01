# Review — T-008-01: Quote API Route

## Summary

Added `GET /projects/{id}/quote/{tier}` to the Plantastic API. The route loads zone
geometry from PostGIS, tier assignments, and the tenant's material catalog; converts
repo types to pt-quote domain types; calls `compute_quote()`; and returns the Quote
as JSON. Handles 404 (missing project), 400 (invalid tier, bad catalog data), and
empty quotes (no assignments → $0 total).

## Files changed

| File | Action | Lines |
|------|--------|-------|
| `crates/plantastic-api/Cargo.toml` | Modified | +1 (pt-quote dep) |
| `crates/plantastic-api/src/routes/mod.rs` | Modified | +2 (module + merge) |
| `crates/plantastic-api/src/routes/tiers.rs` | Modified | +1 word (pub(crate)) |
| `crates/plantastic-api/src/routes/quotes.rs` | **Created** | ~90 lines |
| `crates/plantastic-api/tests/crud_test.rs` | Modified | +100 lines (integration test) |
| `tests/scenarios/src/progress.rs` | Modified | +12 lines (milestone) |

Total: 1 new file, 5 modified files.

## Test coverage

### New test

- `quote_route_integration` — full HTTP round-trip integration test (requires Postgres):
  1. Creates project with 12×15 ft patio zone, Travertine Pavers at $8.50/sq_ft, good tier assignment
  2. Asserts `GET /projects/:id/quote/good` returns 200 with 1 line item, `$1,530.00` total
     (independently computed: 12 × 15 = 180 sq_ft × $8.50 = $1,530.00)
  3. Asserts `GET /projects/:id/quote/better` returns 200 with empty line items, `$0` total
  4. Asserts `GET /projects/:id/quote/premium` returns 400 (invalid tier)
  5. Asserts `GET /projects/{random-uuid}/quote/good` returns 404 (missing project)

### Existing test coverage (unchanged, still passing)

- 12 unit tests in pt-quote covering all material units, overrides, edge cases, error paths
- 8 integration tests in plantastic-api covering CRUD for projects, zones, materials, tiers
- S.3.1 and S.3.2 scenario tests passing at OneStar

### Coverage gaps

- The integration test is `#[ignore]` and requires Postgres — cannot verify without DB
- No test for `QuoteError` paths via HTTP (requires inserting a tier assignment that
  references a material from a different tenant, or a cu_yd material without depth).
  These are data-integrity issues unlikely in normal use, and the unit tests in pt-quote
  cover the engine-level error paths.

## Scenario dashboard — before and after

| Metric | Before | After |
|--------|--------|-------|
| Effective savings | 20.0 min / 240.0 min (8.3%) | 20.0 min / 240.0 min (8.3%) |
| Scenarios passing | 4 | 4 |
| Milestones delivered | 6 / 19 | 7 / 20 |

The effective savings didn't change because S.3.1 and S.3.2 scenario tests still test
the pure compute engine, not the API. Advancing them to TwoStar requires updating the
scenario test functions to call the API — that's T-008-02's scope. The new milestone
makes the API route's engineering contribution visible on the dashboard.

## Acceptance criteria verification

| Criterion | Status |
|-----------|--------|
| GET /projects/:id/quote/:tier returns Quote JSON | Done |
| Loads zone geometry from PostGIS | Done |
| Loads tier assignments and materials from tenant catalog | Done |
| Returns 404 for missing project | Done |
| Returns 400 for invalid tier name | Done |
| Returns empty quote ($0 total) if tier has no assignments | Done |
| Integration test: create data via API, fetch quote, verify totals | Done |

## Open concerns

1. **Extra DB round-trip for tenant_id.** The handler calls `get_by_id` to get the
   project's `tenant_id` (needed for `list_by_tenant` on materials). This is an extra
   query beyond `verify_project_tenant`. Could be optimized by having `verify_project_tenant`
   return the `ProjectRow`, but that would change its signature for all callers. Low priority
   — one extra query per request at Lambda scale is negligible.

2. **Full catalog load.** The handler loads all materials for the tenant, even though only
   assigned ones are needed. For typical catalog sizes (tens to low hundreds), this is fine.
   If a tenant has thousands of materials, a targeted query would be more efficient.

3. **Scenario TwoStar advancement.** S.3.1 and S.3.2 can now claim TwoStar since the API
   exists, but the scenario test functions need to be updated to exercise the HTTP path
   to make the claim honest. This is explicitly T-008-02's scope.

## Quality gate

```
just check → All gates passed.
```
- `fmt-check` — pass
- `lint` (clippy strict) — pass
- `test` — 127 passed, 0 failed, 28 ignored
- `scenarios` — 4 pass, 0 fail, 13 not implemented, 0 blocked
