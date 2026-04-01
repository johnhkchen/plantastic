# T-038-02 Review: Fix Tenant Isolation Scenario

## Summary

Fixed S.INFRA.2 (tenant isolation) scenario which failed at Step 5 with 422 Unprocessable
Entity. Root cause: the test sent `"zone_type": "Patio"` (PascalCase) but the API's
`ZoneType` enum uses `#[serde(rename_all = "snake_case")]` and expects `"patio"`.

## Changes

### File Modified

**`tests/scenarios/src/suites/infrastructure.rs`**

1. **Fixed zone_type casing in S.INFRA.2** (the actual bug fix):
   - Step 5 (line ~443): `"Patio"` → `"patio"`
   - Step 6 (line ~468): `"Bed"` → `"bed"`

2. **Added response body to failure messages in S.INFRA.2** (7 steps):
   - All `ScenarioOutcome::Fail` messages now include the HTTP response body
   - Captured body where previously discarded with `_` (steps 2, 5, 6, 7)

3. **Added response body to failure messages in S.INFRA.1** (9 steps):
   - Same improvement for consistency
   - Captured body where previously discarded with `_` (steps 6, 8, 9)

### No API Code Changes

The API is correct. Only the test payload was wrong.

## Acceptance Criteria

| Criterion | Status |
|-----------|--------|
| Diagnose 422 response body + add error body logging | Done — all failure messages now include `{body}` |
| Fix zone POST payload to match API | Done — `"patio"` and `"bed"` (snake_case) |
| S.INFRA.2 passes at ★★☆☆☆ with DATABASE_URL | Cannot verify without DATABASE_URL; fix is correct per S.INFRA.1 pattern |
| `just check` (no DATABASE_URL) still passes | Partial — pt-scenarios compiles and tests pass; pre-existing pt-scan lint errors block `just lint` workspace-wide |

## Scenario Dashboard

- **Before:** 87.5 min / 240.0 min (36.5%) — 10 pass, 0 fail, 4 not impl, 3 blocked
- **After:** 87.5 min / 240.0 min (36.5%) — 10 pass, 0 fail, 4 not impl, 3 blocked
- **No regressions.** S.INFRA.2 remains BLOCKED (no DATABASE_URL) but the payload fix
  removes the 422 that caused the failure.

## Test Coverage

- `cargo test -p pt-scenarios`: 2 passed, 0 failed
- `cargo check -p pt-scenarios`: compiles with no warnings
- S.INFRA.1 and S.INFRA.2 both require DATABASE_URL for live testing

## Open Concerns

1. **Pre-existing pt-scan lint errors** block `just lint` / `just check` workspace-wide.
   These are from T-033-03/04 in-progress work (derivable_impls, missing_errors_doc) and
   are not related to this ticket. They will be resolved when those tickets complete.

2. **DATABASE_URL testing** — the fix can only be fully verified with a live database.
   The casing fix is high-confidence (matches S.INFRA.1 which passes) but hasn't been
   run end-to-end in this session.
