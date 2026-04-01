# T-025-02 Design: Full-Stack Round-Trip Scenario

## Decision

**Single approach: mirror the S.INFRA.2 pattern with 9 sequential API steps.**

There's only one viable approach here — this is a scenario test with a precisely
defined 9-step flow from the ticket AC. The design decisions are about details.

## Test Function Shape

```
s_infra_1_full_stack() → ScenarioOutcome
  ├── Check DATABASE_URL, return Blocked if missing
  ├── Create tokio runtime
  └── block_on(s_infra_1_api())

s_infra_1_api() → ScenarioOutcome [async]
  ├── Setup: pool, setup_db, create_tenant, router
  ├── Step 1: POST /projects → 201, capture project_id
  ├── Step 2: GET /projects/:id → 200, verify client_name
  ├── Step 3: POST /projects/:id/zones (12×15 patio) → 201, capture zone_id
  ├── Step 4: GET /projects/:id/zones → 200, verify zone present
  ├── Step 5: POST /materials (Travertine, $8.50/sqft) → 201, capture material_id
  ├── Step 6: PUT /projects/:id/tiers/good → 200, assign material to zone
  ├── Step 7: GET /projects/:id/quote/good → 200, verify line_total = "1530.00"
  ├── Step 8: DELETE /projects/:id → 200
  └── Step 9: GET /projects/:id → 404
```

## Design Decisions

### Quote Verification (Step 7)

Per CLAUDE.md rule 2: expected values computed independently.
12 × 15 = 180 sqft. 180 × $8.50 = $1,530.00.
Assert `line_items[0].line_total == "1530.00"` and `subtotal == "1530.00"`.

### Geometry Verification (Step 4)

Verify the zone is returned in the zones list. Don't need to deep-compare
GeoJSON coordinates — just confirm zone_id exists and zone_type matches.
Full geometry round-trip is covered by S.3.1.

### Error Handling

Every api_call returns Result. Use match + early return pattern (not unwrap).
Errors → `ScenarioOutcome::Fail(msg)` with step context.
Missing DATABASE_URL → `ScenarioOutcome::Blocked(msg)`.

### Tier Assignment Body

From quoting.rs S.3.1, the PUT /tiers/:tier body is:
```json
{
  "assignments": [
    { "zone_id": "<uuid>", "material_id": "<uuid>" }
  ]
}
```

### Rating

TwoStar integration (API-level, no UI), OneStar polish (infrastructure test).
0.0 time_savings_minutes (infrastructure correctness, not user value).

### Milestones

Claim three milestones in progress.rs:
1. pt-project — model exists, used by API routes
2. pt-quote — engine exists, used by quote route
3. SvelteKit frontend + CF Worker proxy — note: frontend deployed, CF Worker active

These milestones already have code behind them — they just haven't been formally
claimed. This test proves the full stack works end-to-end.

## Rejected Alternatives

**Computation-only fallback**: Unlike S.3.1 which can fall back to testing
pt-quote directly without a database, S.INFRA.1 is specifically about the
full stack. No fallback — just Blocked if no DATABASE_URL.

**Deep geometry assertion**: Could compare all GeoJSON coordinates, but that's
testing the serialization layer which is already covered. Just verify zone exists.
