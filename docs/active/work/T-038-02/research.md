# T-038-02 Research: Fix Tenant Isolation Scenario

## Problem Statement

S.INFRA.2 (tenant isolation) fails at Step 5: "POST /projects/:id/zones as Tenant A:
expected 201, got 422 Unprocessable Entity." The API's validation rejects the zone
creation payload because the `zone_type` field uses wrong casing.

## Relevant Files

| File | Role |
|------|------|
| `tests/scenarios/src/suites/infrastructure.rs` | S.INFRA.1 and S.INFRA.2 scenario definitions |
| `crates/plantastic-api/src/routes/zones.rs` | Zone POST handler + `AddZoneRequest` struct |
| `crates/pt-project/src/types.rs` | `ZoneType` enum with serde rename |

## Root Cause Analysis

### The ZoneType enum (pt-project/src/types.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneType {
    Bed,       // serializes as "bed"
    Patio,     // serializes as "patio"
    Path,      // serializes as "path"
    Lawn,      // serializes as "lawn"
    Wall,      // serializes as "wall"
    Edging,    // serializes as "edging"
}
```

`rename_all = "snake_case"` means deserialization expects lowercase: `"patio"`, `"bed"`, etc.

### S.INFRA.1 (works) — line 118-122

```rust
Some(json!({
    "geometry": patio_geojson,
    "zone_type": "patio",     // ← lowercase, correct
    "label": "Back patio"
})),
```

### S.INFRA.2 (fails) — line 441-445

```rust
Some(json!({
    "geometry": zone_geojson,
    "zone_type": "Patio",     // ← PascalCase, WRONG
    "label": "Test Patio"
})),
```

### S.INFRA.2 Step 6 — line 465-468

```rust
Some(json!({
    "geometry": zone_geojson,
    "zone_type": "Bed",       // ← PascalCase, WRONG (should be "bed")
    "label": "Intruder Zone"
})),
```

Step 5 sends `"Patio"` → serde rejects → 422. The test expected 201 and fails. Step 6
sends `"Bed"` but never runs because Step 5 already failed. Step 6 expects 404 (tenant
isolation) but would also get 422 first.

## Error Message Quality

The current failure messages (lines 453-455, 476-478) do not include the response body:

```rust
"POST /projects/{project_id}/zones as Tenant A: expected 201, got {status}"
```

The ticket's acceptance criteria require adding body logging so future 422s are
self-diagnosing.

## Scope

The fix is two lines in the test file (change `"Patio"` → `"patio"`, `"Bed"` → `"bed"`),
plus improving error messages to include the response body. The API code is correct — only
the test payloads need fixing.
