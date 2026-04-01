# T-025-03 Plan: Claim Unclaimed Milestones

## Steps

### Step 1: Update "pt-project" milestone (line ~171)

Change `delivered_by` from `Some("T-025-02")` to `Some("T-002-01, T-003-02")`.
Rewrite note to credit T-002-01 (domain types) and T-003-02 (repo layer).

### Step 2: Update "pt-quote" milestone (line ~183)

Change `delivered_by` from `Some("T-025-02")` to `Some("T-002-02")`.
Rewrite note to credit T-002-02 (pt-quote crate).

### Step 3: Update "SvelteKit frontend" milestone (line ~315)

Change `delivered_by` from `Some("T-025-02")` to `Some("T-005-02, T-005-03")`.
Rewrite note to credit T-005-02 (CF Worker) and T-005-03 (route skeleton).

### Step 4: Update "pt-tenant" milestone (line ~325)

Change `delivered_by` from `Some("T-025-01")` to `Some("T-003-02, T-004-02")`.
Rewrite note to credit T-003-02 (TenantRepo) and T-004-02 (X-Tenant-Id extractor).

### Step 5: Run `just check`

Verify compilation, formatting, linting, tests, and scenario dashboard.
Confirm milestones remain at 19/24 with no regressions.

## Verification

- `just fmt-check` — string literal changes are format-neutral
- `just lint` — no new warnings
- `just test` — no test logic changed
- `just scenarios` — milestone count stays 19/24, effective savings ≥ 76.5 min

## Testing Strategy

No new tests needed. This is a metadata-only change. The existing scenario
dashboard validates that milestones compile and render correctly.
