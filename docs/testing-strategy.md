# Plantastic — Testing Strategy & Value Delivery Framework

## The Problem This Solves

LLM-assisted development has a consistent failure mode: agents report "done" based on code existence and self-authored test passage, while actual functionality is partial, untested against real inputs, or broken at integration boundaries. Green unit tests become a false signal that masks incomplete delivery.

A subtler failure: a computation engine passes all its tests but no user can reach it. The code is correct but the value is unrealized. Claiming "25 minutes saved" when the capability exists only as a library function with no API or UI is misleading.

This testing strategy addresses both problems by tying every test to a measurable customer outcome weighted by how deeply integrated it is into the product.

-----

## Value Map

Plantastic claims to reduce the design-to-proposal cycle from ~4 hours to ~30 minutes. That claim breaks down into four capability areas, each with a measurable time savings target. Every scenario test maps to one of these areas.

### Area 1: Site Assessment — saves ~90 minutes

Replaces: manual measurement, site visits for data gathering, looking up sun exposure and climate data, identifying existing plants.

| Capability | Time Saved | Scenario | Integration |
|-----------|-----------|----------|-------------|
| Scan processing (PLY → terrain + plan view) | 30 min | S.1.1 | — |
| Satellite pre-population (address → lot, trees, sun) | 25 min | S.1.2 | — |
| Sun exposure analysis (solar radiance grid) | 20 min | S.1.3 | — |
| Plant identification (Plant.id integration) | 15 min | S.1.4 | — |

### Area 2: Design — saves ~60 minutes

Replaces: sketching on paper, verbal descriptions, revision rounds, flipping through plant catalogs, guessing what grows where.

| Capability | Time Saved | Scenario | Integration |
|-----------|-----------|----------|-------------|
| Zone drawing with live measurements | 20 min | S.2.1 | — |
| Material catalog with search and filtering | 10 min | S.2.2 | — |
| Plant recommendations (AI-powered, site-specific) | 20 min | S.2.3 | — |
| 3D preview per tier | 10 min | S.2.4 | — |

### Area 3: Quoting — saves ~60 minutes

Replaces: manual quantity takeoff, spreadsheet pricing, formatting proposals, generating PDFs, three separate price calculations for good/better/best.

| Capability | Time Saved | Scenario | Integration |
|-----------|-----------|----------|-------------|
| Automatic quantity computation from geometry | 25 min | S.3.1 | ★☆☆☆☆ |
| Three-tier quote generation | 15 min | S.3.2 | ★☆☆☆☆ |
| Branded PDF export | 10 min | S.3.3 | — |
| Client-facing quote comparison view | 10 min | S.3.4 | — |

### Area 4: Crew Handoff — saves ~30 minutes

Replaces: redrawing the design for the crew, printing material lists, verbal walkthroughs of what goes where.

| Capability | Time Saved | Scenario | Integration |
|-----------|-----------|----------|-------------|
| 3D viewer on tablet (approved plan) | 15 min | S.4.1 | — |
| DXF export for CAD tools | 10 min | S.4.2 | — |
| Material callouts with supplier SKU and depth | 5 min | S.4.3 | — |

-----

## Integration Rating

A passing scenario test proves the computation is correct. The integration rating captures how reachable that capability is by a real user. Time savings are weighted by integration level.

**Effective minutes = raw minutes × (stars / 5)**

A 25-minute capability at ★☆☆☆☆ contributes 5 effective minutes. The same capability at ★★★★★ contributes the full 25. This means integration work — wiring an engine to an API, building a UI, polishing for production — directly increases the effective savings number without any new computation work.

| Rating | Meaning | Example |
|--------|---------|---------|
| ★☆☆☆☆ | Pure computation works in isolation. No API, no UI, no persistence. "The engine runs but no user can reach it." | pt-quote computes correct line items from in-memory structs |
| ★★☆☆☆ | Reachable via API but no UI. Could test with curl. "A developer can use it, a landscaper can't." | GET /projects/:id/quote/good returns correct JSON |
| ★★★☆☆ | API + basic UI exists. Functional but rough. "A landscaper could use it with hand-holding." | Dashboard shows quotes, but no material assignment UI yet |
| ★★★★☆ | Polished UI, persisted, handles errors. Missing some edge cases. "A landscaper could use it in a demo." | Full quote flow works: draw zones → assign materials → see 3-tier comparison |
| ★★★★★ | Production-ready. Branded, reliable, handles edge cases, tested on real data. "A landscaper uses it daily to win contracts." | Branded PDF exports, client-facing view, works on mobile |

### How integration level advances

Each star corresponds to a layer of the stack being wired up:

1. **★ → ★★**: Wire the computation to an API route. The capability becomes reachable over HTTP.
2. **★★ → ★★★**: Build a basic UI that calls the API. The capability becomes usable by a non-developer.
3. **★★★ → ★★★★**: Polish the UI, add error handling, persist state, handle edge cases. The capability becomes demo-ready.
4. **★★★★ → ★★★★★**: Brand it (tenant logo/colors), test on real data, verify on target devices (iPad for crew), harden for production load.

Agents set the integration level when they return `ScenarioOutcome::Pass(Integration::TwoStar)` (or whichever level). The level should be honest — don't claim ★★★ if there's no UI, don't claim ★★ if the API route doesn't exist yet.

-----

## Engineering Milestones

Not all work directly flips a scenario to green. Foundational engineering — setting up the workspace, writing domain crates, configuring infrastructure — is critical but doesn't show up in the "effective savings" number.

The milestone tracker (`tests/scenarios/src/progress.rs`) makes this work visible. Each milestone represents a piece of foundational capability that unlocks one or more scenarios. When an agent completes a ticket that delivers a milestone, they claim it with their ticket ID and a note explaining what was delivered.

The dashboard shows:
- How many milestones are delivered out of the total
- Which scenarios each milestone unlocks
- What's next in the pipeline

This ensures that an agent working on pt-geo (foundational, no direct user value) sees their contribution reflected alongside the agent who later wires pt-geo into a quote route (direct user value).

-----

## Testing Layers

### Layer 1: Unit Tests

**Written by:** implementing agent, during Implementation phase
**Location:** `crates/*/src/**/*_test.rs` or `#[cfg(test)]` modules
**Purpose:** fast feedback, regression catching, documenting function behavior
**Trust level:** necessary but not sufficient — these share the agent's assumptions

Conventions:
- Every public function has at least one test
- Pure computation crates (pt-geo, pt-quote, pt-solar) have extensive property-based tests
- Tests use known inputs with independently verifiable expected outputs
- No `#[ignore]` without a scenario ID or ticket reference explaining when it will be un-ignored
- No mocking of adjacent crates — if pt-quote calls pt-geo, the test uses the real pt-geo

### Layer 2: Integration Tests

**Written by:** implementing agent, during Implementation phase
**Location:** `crates/*/tests/` or `apps/api/tests/`
**Purpose:** verify that components compose correctly across crate boundaries
**Trust level:** higher than unit tests because they exercise real interfaces

Conventions:
- Database integration tests use a real Postgres instance (not mocks, not SQLite)
- API integration tests make real HTTP requests to the Axum router
- Tests that require external services (Postgres, S3) are gated behind a feature flag or env var, not `#[ignore]`
- Every repository function is tested with a round-trip: write → read → assert equality

### Layer 3: Scenario Tests (the value delivery layer)

**Written by:** human or designated agent during Design phase, before implementation begins
**Location:** `tests/scenarios/`
**Purpose:** prove that customer-facing capabilities work end-to-end
**Trust level:** high — these are the acceptance gate

Each scenario test:
1. Is tagged with its Value Map ID (e.g., `S.3.1`)
2. Exercises a complete user workflow, not an isolated function
3. Computes expected outputs independently from the system under test
4. Crosses crate boundaries
5. Asserts on customer-visible outcomes, not internal state
6. Returns `ScenarioOutcome::Pass(Integration::XStar)` with an honest integration level

### Layer 4: Smoke Tests (deployment verification)

**Written by:** during infrastructure tickets
**Location:** `tests/smoke/` and `scripts/verify-deploy.sh`
**Purpose:** verify that deployed infrastructure is alive and connected
**Trust level:** binary — either the stack is up or it isn't

-----

## Conventions for Agents

### During Design Phase (RDSPI)

When writing design.md for a ticket, identify which scenario tests the ticket's output must satisfy. Reference them by ID. If the ticket advances a scenario's integration level, note the expected before/after star rating.

### During Implementation Phase

1. Write unit and integration tests as normal.
2. Before marking a ticket done, run `just scenarios` and compare against baseline.
3. If a scenario test regresses, the ticket is not done.
4. If your work advances a scenario's integration level, update the `Integration::XStar` in the scenario's return value.
5. If your work delivers a milestone, claim it in `progress.rs`.

### During Review Phase

The review.md artifact must include:
- `just scenarios` output (before and after)
- Which scenarios advanced in star rating and why
- Which milestones were claimed
- The effective savings number after this ticket

### Rules

1. **No mocking across crate boundaries in scenario tests.**
2. **Expected values are computed in the test, not extracted from the system.**
3. **Scenario tests are append-only.** Once a scenario passes, it must keep passing.
4. **Integration levels are honest.** Don't claim ★★ if there's no API route. Don't claim ★★★ if there's no UI.
5. **Star ratings only go up.** A scenario dropping from ★★★ to ★★ is a regression.
6. **The Value Map is updated by humans** during Review phase with human sign-off.

-----

## Reporting

### Quick Status

```bash
just scenarios       # value delivery dashboard with integration ratings
just check           # full quality gate (format + lint + test + scenarios)
```

### Reading the Dashboard

The dashboard shows three key numbers:

1. **Effective savings** — raw time savings weighted by integration level. This is the honest number.
2. **Raw passing** — unweighted time savings for all passing scenarios. The gap between this and effective savings is the integration debt.
3. **Milestones** — foundational engineering progress. Shows what's been built and what's next.

Example reading: "Effective savings: 8.0 / 240.0 min (3.3%). Raw passing: 40.0 min. Two quoting scenarios pass at ★☆☆☆☆ — the engine works but it's not reachable by users. 0/18 milestones claimed. Next priority: wire quote engine to API (★→★★) and build basic quote UI (★★→★★★)."

The gap between raw and effective is a concrete measure of integration debt. Closing that gap — by building API routes, UI pages, and deployment infrastructure — is how raw capability becomes real user value.
