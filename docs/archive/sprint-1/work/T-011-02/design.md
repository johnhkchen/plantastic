# T-011-02 Design — Prepopulation API

## Decision: Synchronous Inline Baseline with Graceful Fallback

### Approach

When `POST /projects` receives an address, invoke `BaselineBuilder::build()` via
`spawn_blocking`, serialize the result to JSON, and store it in the `baseline` JSONB
column in the same request. If baseline generation fails (unknown address, no parcel
data), create the project anyway without baseline — log the error, don't fail the request.

Return baseline in `ProjectResponse` on both POST and GET.

### Why This Approach

1. **Simple**: No background job system, no polling, no status field. The baseline is
   either there or it isn't.
2. **Fast enough**: EmbeddedSource completes in <10ms. Even with real data sources later,
   the geocode+parcel+canopy pipeline should complete in <2s — acceptable for project
   creation.
3. **Graceful**: Unknown addresses don't block project creation. The user can still
   manually design without satellite pre-population.
4. **Testable**: The scenario can POST a project with the known test address, GET it back,
   and verify the baseline — clean TwoStar path.

### Rejected Alternatives

**A. Background job with status polling**
- Requires: job queue, status field, polling endpoint, frontend polling logic.
- Complexity: 5x the code for a pipeline that currently takes <10ms.
- When to revisit: when real data sources are introduced and latency exceeds ~3s.
- Verdict: premature infrastructure.

**B. Make satellite traits async**
- Requires: refactoring all of pt-satellite, EmbeddedSource, and BaselineBuilder.
- Would be the right long-term move when real HTTP-based data sources arrive.
- But for embedded data, sync is correct — there's no I/O to await.
- Verdict: out of scope. spawn_blocking bridges the gap correctly.

**C. Separate endpoint (POST /projects/:id/baseline)**
- Decouples creation from baseline generation.
- But the acceptance criteria says "POST /projects with address triggers" — it should
  happen automatically.
- Verdict: doesn't match the spec. May be useful later for re-generation.

### Error Handling Strategy

Baseline failure → project still created, `baseline` field is `null`. Reasoning:
- The address might not be in EmbeddedSource's test data.
- Even with real data, geocoding/parcel lookups can fail transiently.
- A project without baseline is still useful — the user just doesn't get pre-population.
- Specific error mapping:
  - `AddressNotFound` → log warn, baseline = null
  - `NoParcelData` → log warn, baseline = null
  - `CanopyUnavailable` → log warn, baseline = null

### Data Flow

```
Client                     API                          pt-satellite    pt-repo
  │                         │                               │              │
  ├─POST /projects─────────►│                               │              │
  │  {address, client_name} │                               │              │
  │                         ├─create(pool, input)──────────►│──────────────►│
  │                         │◄─────────────────── id ◄──────│──────────────┤
  │                         │                               │              │
  │                         ├─spawn_blocking────────────────►│              │
  │                         │  builder.build(address)        │              │
  │                         │◄──────── Ok(baseline) ────────┤              │
  │                         │                               │              │
  │                         ├─set_baseline(pool, id, json)─►│──────────────►│
  │                         │◄─────────────────────────────┤◄──────────────┤
  │                         │                               │              │
  │                         ├─get_by_id(pool, id)──────────►│──────────────►│
  │                         │◄─────────── ProjectRow ──────┤◄──────────────┤
  │                         │                               │              │
  │◄─201 {id, baseline,...}─┤                               │              │
```

### AppState Extension

Add an `EmbeddedSource`-backed `BaselineBuilder` to `AppState`. This keeps the builder
injectable and testable. Since `EmbeddedSource` is `Clone` + stateless, this is trivial.

To avoid making AppState generic over the satellite traits, store a concrete
`BaselineBuilder<EmbeddedSource, EmbeddedSource, EmbeddedSource>`. When real sources are
added, this type will change — that's fine, it's an internal detail.

However, since `BaselineBuilder::build()` takes `&self` and the builder itself only
stores the data sources (stateless), we can also construct it inline in the handler.
This avoids touching AppState at all. The builder is cheap to construct — just 3 clones
of `EmbeddedSource` (a unit struct).

**Decision**: Construct inline. Simpler, no AppState changes, no trait bounds propagation.

### Frontend Changes

Extend the `Project` interface with an optional `baseline` field (raw JSON object).
On the project page, add a "Site Baseline" section that renders:
- Lot area (sqft) and polygon vertex count
- Number of detected trees with a summary table (height, spread, confidence)
- Sun grid dimensions

This is a data display — no map rendering, no canvas. Maps come with a later ticket.

### Scenario S.1.2 TwoStar

The scenario test needs to exercise the API path. Options:
1. **In-process router test**: Use `axum::Router` directly with a real `PgPool`. This is
   the pattern already used by the `#[ignore]` integration tests in plantastic-api.
2. **Standalone HTTP test**: Spin up the server, make HTTP requests.

Option 1 is better — matches existing patterns, faster, no port binding needed.

But: the scenario harness runs as `cargo run -p pt-scenarios`, which is a separate binary.
It currently calls pt-satellite directly (no database). To make it call through the API,
it would need either a database connection or mock infrastructure — which violates "no
mocks across crate boundaries."

**Decision**: Keep S.1.2 scenario at the library level (calling BaselineBuilder directly
is correct — it tests the capability). Add a separate API-level integration test in
`plantastic-api` that verifies the round-trip (POST with address → GET → baseline
present). Update the S.1.2 scenario to TwoStar by adding a serialization round-trip
check: serialize ProjectBaseline to JSON, deserialize back, verify all fields survive.
This proves the baseline is JSON-safe for storage, which is the key enabler for the
API path.

### Milestone

Claim or update the pt-satellite milestone note: "T-011-02 wires BaselineBuilder into
POST /projects. Baseline stored as JSONB, returned on GET. S.1.2 upgraded to TwoStar."
