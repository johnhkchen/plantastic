# T-038-02 Plan: Fix Tenant Isolation Scenario

## Steps

### Step 1: Fix zone_type casing in S.INFRA.2

Change line 443 `"Patio"` → `"patio"` and line 468 `"Bed"` → `"bed"`.

**Verification:** `just lint` passes (no unused variable warnings from body captures).

### Step 2: Add response body to S.INFRA.2 failure messages

For each status-check failure in `s_infra_2_api()`, include `{body}` in the format string.
Where body is currently discarded with `_`, capture it as `body` or `_body` (for steps
that check non-200 status and don't need the body content, but should still log it on
unexpected status).

Steps affected:
- Step 1 (POST /projects): already has `body`, just add to format string
- Step 2 (GET /projects): captures `_`, change to capture body for error msg
- Step 3 (POST /materials): already has `mat_body`, use in format string
- Step 4 (GET /materials): already has `materials`, use in format string
- Step 5 (POST /zones): captures `_`, change to capture body
- Step 6 (POST /zones as B): captures `_`, change to capture body
- Step 7 (PUT /tiers as B): captures `_`, change to capture body

### Step 3: Add response body to S.INFRA.1 failure messages

Same pattern for `s_infra_1_api()`. Lower priority but same file, easy to do.

### Step 4: Verify

- `just fmt` — auto-format
- `just lint` — no warnings
- `just test` — all tests pass
- `just scenarios` — no regressions (INFRA scenarios stay BLOCKED without DATABASE_URL)

## Testing Strategy

- **Primary:** `just check` must pass without DATABASE_URL
- **With DATABASE_URL:** S.INFRA.2 should pass at ★★☆☆☆/★☆☆☆☆
- No new tests needed — this fixes an existing test
