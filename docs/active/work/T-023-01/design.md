# T-023-01 Design: Baseline Polish Audit

## Decision Summary

Adopt **Option A** from the ticket: pure computation scenarios (OneStar integration) get
auto-rated Polish ★★★★★. All other scenarios stay at their current ★☆☆☆☆ until
UX polish work is actually done and verified.

## Per-Scenario Rationale

### Upgrading to ★★★★★ (3 scenarios)

**S.1.1 — Scan processing** → Polish ★★★★★
- OneStar integration (pure computation). No UX surface exists.
- The computation pipeline (PLY parsing → classification → mesh → GLB/PNG) is the entire
  deliverable at this integration level. There is literally nothing to "polish."
- Polish debt at ★☆☆☆☆ = 12.0 min. Debt at ★★★★★ = 0.0 min. Recovery: 12.0 min.

**S.1.3 — Sun exposure analysis** → Polish ★★★★★
- OneStar integration (pure computation). radiance_grid() and classify() are pure functions.
- No UX to polish. Same logic as S.1.1.
- Polish debt recovery: 8.0 min.

**S.2.2 — Material catalog search and filter** → Polish ★★★★★
- OneStar integration (domain model only). Material builder, JSON contract, filtering.
- The scenario verifies the domain model in isolation. The catalog page exists (/catalog)
  but isn't tested in this scenario — that would be a TwoStar+ integration upgrade, not polish.
- Polish debt recovery: 4.0 min.

### Staying at ★☆☆☆☆ (5 scenarios)

**S.1.2 — Satellite pre-population** → Polish ★☆☆☆☆
- TwoStar integration. API returns structured data but no evidence of:
  - Loading indicators during baseline computation (spawn_blocking)
  - User-friendly error messages (programmatic error strings)
  - Empty state handling when address is unknown (graceful fallback exists but no UX)
- ★★☆☆☆ would require "loading indicators, error messages, empty state prompts."
  We can't verify these exist.

**S.2.1 — Zone drawing with measurements** → Polish ★☆☆☆☆
- TwoStar integration. Zone editor and measurements panel exist, but:
  - Milestone notes explicitly list "polish the measurements UI" as future work
  - Scenario test exercises pt-geo directly, not the UI
  - No loading states, error handling, or empty zone prompts verified
- Cannot claim ★★☆☆☆ without verifying frontend components have loading/error/empty states.

**S.2.4 — 3D preview per tier** → Polish ★☆☆☆☆
- TwoStar integration. Viewer has error protocol but:
  - Scenario validates JSON message shapes only
  - No loading spinner during scene load verified
  - No error UI rendering verified (just protocol definition)
  - Tier toggle and sunlight slider exist but "rough" per assessment
- Protocol-level error support ≠ UX polish.

**S.3.1 — Quantity computation from geometry** → Polish ★☆☆☆☆
- Currently ThreeStar integration (computation fallback path — see research.md finding).
- Even if integration were corrected to OneStar, this ticket audits current state.
- No UX layer tested in either the computation or API path.
- Filing a separate concern for the integration rating bug.

**S.3.2 — Three-tier quote generation** → Polish ★☆☆☆☆
- Same reasoning as S.3.1. ThreeStar integration via computation path.
- No UX polish verified.

## Alternatives Considered

### Option B: Rate all scenarios honestly including computation
Rejected. The ticket explicitly recommends Option A. Rating pure computation at ★☆☆☆☆
polish creates artificial "debt" that doesn't correspond to any real work needed — you
can't polish what has no UX.

### Option A with ★★★☆☆ instead of ★★★★★
The AC mentions "auto ★★★☆☆" but the Decision section says ★★★★★ for Option A.
Using ★★★★★ is correct: if there's literally no UX to critique, the UX is as good
as it needs to be at this integration level. Using ★★★☆☆ would still claim there's
"decent UX" when there's no UX at all — semantically wrong.

### Upgrade S.3.1/S.3.2 computation paths too
Rejected. Their computation paths incorrectly claim ThreeStar integration.
Option A applies only to OneStar integration. The integration bug should be fixed first
in a separate ticket, after which Option A would naturally apply.

## Impact Projection

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| S.1.1 effective | 6.0 min | 18.0 min | +12.0 |
| S.1.3 effective | 4.0 min | 12.0 min | +8.0 |
| S.2.2 effective | 2.0 min | 6.0 min | +4.0 |
| Total effective | 44.5 min | 68.5 min | +24.0 |
| Coverage | 18.5% | 28.5% | +10.0% |
| Polish debt | 62.0 min | 38.0 min | -24.0 |
