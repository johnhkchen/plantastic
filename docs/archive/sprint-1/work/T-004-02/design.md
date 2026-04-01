# T-004-02 Design: CRUD Routes

## Decision: Flat handler modules with API-specific DTOs

### Approach

One route module per resource group (`projects`, `zones`, `materials`, `tiers`), each defining:
1. Request DTOs (serde `Deserialize`) for inputs
2. Response DTOs (serde `Serialize`) for outputs
3. Async handler functions
4. A `routes()` function returning `Router<AppState>`

### Tenant scoping

**Decision:** Header-based tenant ID (`X-Tenant-Id`) via a custom Axum extractor.

V1 has no auth. The ticket says "placeholder: tenant_id from header or hardcoded for V1." A header extractor is:
- Simple to implement (5 lines)
- Easy to test (just set a header)
- Easy to replace later (swap extractor impl, handler signatures don't change)
- Safe by default (no tenant_id = 400 error, prevents accidental cross-tenant access)

The extractor returns `TenantId(Uuid)` and handlers pass it to repo functions. For project `get_by_id` and `delete`, we fetch the project and verify `tenant_id` matches before proceeding.

Rejected: hardcoded tenant_id — makes testing tenant isolation impossible, which S.INFRA.2 needs.

### Request/response DTOs

**Decision:** Separate API DTOs from repo input types.

The API layer defines its own request structs that map to repo input types. This keeps:
- API concerns (JSON field names, validation) in the API crate
- Repo concerns (SQL binding) in the repo crate
- No coupling between what the frontend sends and what the DB layer expects

Zone geometry in API requests/responses uses GeoJSON format (same as `pt_project::Zone` serde). We use the `geojson` crate's `Geometry` type in DTOs and convert to `geo::Polygon<f64>` before calling the repo.

### Route structure

```
POST   /projects              → create_project
GET    /projects              → list_projects
GET    /projects/:id          → get_project
DELETE /projects/:id          → delete_project

GET    /projects/:id/zones    → list_zones
PUT    /projects/:id/zones    → bulk_update_zones
POST   /projects/:id/zones    → add_zone
PATCH  /projects/:id/zones/:zid → update_zone
DELETE /projects/:id/zones/:zid → delete_zone

GET    /materials             → list_materials
POST   /materials             → create_material
PATCH  /materials/:id         → update_material
DELETE /materials/:id         → delete_material

GET    /projects/:id/tiers         → get_all_tiers
PUT    /projects/:id/tiers/:tier   → set_tier_assignments
```

### Validation strategy

**Decision:** Rely on serde deserialization + domain constraints, not a validation framework.

- Missing required fields → serde fails → Axum returns 422 (or we catch with custom rejection)
- Invalid enum values → serde fails (ZoneType, TierLevel, etc. already have `snake_case` serde)
- Invalid UUIDs in path → Axum `Path<Uuid>` fails → 400
- Business rules (e.g., valid tier name) → checked in handler, returned as `BadRequest`

Rejected: validator crate — adds dependency for what serde + 2-3 manual checks already handle.

### Error handling

Use existing `AppError` as-is. The `From<RepoError>` conversion handles:
- `NotFound` → 404
- `Conflict` → 409
- `Database` → 500
- `Conversion` → 500

Add custom Axum rejection handler to turn serde deserialization errors into `AppError::BadRequest` with the serde error message, instead of Axum's default 422.

### Testing strategy

**Unit-style tests** (no DB): Build a router with a mock state? No — CLAUDE.md says no mocking across crate boundaries.

**Integration tests** (with DB): Mark with `#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-004-02"]`. Use real pool + migrations + `tower::ServiceExt::oneshot` to hit real handlers with a real database.

These tests will:
1. Create a tenant (prereq for all operations)
2. Exercise each CRUD route
3. Verify tenant isolation (one tenant can't see another's data)
4. Verify cascade behavior (deleting a project removes its zones)

### What was rejected

1. **Trait-based handlers** — Unnecessary abstraction for CRUD routes that map 1:1 to repo calls.
2. **Generic CRUD macro** — The routes differ enough (tenant scoping, nested resources, bulk ops) that a generic macro would be more complex than the code it replaces.
3. **Embedded validation framework** — Serde + manual checks are sufficient for V1's straightforward inputs.
4. **Middleware-based tenant verification** — Would need to parse the request body to know which project to check. Simpler to do it in each handler.

### Scenario impact

This ticket completes the HTTP wiring between frontend and repo. After this:
- S.INFRA.1 (full stack round-trip) has 3/4 prereqs met (still needs frontend)
- S.INFRA.2 (tenant isolation) has 3/3 prereqs from the backend side
- Milestone "Axum API: routes + Lambda deployment" gets extended from skeleton to full CRUD
