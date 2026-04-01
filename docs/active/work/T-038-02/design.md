# T-038-02 Design: Fix Tenant Isolation Scenario

## Decision

Fix the test payloads. The API is correct; the S.INFRA.2 test sends the wrong casing.

## Options Considered

### Option A: Fix test payloads (chosen)

Change `"Patio"` → `"patio"` and `"Bed"` → `"bed"` in S.INFRA.2 steps 5 and 6.

**Pros:** Minimal change, matches S.INFRA.1 which already works, tests the actual
isolation behavior instead of accidentally testing validation.

**Cons:** None.

### Option B: Make ZoneType case-insensitive

Add a custom deserializer or `#[serde(alias = "Patio")]` to accept both casings.

**Rejected:** The API has a clear contract (`snake_case`). Making deserialization
permissive to accommodate a broken test is backwards. S.INFRA.1 already demonstrates the
correct format. The frontend and any real clients use the documented format.

## Additional Changes

### Error body logging

Per acceptance criteria, add response body text to failure messages throughout S.INFRA.2.
Current:
```rust
format!("POST /zones as Tenant A: expected 201, got {status}")
```

After:
```rust
format!("POST /zones as Tenant A: expected 201, got {status}: {body}")
```

This makes future validation failures self-diagnosing without needing to debug.

### Apply same pattern to S.INFRA.1

S.INFRA.1 also omits response bodies in failure messages. Apply the same improvement
there for consistency, since both scenarios are in the same file and benefit equally.

## Scenario Impact

- S.INFRA.2 should pass at ★★☆☆☆ Integration / ★☆☆☆☆ Polish with DATABASE_URL
- No impact on `just check` (no DATABASE_URL in CI, both scenarios stay BLOCKED)
- No code changes outside the test file
