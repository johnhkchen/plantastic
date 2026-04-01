# T-038-02 Structure: Fix Tenant Isolation Scenario

## Files Modified

### `tests/scenarios/src/suites/infrastructure.rs`

This is the only file that needs changes.

#### Change 1: Fix zone_type casing in S.INFRA.2 Step 5 (line 443)

```
- "zone_type": "Patio"
+ "zone_type": "patio"
```

#### Change 2: Fix zone_type casing in S.INFRA.2 Step 6 (line 468)

```
- "zone_type": "Bed"
+ "zone_type": "bed"
```

#### Change 3: Add response body to S.INFRA.2 failure messages

All `ScenarioOutcome::Fail` messages that report HTTP status codes should include the
response body. Affected steps: 1, 2, 3, 4, 5, 6, 7.

Pattern:
```rust
// Before:
format!("POST /projects as Tenant A: expected 201, got {status}")

// After:
format!("POST /projects as Tenant A: expected 201, got {status}: {body}")
```

For steps that discard body with `_`, capture it instead.

#### Change 4: Add response body to S.INFRA.1 failure messages

Same pattern applied to S.INFRA.1 for consistency. Affected steps: 1–9.

## Files NOT Modified

- No API code changes
- No domain type changes
- No new files created
- No scenario registration changes (S.INFRA.2 already registered)
