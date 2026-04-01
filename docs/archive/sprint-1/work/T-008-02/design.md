# Design — T-008-02: Quote Scenario Two-Star

## Decision: Inline async block with tokio Runtime

### Options Considered

**Option A: Extract test helpers into pt-test-utils**
Move `test_pool()`, `setup_test_db()`, `create_test_tenant()`, `test_router()`, `send()`
from `crates/plantastic-api/tests/common/mod.rs` into `crates/pt-test-utils/src/lib.rs`.
Both the API integration tests and scenarios import from there.

- Pro: Single source of truth for test infrastructure.
- Con: Adds axum/sqlx/tower deps to pt-test-utils, which currently only handles timeouts.
  Pollutes a focused crate. The API test helpers are specific to the API, not general.
- **Rejected:** Scope creep. pt-test-utils is for compute test utilities.

**Option B: Create a new pt-api-test-utils crate**
Dedicated crate for API test infrastructure, imported by both API tests and scenarios.

- Pro: Clean separation.
- Con: One more crate for 5 small functions. Premature abstraction for 2 consumers.
- **Rejected:** YAGNI. If a third consumer appears, revisit.

**Option C: Duplicate helpers in scenarios crate (chosen)**
Write equivalent helpers directly in the scenarios crate (a small module, ~80 lines).
The helpers mirror the API test common module but live in the scenarios crate.

- Pro: No cross-crate coupling. Scenarios are self-contained. Helpers can diverge
  (e.g., scenarios might want different error handling than API tests).
- Con: Duplication of ~80 lines.
- **Chosen:** The duplication is small, the helpers are simple, and the consumers
  have different error handling needs (API tests panic, scenarios return Blocked/Fail).

### Async Runtime Strategy

Scenario functions are `fn() -> ScenarioOutcome` (sync). API calls are async.

**Approach:** Create a single-threaded tokio runtime inside each scenario function.
```rust
let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();
rt.block_on(async { ... })
```

Using `new_current_thread` rather than `new_multi_thread` to minimize overhead
(scenarios are sequential, no parallelism needed within a single scenario).

### DATABASE_URL Handling

If DATABASE_URL is not set, return `ScenarioOutcome::Blocked("DATABASE_URL not set")`
instead of panicking. This keeps `just scenarios` runnable without Postgres — the
dashboard will show the scenarios as blocked rather than failing.

### Old Test Preservation

The acceptance criteria says "Old OneStar tests preserved as unit-level regression tests."

**Approach:** Rename old functions to `s_3_1_unit_regression()` and `s_3_2_unit_regression()`.
Keep them in `quoting.rs` as private functions. The new API-based functions become the
scenario entries. The regression functions are called from new `#[cfg(test)]` unit tests
in the same file, so they still compile and run during `cargo test`.

This is better than moving them to another file because:
1. They stay colocated with the scenario they originally validated
2. They run with `cargo test -p pt-scenarios`
3. The diff is minimal — rename + add `#[cfg(test)]` test wrappers

### Assertion Strategy

The API returns Decimal values as strings (e.g., `"1530.00"`). Assertions will:
1. Parse the JSON response
2. Compare string values directly for exact decimal match
3. For Good < Better < Best, parse to `Decimal` and use `<` operator

This avoids float precision issues entirely.

### Verification Scope

S.3.1 TwoStar assertions (same arithmetic as OneStar, but from API response):
- 3 line items
- Patio line_total = "1530.00"
- Mulch line_total = "88.89"
- Edging line_total = "130.00"
- subtotal = "1748.89"
- total = "1748.89" (no tax)

S.3.2 TwoStar assertions:
- Good < Better < Best (parse totals to Decimal)
- subtotal == sum(line_totals) for each tier (from API response)
- Exact totals: Good = "839.26", Best = "3148.40"
- 3 line items per tier
- No duplicate zone_id within a tier

### Milestones

The acceptance criteria mentions claiming "Axum API: routes + Lambda deployment" and
"PostGIS schema + sqlx repository layer" if not already claimed. Both are already claimed
(T-004-02 and T-003-02 respectively). No milestone changes needed.

---

## Summary of Changes

1. Add async dependencies to `tests/scenarios/Cargo.toml`
2. Add API test helper module to `tests/scenarios/src/api_helpers.rs`
3. Rewrite S.3.1 and S.3.2 in `quoting.rs` to use API path
4. Preserve old test logic as `#[cfg(test)]` unit tests
5. No milestone changes (already claimed)
