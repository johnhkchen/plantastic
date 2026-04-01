# T-004-02 Plan: CRUD Routes

## Step 1: Dependencies and extractors

**Changes:**
- Update `Cargo.toml` to add pt-project, pt-materials, chrono, uuid, rust_decimal, geojson, geo
- Create `src/extract.rs` with `TenantId` extractor
- Update `src/error.rs` to handle JSON rejection (serde errors → 400)
- Update `src/main.rs` to add `mod extract;` and remove `#[allow(dead_code)]`

**Verification:** `cargo check -p plantastic-api` compiles

## Step 2: Project routes

**Changes:**
- Create `src/routes/projects.rs` with all four handlers + DTOs
- Wire into `routes/mod.rs`

**Handlers:**
- `POST /projects` — creates project, returns 201 + ProjectResponse
- `GET /projects` — lists tenant's projects
- `GET /projects/:id` — gets project, verifies tenant match
- `DELETE /projects/:id` — deletes project, verifies tenant match

**Verification:** `cargo check -p plantastic-api`

## Step 3: Material routes

**Changes:**
- Create `src/routes/materials.rs` with all four handlers + DTOs
- Wire into `routes/mod.rs`

**Handlers:**
- `GET /materials` — lists tenant's materials
- `POST /materials` — creates material, returns 201
- `PATCH /materials/:id` — updates material (needs tenant verification via a get-then-check)
- `DELETE /materials/:id` — deletes material

**Verification:** `cargo check -p plantastic-api`

## Step 4: Zone routes

**Changes:**
- Create `src/routes/zones.rs` with all five handlers + DTOs + GeoJSON conversion helper
- Wire into `routes/mod.rs`
- Add `verify_project_tenant` helper (pub(crate) for reuse)

**Handlers:**
- `GET /projects/:id/zones` — lists zones for project
- `POST /projects/:id/zones` — adds single zone
- `PUT /projects/:id/zones` — bulk replaces all zones
- `PATCH /projects/:id/zones/:zid` — updates zone
- `DELETE /projects/:id/zones/:zid` — deletes zone

All routes verify project belongs to tenant before proceeding.

**Verification:** `cargo check -p plantastic-api`

## Step 5: Tier routes

**Changes:**
- Create `src/routes/tiers.rs` with two handlers + DTOs
- Wire into `routes/mod.rs`

**Handlers:**
- `GET /projects/:id/tiers` — returns all three tiers (good, better, best) with assignments
- `PUT /projects/:id/tiers/:tier` — replaces assignments for one tier

Both routes verify project belongs to tenant.

**Verification:** `cargo check -p plantastic-api`

## Step 6: Compile, format, lint

- `just fmt`
- `just lint` — fix any clippy warnings
- `just test` — existing tests still pass (health test, domain tests, repo tests)

## Step 7: Integration tests

**Changes:**
- Create `crates/plantastic-api/tests/crud_test.rs`
- Tests marked `#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-004-02"]`
- Use real PgPool + migrations + oneshot requests through the full router

**Test cases:**
1. Project CRUD lifecycle: create → get → list → delete → get (404)
2. Zone CRUD: create project → add zone → list → update → delete
3. Zone bulk update: create project → add zones → PUT replaces all
4. Material CRUD: create → list → update → delete
5. Tier assignments: create project + zones + materials → set tier → get tiers
6. Tenant isolation: create as tenant A → get as tenant B → 404
7. Validation: missing required fields → 400, invalid UUID → 400
8. Missing tenant header → 400

## Step 8: Quality gate

- `just check` — fmt + lint + test + scenarios
- Verify scenario dashboard: no regressions, claim milestone

## Step 9: Claim milestone

- Update `tests/scenarios/src/progress.rs` milestone for "Axum API: routes + Lambda deployment" to reflect CRUD routes delivery
