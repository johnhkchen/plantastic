# T-023-01 Research: Baseline Polish Audit

## Current State

8 passing scenarios, all at Polish ★☆☆☆☆ (OneStar). The polish dimension was
added by T-022-01. This ticket audits each scenario's polish rating against the
rubric and adjusts where warranted.

## Polish Rubric (from registry.rs:103-145)

| Rating    | Definition                                                  |
|-----------|-------------------------------------------------------------|
| ★☆☆☆☆    | Bare computation works. No UX consideration.                |
| ★★☆☆☆    | Basic UI exists, functional but rough.                      |
| ★★★☆☆    | Decent UX, handles common paths well.                       |
| ★★★★☆    | Polished UX, error handling, edge cases covered.            |
| ★★★★★    | Production-quality UX, delightful, tested on real users.    |

## Formula

`effective_minutes = raw_minutes × (integration_stars + polish_stars) / 10`

Polish debt = `raw_minutes × (5 - polish_stars) / 10` per passing scenario.

Current polish debt: 62.0 min recoverable.

## Passing Scenario Inventory

### Pure Computation (Integration ★☆☆☆☆)

**S.1.1 — Scan processing** (30 min, site_assessment.rs:199)
- Integration: OneStar. Pure computation: PLY → classified point cloud → terrain mesh + GLB + PNG.
- No API, no UI, no persistence. Tests pt_scan::process_scan() and generate_terrain() directly.
- Current polish: ★☆☆☆☆. There is no UX to polish — this is a pure compute pipeline.

**S.1.3 — Sun exposure analysis** (20 min, site_assessment.rs:426)
- Integration: OneStar. Pure computation: location + date range → radiance grid + classifications.
- Tests pt_solar::radiance_grid() and classify() directly.
- Current polish: ★☆☆☆☆. No UX surface exists; pure math functions.

**S.2.2 — Material catalog search and filter** (10 min, design.rs:355)
- Integration: OneStar. Domain model verified in isolation: Material builder, JSON serialization,
  category filtering, name search, combined filters.
- Tests pt_materials types directly. Catalog CRUD page exists at /catalog (SvelteKit) per
  milestone notes, but the scenario test doesn't exercise it.
- Current polish: ★☆☆☆☆. The scenario tests pure domain model, not UI.

### API-Accessible (Integration ★★☆☆☆)

**S.1.2 — Satellite pre-population** (25 min, site_assessment.rs:335)
- Integration: TwoStar. BaselineBuilder wired into POST /projects (T-011-02).
  JSON round-trip tested to verify JSONB storage compatibility.
- No UI for displaying baseline data. API returns structured ProjectBaseline
  (coordinates, lot_boundary, trees, sun_grid) but no error message UX.
- Current polish: ★☆☆☆☆.

**S.2.1 — Zone drawing with measurements** (20 min, design.rs:136)
- Integration: TwoStar. API route exists (T-004-02), zone editor UI (ZoneEditor.svelte)
  exists with measurements panel (T-007-02).
- Scenario test exercises pt-geo directly, not the API or UI. Tests area_sqft() and
  perimeter_ft() against hand-computed values.
- Milestone notes: "polish the measurements UI" listed as path to ThreeStar.
  No loading indicators, error messages, or empty state prompts verified.
- Current polish: ★☆☆☆☆.

**S.2.4 — 3D preview per tier** (10 min, design.rs:477)
- Integration: TwoStar. Bevy viewer embedded in SvelteKit via iframe with typed
  postMessage protocol. Has orbit camera, tier switching, sunlight slider (T-014-02).
- Scenario test validates JSON protocol shapes only, not actual rendering or UX.
- Protocol includes "error" outbound message type with message field.
  "ready" message confirms scene loaded. No loading spinner or error UI verified.
- Current polish: ★☆☆☆☆.

### API with Computation Fallback (Integration ★★★☆☆ via computation path)

**S.3.1 — Quantity computation from geometry** (25 min, quoting.rs:826/348)
- Without DATABASE_URL: computation-only path returns Integration::ThreeStar (line 826).
- With DATABASE_URL: API path returns Integration::TwoStar (line 348).
- **Finding**: computation path claims ThreeStar integration but only calls compute_quote()
  directly — this should be OneStar. The API path at TwoStar is the correct higher rating.
  This is a pre-existing bug in integration rating, not polish.
- No UX layer tested in either path. Scenario verifies arithmetic correctness.
- Current polish: ★☆☆☆☆.

**S.3.2 — Three-tier quote generation** (15 min, quoting.rs:1054/653)
- Same architecture as S.3.1. Computation path at ThreeStar (line 1054), API at TwoStar (653).
- Same integration rating bug as S.3.1.
- Scenario verifies Good < Better < Best ordering and exact totals.
- No UX layer tested.
- Current polish: ★☆☆☆☆.

## Ticket Decision: Option A

The ticket recommends Option A: "Polish is N/A for computation-only scenarios → auto-rate
at ★★★★★ (no UX to critique)." This applies to ★☆☆☆☆ integration scenarios only.
"Polish becomes meaningful at ★★☆☆☆+ where a user or developer interacts with an interface."

## Key Files

| File | Lines | What to change |
|------|-------|----------------|
| `tests/scenarios/src/suites/site_assessment.rs` | 199, 426 | S.1.1 and S.1.3 polish |
| `tests/scenarios/src/suites/design.rs` | 355 | S.2.2 polish |
| `tests/scenarios/src/suites/quoting.rs` | 826, 1054 | Not changing (ThreeStar integration, not OneStar) |
| `tests/scenarios/src/registry.rs` | 103-145 | Polish enum definition (no change needed) |
| `tests/scenarios/src/report.rs` | 46-62 | Polish debt calculation (no change needed) |

## Open Finding: S.3.1/S.3.2 Integration Rating Bug

The computation fallback paths for S.3.1 and S.3.2 return `Integration::ThreeStar` but
they only test `compute_quote()` directly — no API, no UI. This should be OneStar by
the integration rubric definition. If corrected, Option A would apply to those paths too.
This is out of scope for T-023-01 (polish audit) and should be a separate ticket.
