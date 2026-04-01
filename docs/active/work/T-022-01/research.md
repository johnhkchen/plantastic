# T-022-01 Research: Polish Enum + Dashboard

## Current State

### ScenarioOutcome enum (registry.rs:103-114)

`ScenarioOutcome` has four variants:
- `Pass(Integration)` — single-arg tuple with integration level
- `Fail(String)` — failure message
- `NotImplemented` — stub
- `Blocked(String)` — blocked reason

The `Pass` variant is the only one that changes in this ticket.

### Integration enum (registry.rs:55-99)

Five-level star rating: `OneStar` through `FiveStar`. Methods:
- `stars(self) -> u8` — returns 1–5
- `weight(self) -> f64` — stars / 5.0
- `label(self) -> &'static str` — renders star display like `★★☆☆☆`

### effective_minutes formula (registry.rs:140-145)

```rust
ScenarioOutcome::Pass(level) => raw_minutes * level.weight()
```

Current: `raw × (stars / 5)`.
Ticket requires: `raw × (integration.stars() + polish.stars()) / 10.0`.

This means the maximum possible effective minutes stays the same:
`raw × (5 + 5) / 10 = raw × 1.0`. But with two dimensions, the weights are
additive — a 1-star integration + 1-star polish = 2/10 = 0.2× instead of
the current 1/5 = 0.2×. So at the OneStar/OneStar level the weight is
identical to the current OneStar weight. At FiveStar/OneStar it would be
6/10 = 0.6× instead of the current 5/5 = 1.0×. This is a meaningful
formula change that will reduce effective minutes for scenarios that are
currently FiveStar integration but OneStar polish. Currently no scenarios
are above ThreeStar, so impact is moderate.

### status_label format (registry.rs:126-133)

Currently: `PASS ★★★☆☆`
Ticket requires: `PASS ★★★☆☆ / ★☆☆☆☆`

### Match sites for Pass variant

10 `Pass(Integration::X)` return sites across suite files:
- `site_assessment.rs`: lines 199, 335, 426 (3 sites)
- `design.rs`: lines 136, 355, 477 (3 sites)
- `quoting.rs`: lines 348, 653, 826, 1054 (4 sites)

2 `matches!(outcome, ScenarioOutcome::Pass(_))` sites in `quoting.rs`:
lines 1065, 1074 — these use wildcard pattern, no change needed.

4 `ScenarioOutcome::Pass(_)` match arms in `registry.rs` and `report.rs`:
- `registry.rs:119` — symbol() — wildcard, no change needed
- `registry.rs:128` — status_label() — destructures `level`, needs second field
- `registry.rs:136` — counts_as_delivered() — wildcard, no change needed
- `registry.rs:142` — effective_minutes() — destructures `level`, needs second field

### Dashboard rendering (report.rs)

`print_dashboard()` and `print_area_section()` call `effective_minutes()` and
`status_label()` through the `ScenarioOutcome` methods. The dashboard header
shows "Effective savings" and "Raw passing". No legend or formula explanation
currently exists.

### Existing tests

No unit tests exist for `Integration`, `ScenarioOutcome`, or the formula.
All verification is through the scenario harness itself.

## Impact Analysis

### Breaking change scope

Changing `Pass(Integration)` to `Pass(Integration, Polish)` is a breaking
enum variant change. Every `match` arm and constructor site needs updating.
Wildcard patterns (`Pass(_)`) are unaffected.

Sites requiring changes:
1. **registry.rs**: `status_label()`, `effective_minutes()` — destructure both fields
2. **site_assessment.rs**: 3 return sites
3. **design.rs**: 3 return sites
4. **quoting.rs**: 4 return sites + 0 wildcard (no change)
5. **report.rs**: 0 direct — all access is through ScenarioOutcome methods

### Dashboard number impact

With the new formula `raw × (int + pol) / 10`, all current passes at
`Polish::OneStar`:
- OneStar int + OneStar pol = (1+1)/10 = 0.2× (same as current 1/5 = 0.2×)
- TwoStar int + OneStar pol = (2+1)/10 = 0.3× (was 2/5 = 0.4×)
- ThreeStar int + OneStar pol = (3+1)/10 = 0.4× (was 3/5 = 0.6×)

So effective minutes will **decrease** for scenarios above OneStar integration.
Current 58.0 min will drop. This is expected — the polish dimension reveals
previously hidden "polish debt."

### Files unchanged

- `main.rs` — no direct ScenarioOutcome construction
- `progress.rs` — milestones, no ScenarioOutcome
- `api_helpers.rs` — no ScenarioOutcome
- `suites/mod.rs` — just re-exports
- `suites/crew_handoff.rs` — all NotImplemented, no Pass sites
- `suites/infrastructure.rs` — all NotImplemented, no Pass sites
