# T-025-01 Research: Tenant Isolation Scenario

## Objective

Implement S.INFRA.2 (Tenant isolation) scenario test and claim the "pt-tenant" milestone.

## Existing Tenant Infrastructure

### Tenant Model
- **Schema**: `migrations/001-create-tenants.sql` — `tenants` table with UUID PK, name, logo_url, brand_color, contact (JSONB), timestamps.
- **Repository**: `crates/pt-repo/src/tenant.rs` — `create(pool, name) -> Uuid`, `get_by_id(pool, id) -> TenantRow`.

### Tenant Scoping in Data Model
- **Projects**: `migrations/002-create-projects.sql` — `tenant_id UUID NOT NULL REFERENCES tenants(id)`, indexed.
- **Materials**: `migrations/004-create-materials.sql` — `tenant_id UUID NOT NULL REFERENCES tenants(id)`, indexed.
- Zones are scoped via project (no direct tenant FK; isolation is transitive through project ownership).
- Tier assignments are scoped via project (same transitive pattern).

### Tenant Extraction
- **Extractor**: `crates/plantastic-api/src/extract.rs` — custom Axum `FromRequestParts` impl pulls UUID from `X-Tenant-Id` header. Returns 400 if missing/invalid.

### Tenant Enforcement in Routes

All routes use tenant-scoped queries or verify-then-404 patterns:

| Route | File | Pattern |
|-------|------|---------|
| GET /projects | `routes/projects.rs` | `list_by_tenant(pool, tenant.0)` — only returns tenant's own |
| GET /projects/:id | `routes/projects.rs` | Fetch by ID, check `tenant_id != tenant.0` → 404 |
| POST /projects | `routes/projects.rs` | Creates with caller's tenant_id |
| DELETE /projects/:id | `routes/projects.rs` | Same verify-then-404 pattern |
| GET /materials | `routes/materials.rs` | `list_by_tenant(pool, tenant.0)` |
| PATCH /materials/:id | `routes/materials.rs` | Checks material exists in tenant's catalog |
| POST /projects/:id/zones | `routes/zones.rs` | `verify_project_tenant()` helper → 404 |
| GET /projects/:id/zones | `routes/zones.rs` | Same `verify_project_tenant()` |
| GET /projects/:id/quote/:tier | `routes/quotes.rs` | `verify_project_tenant()` |
| GET /projects/:id/tiers | `routes/tiers.rs` | `verify_project_tenant()` |
| PUT /projects/:id/tiers/:tier | `routes/tiers.rs` | `verify_project_tenant()` |

**Key**: Cross-tenant access returns 404, not 403. This is intentional — prevents existence leaking.

### `verify_project_tenant` Helper
- `crates/plantastic-api/src/routes/zones.rs:34-44` — shared by zones, quotes, tiers routes.
- Fetches project by ID, compares `tenant_id`, returns `AppError::NotFound` on mismatch.

## Scenario Harness

### Current State of S.INFRA.2
- `tests/scenarios/src/suites/infrastructure.rs:46-55` — returns `ScenarioOutcome::NotImplemented`.
- Has 5-step comment spec matching ticket acceptance criteria.

### API Helpers Available
- `tests/scenarios/src/api_helpers.rs` — `scenario_pool()`, `setup_db()`, `create_tenant()`, `router()`, `api_call()`.
- All return `Result` for graceful `Blocked`/`Fail` handling.
- `api_call()` sets `X-Tenant-Id` header automatically.

### Pattern to Follow
- `suites/quoting.rs:56-67` — S.3.1 pattern: check DATABASE_URL → build tokio runtime → `block_on(async_fn)`.
- Falls back to computation-only (OneStar) when no DB. For infra, no computation fallback makes sense — return `Blocked`.

### Milestones
- `progress.rs:310-314` — "pt-tenant: multi-tenant model + auth context" milestone exists with `delivered_by: None`.
- Unlocks: `["S.INFRA.2"]`.
- 3/4 prereqs already delivered: Axum routes (T-004-02), PostGIS schema (T-003-02), connection hardening (T-020-02).

### Existing Unit-Level Test
- `crates/plantastic-api/tests/crud_test.rs:87-138` — `tenant_isolation` test exists but is `#[ignore]` requiring Postgres.
- Covers projects only (create as A, fetch as B → 404, list as B → absent, delete as B → 404).
- Does NOT cover materials, zones, or tiers cross-tenant.

## Constraints
- No computation-only fallback — tenant isolation is inherently an API/DB test.
- Must return `Blocked("no DATABASE_URL")` when DATABASE_URL is absent.
- Pass at TwoStar integration (API-level), OneStar polish (infra correctness, no UX).
- time_savings_minutes stays at 0.0 (infrastructure correctness, not user time).

## Key Finding
The tenant isolation infrastructure is **fully built**. TenantRepo exists, X-Tenant-Id extraction works, all routes enforce tenant scoping with 404 on mismatch. The only missing piece is the scenario test itself and claiming the milestone. This is a verification ticket, not a build ticket.
