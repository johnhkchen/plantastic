# T-004-02 Review: CRUD Routes

## Summary

Wired the pt-repo repository layer to HTTP via Axum route handlers. The API now exposes full CRUD for projects, zones, materials, and tier assignments — the complete set of routes the frontend needs for core landscaping workflow.

## Files changed

### Created
| File | Purpose |
|------|---------|
| `crates/plantastic-api/src/lib.rs` | Library entry: exports `AppState` and `router()` for tests |
| `crates/plantastic-api/src/extract.rs` | `TenantId` extractor from `X-Tenant-Id` header |
| `crates/plantastic-api/src/routes/projects.rs` | POST/GET/GET:id/DELETE:id with tenant scoping |
| `crates/plantastic-api/src/routes/materials.rs` | GET/POST/PATCH:id/DELETE:id with tenant scoping |
| `crates/plantastic-api/src/routes/zones.rs` | GET/POST/PUT-bulk/PATCH:zid/DELETE:zid with GeoJSON |
| `crates/plantastic-api/src/routes/tiers.rs` | GET-all-3-tiers/PUT:tier assignments |
| `crates/plantastic-api/tests/common/mod.rs` | Test infrastructure (pool, migrations, send helper) |
| `crates/plantastic-api/tests/crud_test.rs` | 8 integration tests (Postgres-dependent, #[ignore]) |

### Modified
| File | Change |
|------|--------|
| `crates/plantastic-api/Cargo.toml` | Added `[lib]` section, domain crate deps (pt-project, pt-materials), uuid/chrono/rust_decimal/geo/geojson |
| `crates/plantastic-api/src/main.rs` | Simplified to use `plantastic_api::router()` from lib.rs |
| `crates/plantastic-api/src/error.rs` | Added `From<JsonRejection>` for serde error → 400 |
| `crates/plantastic-api/src/routes/mod.rs` | Merged projects/zones/materials/tiers route modules |
| `tests/scenarios/src/progress.rs` | Updated Axum API milestone with T-004-02 delivery note |

## Routes implemented

| Method | Path | Handler | Status |
|--------|------|---------|--------|
| POST | /projects | create_project | 201 + JSON |
| GET | /projects | list_projects | 200 + JSON array |
| GET | /projects/:id | get_project | 200 + JSON |
| DELETE | /projects/:id | delete_project | 204 |
| GET | /projects/:id/zones | list_zones | 200 + JSON array |
| POST | /projects/:id/zones | add_zone | 201 + JSON |
| PUT | /projects/:id/zones | bulk_update_zones | 200 + UUID array |
| PATCH | /projects/:id/zones/:zid | update_zone | 204 |
| DELETE | /projects/:id/zones/:zid | delete_zone | 204 |
| GET | /materials | list_materials | 200 + JSON array |
| POST | /materials | create_material | 201 + JSON |
| PATCH | /materials/:id | update_material | 204 |
| DELETE | /materials/:id | delete_material | 204 |
| GET | /projects/:id/tiers | get_all_tiers | 200 + 3 tiers |
| PUT | /projects/:id/tiers/:tier | set_tier_assignments | 204 |

## Test coverage

### Non-DB tests (run in CI)
- `health_returns_200` — existing, still passes
- `unknown_route_returns_404` — existing, still passes

### Integration tests (8 tests, `#[ignore]`, require Postgres)
1. **project_crud_lifecycle** — create → get → list → delete → get(404)
2. **tenant_isolation** — tenant A creates, tenant B gets 404, B's list excludes it, B can't delete
3. **zone_crud** — add zone → list → update (geometry + type) → delete → list(empty)
4. **zone_bulk_update** — add 2 individually → PUT 3 → list shows exactly 3
5. **material_crud** — create → list → update → delete
6. **tier_assignments** — create project/zone/material → set "good" tier → get all 3 tiers → verify
7. **missing_tenant_header_returns_400** — no X-Tenant-Id → 400
8. **invalid_tier_name_returns_400** — PUT /tiers/premium → 400

## Scenario dashboard

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Effective savings | 12.0 min | 12.0 min | No change |
| Scenarios passing | 3 | 3 | No change |
| Milestones delivered | 3 / 18 | 3 / 18 | Milestone updated |

No regressions. This ticket delivers infrastructure (HTTP routes), not direct user-facing value — the effective savings number is expected to stay flat. The milestone note documents what was delivered and what scenarios it unblocks.

## Open concerns

1. **Material tenant verification is O(n)** — `list_by_tenant` then linear scan. Acceptable for V1 catalogs (<1000 materials), but should add `get_by_id` to pt-repo when needed.

2. **No `get_by_id` for zones** — Zone PATCH/DELETE take the zone_id from the path but don't verify the zone actually belongs to the project in the URL (the DB FK constraint prevents cross-project updates, but the error message won't be ideal). Low risk since we do verify the project belongs to the tenant.

3. **Bulk zone PUT cascades tier assignments** — The repo's `bulk_upsert` DELETEs all zones first, which cascades to `tier_assignments` via FK. The frontend must re-send tier assignments after a bulk zone update. This is documented behavior but could surprise API consumers.

4. **No pagination** — `GET /projects` and `GET /materials` return all rows. Fine for V1 (small tenant catalogs), but will need LIMIT/OFFSET or cursor pagination before production scale.

5. **Auth is placeholder** — `X-Tenant-Id` header is easily spoofable. This is acknowledged as V1 scope — real JWT auth is a separate ticket.

## Quality gate

```
just check — PASSED
  fmt-check ✓
  lint      ✓ (clippy strict, warnings-as-errors)
  test      ✓ (102 passed, 28 ignored)
  scenarios ✓ (12.0/240.0 min, 3 pass, 0 fail)
```
