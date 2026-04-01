# T-025-03 Research: Claim Unclaimed Milestones

## Current State

`tests/scenarios/src/progress.rs` contains 24 milestones. 19/24 are delivered.
The scenario dashboard reports 76.5 min / 240.0 min (31.9%) effective savings.

## The Four Milestones in Question

All four milestones are already claimed, but attributed to T-025-02 or T-025-01
(the full-stack round-trip tickets) rather than the original Sprint 1 tickets
that actually delivered the work. The ticket was written when milestones were at
15/24, but T-025-02 already claimed them during its implementation.

### 1. "SvelteKit frontend + CF Worker proxy" (line 315)

- **Current:** `delivered_by: Some("T-025-02")`
- **Should be:** T-005-02 (CF Worker) + T-005-03 (route skeleton + API client)
- **Evidence:** `web/` directory contains SvelteKit app, `worker/` contains CF Worker proxy.
  T-005-02 delivered the CF Worker with CORS, rate limiting, auth passthrough, SSE streaming.
  T-005-03 delivered route skeleton, layout components, API client wrapper, mock API mode.
- **Current note** is generic; needs more specificity about what each ticket delivered.

### 2. "pt-project: Project/Zone/Tier model + GeoJSON serde" (line 171)

- **Current:** `delivered_by: Some("T-025-02")`
- **Should be:** T-002-01 (domain crates) + T-003-02 (repo layer)
- **Evidence:** pt-repo crate has ProjectRepo, ZoneRepo with GeoJSON serde.
  T-002-01 delivered domain types (Project, Zone, Tier, MaterialAssignment).
  T-003-02 delivered the repository layer with PostGIS GeoJSON round-trip.
- **Current note** is adequate but attributes to wrong ticket.

### 3. "pt-quote: quantity takeoff engine" (line 183)

- **Current:** `delivered_by: Some("T-025-02")`
- **Should be:** T-002-02 (pt-quote crate)
- **Evidence:** pt-quote crate exists with compute_quote() function.
  T-002-02 delivered the entire crate.
- **Current note** is adequate but attributes to wrong ticket.

### 4. "pt-tenant: multi-tenant model + auth context" (line 325)

- **Current:** `delivered_by: Some("T-025-01")`
- **Should be:** T-003-02 (TenantRepo) + T-004-02 (X-Tenant-Id extractor)
- **Evidence:** TenantRepo in pt-repo, X-Tenant-Id extractor in plantastic-api.
  T-003-02 delivered TenantRepo. T-004-02 delivered the header extractor.
- **Current note** already mentions both tickets in the text but delivered_by
  credits T-025-01 which only *verified* the isolation, not *built* it.

## File to Modify

Only one file: `tests/scenarios/src/progress.rs`

## Constraints

- Must not change any milestone's `unlocks` list
- Must not change any other milestone's `delivered_by`
- `just check` must pass after changes
- Dashboard milestone count must remain 19/24 (no regression)

## Conclusion

This is a metadata correction — fixing attribution from the verifying tickets
(T-025-01, T-025-02) to the original delivering tickets. The notes should also
be updated to accurately describe what each original ticket built.
