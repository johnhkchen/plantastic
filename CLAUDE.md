# CLAUDE.md

## Project

Plantastic — B2B platform for landscaping companies. Turns iPhone LiDAR scans into 3D digital twins of gardens with zone-based design, three-tier quoting, and crew-ready exports. Rust backend (Axum on Lambda), SvelteKit frontend (CF Pages), Bevy 3D viewer (WASM).

Specification: `docs/specification.md`
Testing strategy: `docs/testing-strategy.md`
Value dashboard: `cargo run -p pt-scenarios`
Quality gate: `just check` (format + lint + test + scenarios)

### Key Commands

```
just check       # Full pre-commit gate — run before marking any ticket done
just test        # Workspace tests with 60s binary timeout
just scenarios   # Value delivery dashboard (the number that matters)
just scenarios-db # Scenarios with Docker Compose Postgres (starts DB if needed)
just lint        # Clippy strict — warnings are errors
just fmt         # Auto-format
just status      # Lisa ticket DAG
```

### Directory Conventions

```
docs/active/tickets/    # Ticket files (markdown with YAML frontmatter)
docs/active/stories/    # Story files (same frontmatter pattern)
docs/active/epics/      # Epic files
docs/active/work/       # Work artifacts, one subdirectory per ticket ID
tests/scenarios/        # Value delivery scenario harness
```

---

The RDSPI workflow definition is in docs/knowledge/rdspi-workflow.md and is injected into agent context by lisa automatically.

---

## Testing Philosophy

This project measures progress by **verified customer value**, not by code coverage or test count. The scenario harness (`cargo run -p pt-scenarios`) reports how many minutes of real time savings the system delivers end-to-end. That number is the ground truth. Everything below serves it.

### The rules

**1. A ticket is not done if its scenario tests don't pass.**

Unit tests passing is necessary but not sufficient. If your ticket's design.md references scenario S.3.1 and S.3.1 still returns `NotImplemented` or `Fail` after your work, the ticket is not done. Check the scenario harness before marking complete.

**2. Do not write tests that confirm your own assumptions.**

When a test computes an expected value, that value must be derived independently from the code under test. If you're testing that a 12×15 ft patio at $8.50/sq_ft costs $1,530.00, the number $1,530.00 is arithmetic you do in the test — you do not call `pt_geo::area()` to get 180 and then multiply. The system and the test must arrive at the same answer from different directions.

Bad:
```rust
let area = pt_geo::area(&patio_polygon);  // system computes
let expected = area * 8.50;                // test trusts system
assert_eq!(quote.line_items[0].total, expected);
```

Good:
```rust
// 12 × 15 = 180 sq ft. 180 × $8.50 = $1,530.00.
// Computed here, not by pt_geo.
let expected_total = rust_decimal::Decimal::from_str("1530.00").unwrap();
assert_eq!(quote.line_items[0].line_total, expected_total);
```

**3. Do not mock across crate boundaries.**

If pt-quote calls pt-geo, the test uses the real pt-geo. If the API calls the repository, the test uses a real Postgres database. Mocks hide integration failures — the exact class of bug that costs the most time to find later. The only acceptable mock boundary is external third-party services (Plant.id, S3) where a real call would be impractical in CI.

**4. Do not use `#[ignore]` without a scenario ID.**

Every `#[ignore]` annotation must include a comment referencing the scenario or ticket that will un-ignore it. Bare `#[ignore]` is how tests go permanently dark.

Bad:
```rust
#[ignore]  // TODO: fix later
```

Good:
```rust
#[ignore = "Blocked on pt-scan crate (S.1.1), tracked in T-007-02"]
```

**5. No stat-padding tests.**

Do not write tests that pass trivially, test internal implementation details rather than behavior, or duplicate existing assertions in slightly different form to inflate test counts. Every test should fail if the capability it claims to verify is broken. If you can delete a test and no real capability is left unverified, that test shouldn't exist.

Questions to ask before writing a test:
- If this function were wrong, would this test catch it?
- Does this test verify something a user would notice, or just an internal detail?
- Is there already a test that covers this case?

**6. Own what you find.**

If you encounter a bug, test failure, or broken behavior near your working area — even if you didn't introduce it — you own it. Do not hand-wave "that's a pre-existing issue" or "that regression is outside my ticket scope." If you found it, one of two things must happen:

- **Fix it** if it's within reach (same crate, adjacent code, <30 min of work).
- **File a ticket** with a failing test that reproduces the bug. The test is the minimum — it ensures the bug is visible, tracked, and can't be silently ignored. Put the ticket in `docs/active/tickets/` with status `open`, reference the failing test, and note what you observed.

The reason: bugs near your silo are bugs that will bite the next person working in your silo. Leaving them undocumented is leaving a trap. A failing test that says "this is broken" is infinitely more useful than nothing.

**7. Run the scenario dashboard before and after your work.**

At the start of your ticket: `cargo run -p pt-scenarios` — note the baseline. At the end: run it again. Your review.md must include the before and after. If the number went down, something regressed and your ticket is not done until it's fixed. If the number didn't go up and your ticket was supposed to advance a capability, explain why in review.md.

**8. Integration tests use real infrastructure.**

Database tests hit a real Postgres instance. API tests make real HTTP requests. Frontend scenario tests (when they exist) hit the real API. SQLite substitutes, in-memory fakes, and mock servers are not acceptable for integration tests in this project. If standing up real infrastructure in CI is hard, that's a problem to solve in infrastructure — not a reason to weaken the tests.

**9. Tests must be fast. Slow tests must justify themselves.**

This is Rust. Compute is fast. A test taking more than 10 seconds is almost certainly blocked on I/O (database, network, subprocess, file system), not doing useful computation. Slow tests are a sign of either a real problem or a missing optimization.

For any test that touches domain logic (pt-geo, pt-quote, pt-solar, etc.), wrap the body with `pt_test_utils::timed()` to enforce a 10-second timeout:

```rust
use pt_test_utils::timed;

#[test]
fn quote_computes_correctly() {
    timed(|| {
        // test body — if this takes >10s, the test panics with diagnostics
    });
}
```

If a test genuinely needs more than 10 seconds (e.g., database integration with setup/teardown), use `run_with_timeout` with an explicit duration and a comment explaining why:

```rust
use pt_test_utils::run_with_timeout;
use std::time::Duration;

#[test]
fn full_project_round_trip() {
    // 30s: includes Postgres container startup + migration + teardown
    run_with_timeout(Duration::from_secs(30), || {
        // ...
    });
}
```

The `just test` command enforces a 120-second hard timeout on the entire test suite. If the suite exceeds this, something is hung — not slow, hung. Fix it.

**10. `just check` before marking done.**

Run `just check` (format + lint + test + scenarios) before marking any ticket as complete. This is the quality gate. If it doesn't pass, the ticket isn't done. The four checks are:

- `just fmt-check` — code is formatted
- `just lint` — clippy strict, warnings are errors
- `just test` — all workspace tests pass within timeout
- `just scenarios` — value dashboard runs, no regressions

### How to register a new scenario

When your ticket introduces a new customer-facing capability:

1. Add a `Scenario` entry to the appropriate suite in `tests/scenarios/src/suites/`.
2. Set the `time_savings_minutes` based on the Value Map in `docs/testing-strategy.md`.
3. Start with `ScenarioOutcome::NotImplemented` if the capability isn't ready yet.
4. Replace with real assertions when the capability is implemented.
5. Once a scenario passes, it must stay passing. A regression in a passing scenario is a blocking issue.

### How to claim a milestone

When your ticket delivers foundational capability (a crate, a service, an infrastructure piece):

1. Open `tests/scenarios/src/progress.rs`.
2. Find the milestone that matches your work (or add a new one if none exists).
3. Set `delivered_by` to your ticket ID: `delivered_by: Some("T-001-02")`.
4. Write a `note` that explains:
   - What you delivered (be specific: name the functions, types, or endpoints)
   - What scenarios this unblocks or advances
   - What's still needed before those scenarios can turn green
5. If your work doesn't map to an existing milestone, add one with the correct `unlocks` list.

This is how foundational engineering work becomes visible. The dashboard shows your contribution even when no scenario flips to green yet. The milestone tracker answers: "which pieces of the pipeline are built, and what's still missing?"

### Summary

Green unit tests are table stakes, not proof of delivery. The scenario dashboard is the honest scoreboard. Every agent session should leave the verified savings the same or higher — never lower. Foundational work that doesn't directly flip a scenario should claim its milestone so the engineering progress is visible.
