# T-025-01 Design: Tenant Isolation Scenario

## Decision

Implement S.INFRA.2 as an API-level async test following the S.3.1 pattern, with no computation fallback. Return `Blocked` when DATABASE_URL is absent.

## Approach: Full API-Level Test (TwoStar Integration)

### Test Flow

```
1. Setup: pool → migrations → create Tenant A + Tenant B → build router
2. Tenant A creates project → assert 201
3. Tenant B GETs project by ID → assert 404
4. Tenant A creates material → assert 201
5. Tenant B lists materials → assert Tenant A's material absent
6. Tenant B POSTs zone on Tenant A's project → assert 404
7. Tenant B PUTs tier assignment on Tenant A's project → assert 404
```

Seven API calls covering all four resource types (project, material, zone, tier).

### Why This Approach

1. **Matches acceptance criteria exactly** — the ticket specifies these 7 steps.
2. **Reuses api_helpers** — no new test infrastructure needed.
3. **Covers transitive isolation** — zones and tiers are scoped through projects, so testing cross-tenant zone/tier ops verifies the `verify_project_tenant` helper works end-to-end.
4. **404 verification** — explicitly asserts 404 (not 403) to confirm no existence leaking.

### Alternatives Considered

**A. Computation-only fallback (OneStar)**
Rejected. Tenant isolation is inherently a multi-actor, stateful test. There's no meaningful computation-only version — isolation requires two tenants hitting the same database.

**B. Repo-level test instead of API-level**
Rejected. The isolation is enforced at the route layer (the extractor + verify_project_tenant pattern), not at the repo layer. Repo functions don't take tenant_id — they return all rows matching a query. Testing at repo level would miss the actual enforcement point.

**C. Extend existing crud_test.rs instead of scenario**
Rejected. The ticket explicitly requires a scenario test in the harness for S.INFRA.2. The crud_test is complementary but doesn't feed the value dashboard.

### Milestone Claim

Claim "pt-tenant" milestone in `progress.rs` with `delivered_by: Some("T-025-01")`. The note will explain that TenantRepo + X-Tenant-Id extractor + verify_project_tenant already deliver multi-tenant isolation, and T-025-01 verifies it end-to-end.

### Error Handling Strategy

| Condition | Outcome |
|-----------|---------|
| DATABASE_URL not set | `Blocked("no DATABASE_URL")` |
| Pool creation fails | `Blocked(error)` |
| Migration fails | `Fail("DB setup failed: ...")` |
| Tenant creation fails | `Fail(error)` |
| Any API call fails | `Fail(error)` |
| Wrong status code | `Fail("expected X, got Y")` |
| Material appears in B's list | `Fail("tenant isolation breach: ...")` |
| All 7 steps pass | `Pass(Integration::TwoStar, Polish::OneStar)` |

### Integration Rating: TwoStar
- API-reachable: full HTTP request/response cycle through Axum router
- Database-backed: real Postgres, real migrations, real queries

### Polish Rating: OneStar
- Infrastructure correctness test — no UX surface to polish
