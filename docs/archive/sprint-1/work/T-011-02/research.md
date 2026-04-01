# T-011-02 Research — Prepopulation API

## Ticket Summary

Wire pt-satellite into the API so `POST /projects` with an address triggers baseline
generation, stores it on the project record, and returns it via `GET /projects/:id`.
Frontend should display lot boundary and detected trees. Upgrade S.1.2 to TwoStar.

## Current State

### pt-satellite crate (delivered by T-011-01)

- `BaselineBuilder<G, P, C>` orchestrates: address → geocode → lot boundary → tree
  detection → pt-solar radiance grid → `ProjectBaseline`.
- Traits: `Geocoder`, `ParcelSource`, `CanopySource` — all synchronous (no async).
- `EmbeddedSource` implements all three traits for one test address:
  "1234 Noriega St, San Francisco, CA" (37.7601, -122.4862).
- `ProjectBaseline` has `#[derive(Serialize, Deserialize)]` — JSON-round-trippable.
- `LotBoundary.polygon` uses custom serde via `serde_helpers::geojson_polygon`.
- `SatelliteError`: `AddressNotFound(String)`, `NoParcelData{lat,lng}`, `CanopyUnavailable`.
- `BaselineBuilder::build()` is synchronous — calls `radiance_grid()` which is CPU-bound
  (~few ms for test data, but potentially heavier for real data later).

### plantastic-api (delivered by T-004-02)

**POST /projects handler** (`crates/plantastic-api/src/routes/projects.rs:59-73`):
- Accepts `CreateProjectRequest { address, client_name, client_email }` (all optional).
- Creates via `pt_repo::project::create()` which only inserts the 4 basic fields.
- Returns `ProjectResponse` — does NOT include `baseline` or `scan_ref` fields.

**GET /projects/:id handler** (`:83-93`):
- Fetches `ProjectRow` (which includes `baseline: Option<serde_json::Value>`).
- Converts to `ProjectResponse` — drops `baseline` and `scan_ref` on the floor.

**AppState**: Just `{ pool: PgPool }`. No satellite builder or data sources stored.

**Error mapping**: `RepoError → AppError` exists. No `SatelliteError → AppError` mapping.

### pt-repo project module (`crates/pt-repo/src/project.rs`)

- `ProjectRow` has `baseline: Option<serde_json::Value>` — already there.
- `CreateProject` struct does NOT have a baseline field.
- No `set_baseline()` or `update_baseline()` function exists.
- The `create()` SQL only inserts `tenant_id, client_name, client_email, address`.
- The DB column `baseline JSONB` exists in migration 002 — ready to receive data.

### Database schema (migration 002)

```sql
baseline    JSONB,  -- satellite baseline (lot, trees, sun grid)
```

Column exists, nullable, no constraints. Ready to store serialized ProjectBaseline.

### Frontend

**Project store** (`web/src/lib/stores/project.svelte.ts`):
- `Project` interface has no `baseline` field — only id, tenant_id, client_name/email,
  address, status, timestamps.

**Project page** (`web/src/routes/(app)/project/[id]/+page.svelte`):
- Loads project + zones on mount.
- Displays: client_name, address, status badge, zone count, created_at, client_email.
- No baseline rendering. No map component. No tree display.

### Scenario S.1.2

- Currently passes at OneStar: builds baseline from `EmbeddedSource` directly, validates
  coordinates/lot/trees/sun_grid.
- TwoStar requirement: wire through API — POST project with address, GET it back,
  verify baseline is present and valid.
- The scenario test is synchronous — an API round-trip test would need either:
  (a) a real Postgres + HTTP request (integration test), or
  (b) in-process router test with sqlx test database.

### Sync vs Async Consideration

The `BaselineBuilder::build()` is synchronous. The Axum handler is async. Options:
1. Call synchronously in handler — blocks the Tokio thread. With EmbeddedSource, this
   takes <10ms, but it's not correct practice. With real data sources (HTTP calls), this
   would deadlock.
2. `tokio::task::spawn_blocking()` — correct for CPU-bound sync work. Moves the call to
   the blocking thread pool.
3. Make traits async — large refactor of pt-satellite, out of scope for this ticket.

Option 2 is the right call. spawn_blocking is the standard pattern for sync-in-async.

### Missing Pieces Inventory

1. **Repo layer**: Need `set_baseline(pool, project_id, baseline_json)` function.
2. **API response**: `ProjectResponse` needs `baseline: Option<serde_json::Value>`.
3. **API handler**: `create_project` needs to invoke `BaselineBuilder` when address is set.
4. **Error mapping**: `SatelliteError → AppError` conversion.
5. **AppState**: Need to carry a `BaselineBuilder` (or just construct inline with `EmbeddedSource`).
6. **Frontend**: `Project` interface needs `baseline` field. Project page needs baseline display.
7. **Scenario**: S.1.2 needs a TwoStar path exercising API round-trip.
8. **Cargo.toml**: plantastic-api needs `pt-satellite` dependency.

### Constraints

- EmbeddedSource only knows one address. Unknown addresses → `AddressNotFound` error.
- Baseline generation should not block project creation for unknown addresses — project
  should still be created, just without baseline.
- The acceptance criteria says "async if needed" — for EmbeddedSource the computation is
  fast enough to run inline. Background job infrastructure would be premature.
- Frontend baseline display is specified as "shows lot boundary and detected trees if
  baseline exists" — a simple data panel is sufficient; no map component needed yet.
