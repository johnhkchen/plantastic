# T-025-02 Research: Full-Stack Round-Trip Scenario

## Objective

Implement S.INFRA.1 — a 9-step API-level scenario test that validates the complete
CRUD lifecycle: create project → add zones → add materials → assign to tier →
compute quote → delete → verify gone.

## Current State

### S.INFRA.1 Scenario Stub

`tests/scenarios/src/suites/infrastructure.rs:26-44` — currently returns
`ScenarioOutcome::NotImplemented`. The 9-step spec is documented in comments.

### S.INFRA.2 (Sibling) — Fully Implemented

`infrastructure.rs:46-244` — T-025-01 implemented tenant isolation scenario.
Uses the same api_helpers module, same setup pattern (pool → setup_db →
create_tenant → router → api_call chain). Returns `TwoStar/OneStar`.

### API Helpers Module

`tests/scenarios/src/api_helpers.rs` provides:

- `scenario_pool()` → `Result<PgPool, String>` from DATABASE_URL
- `setup_db(&PgPool)` → runs migration SQL files
- `create_tenant(&PgPool, name)` → `Result<Uuid, String>`
- `router(pool)` → `axum::Router` with S3, scan jobs, AppState
- `api_call(&Router, Method, uri, tenant_id, body)` → `Result<(StatusCode, Value), String>`

### Available API Endpoints (from T-004-02)

| Method | Route | Expected Status |
|--------|-------|-----------------|
| POST | /projects | 201 |
| GET | /projects/:id | 200 (or 404) |
| DELETE | /projects/:id | 200 |
| POST | /projects/:id/zones | 201 |
| GET | /projects/:id/zones | 200 |
| POST | /materials | 201 |
| PUT | /projects/:id/tiers/:tier | 200 (or 204) |
| GET | /projects/:id/quote/:tier | 200 |

### GeoJSON Convention

`quoting.rs:44-49` defines `geojson_rect(w, h)` — rectangle polygon from (0,0).
The ticket specifies 12×15 ft patio with geometry:
`{"type":"Polygon","coordinates":[[[0,0],[12,0],[12,15],[0,15],[0,0]]]}`.

### Quote Verification Pattern

From S.3.1 (quoting.rs): line_total values are JSON strings like `"1530.00"`,
compared with `==`. The expected total for 12×15 @ $8.50/sqft = 180 × 8.50 =
$1,530.00, computed independently in the test (not by pt-geo).

### Zone Creation Body

From S.INFRA.2 and S.3.1: zones require `geometry`, `zone_type`, and `label`.
Zone type values: "Patio", "Bed", etc. (capitalized in S.INFRA.2, lowercase in
S.3.1 — need to check which the API accepts).

### Tier Assignment Body

From S.INFRA.2:232: `json!({ "assignments": [] })` for empty assignments.
From S.3.1: need to find how zone_id and material_id are structured in PUT tiers.

### Milestones to Claim

Three milestones have `delivered_by: None` and unlock S.INFRA.1:

1. **pt-project: Project/Zone/Tier model** (progress.rs:171-175) — unlocks S.3.1, S.3.2, S.3.4, S.INFRA.1
2. **pt-quote: quantity takeoff engine** (progress.rs:177-181) — unlocks S.3.1, S.3.2
3. **SvelteKit frontend + CF Worker proxy** (progress.rs:304-308) — unlocks S.INFRA.1, S.3.4

Per ticket AC: claim all three with notes explaining what exists.

### Dependencies

T-025-01 (tenant isolation) is a dependency — already complete. S.INFRA.2 is
implemented and blocked only by DATABASE_URL. The api_helpers infrastructure is
proven.

### Dashboard Baseline

- Effective savings: 69.5 / 240.0 min (29.0%)
- S.INFRA.1: NOT IMPLEMENTED, prereqs 5/7 met
- S.INFRA.2: BLOCKED (no DATABASE_URL), prereqs 4/4 met
- Milestones: 16/24 delivered

## Key Findings

1. The implementation pattern is well-established by S.INFRA.2 and S.3.1.
2. All API routes exist and are tested by integration tests.
3. The api_helpers module handles all the async/HTTP boilerplate.
4. The main work is writing the 9-step test function following the pattern.
5. Need to check how tier assignments reference zone_id and material_id
   (the PUT body format from quoting.rs S.3.1 API path).
6. Infrastructure scenarios have 0 time_savings_minutes (correctness-only).
