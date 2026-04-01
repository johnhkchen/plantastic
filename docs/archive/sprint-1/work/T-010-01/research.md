# Research — T-010-01: pt-solar engine

## What exists

### Solar-sim prototype (TypeScript)
Location: `/Volumes/ext1/swe/repos/solar-sim/src/lib/solar/`

The prototype is fully functional with proven algorithms:

**position.ts** — Wraps SunCalc.js library. Converts radians to degrees, normalizes azimuth from south-based (SunCalc convention) to north-based compass bearing. Handles polar conditions (midnight sun, polar night) by checking noon altitude when sunrise/sunset are NaN.

**sun-hours.ts** — 5-minute interval sampling. Iterates 288 samples/day (24h * 60min / 5min). Counts samples where altitude > 0. Fast-paths polar conditions to 24h or 0h. Returns `DailySunData` with sun hours, sun times, and polar condition.

**seasonal.ts** — Aggregates daily sun hours across date ranges. Provides monthly, yearly, and arbitrary-range summaries with min/max/average statistics.

**categories.ts** — Light classification: full-sun (6+h), part-sun (4-6h), part-shade (2-4h), full-shade (<2h). Shade-aware variant uses effective hours when available.

**exposure-grid.ts** — Grid calculation over lat/lng bounding box. Uses meters-to-degrees conversion (111,320 m/deg lat). Samples representative days (default 12) rather than every day. Row-major Float32Array output. Includes tree shadow integration (not needed for T-010-01).

**Key constants**: `SAMPLING_INTERVAL_MINUTES = 5`, `SAMPLES_PER_DAY = 288`, `METERS_PER_DEGREE_LAT = 111320`.

### SunCalc algorithm (what the prototype delegates to)
SunCalc implements a simplified solar position algorithm based on NOAA formulas:
1. Julian date from calendar date
2. Solar mean anomaly, ecliptic longitude, right ascension, declination
3. Hour angle from observer longitude and equation of time
4. Altitude = arcsin(sin(lat)*sin(dec) + cos(lat)*cos(dec)*cos(ha))
5. Azimuth = atan2(-cos(dec)*sin(ha), sin(lat)*cos(dec)*cos(ha) - cos(lat)*sin(dec))

This is ~50 lines of trig. No external library needed in Rust — just `f64::sin()`, `f64::cos()`, etc.

### Existing Plantastic crates

**Workspace pattern** (from Cargo.toml):
- Crates in `crates/*`, auto-included via workspace members
- `edition = "2021"`, `rust-version = "1.75"`
- Workspace deps: `geo 0.29`, `chrono 0.4`, `serde`, `uuid`, `rust_decimal`
- Strict lints: correctness=deny, suspicious=deny, style=warn, unsafe_code=deny

**Crate structure pattern** (from pt-geo):
- `Cargo.toml`: name, version 0.1.0, edition/license/rust-version from workspace, `[lints] workspace = true`
- `lib.rs`: module declarations + re-exports
- Focused modules: one concern per file (area.rs, perimeter.rs, etc.)
- Tests inline with `#[cfg(test)]` blocks
- `approx` crate for floating-point comparison in dev-dependencies

**pt-test-utils**: `timed(|| { ... })` for 10s default timeout, `run_with_timeout(Duration, || { ... })` for custom.

### Scenario infrastructure

**S.1.3** is already registered in `tests/scenarios/src/suites/site_assessment.rs`:
- `id: "S.1.3"`, `name: "Sun exposure analysis"`, `area: SiteAssessment`, `time_savings_minutes: 20.0`
- Currently returns `ScenarioOutcome::NotImplemented`
- Description: "location + date range -> sun hours grid with correct light categories"

**Milestone** exists in `tests/scenarios/src/progress.rs`:
- `label: "pt-solar: sun position + radiance grid"`, `delivered_by: None`, `unlocks: ["S.1.3", "S.2.3"]`

**Scenario Cargo.toml** needs `pt-solar` added as dependency.

### Reference data for validation

From prototype tests (timeanddate.com cross-referenced):
- Portland (45.5152, -122.6784): summer solstice ~15.5h, winter solstice ~8.5h, equinox ~12h
- Singapore (1.3521, 103.8198): ~12h year-round, <0.5h variation
- Tromso (69.6492, 18.9553): midnight sun (24h) in summer, polar night (0h) in winter
- Portland summer solstice noon altitude: ~68 degrees (90 - 45.5 + 23.5)
- Portland winter solstice noon altitude: ~21 degrees (90 - 45.5 - 23.5)

### Performance targets
- Full year single point: <5ms (prototype achieves ~2ms in JS)
- Residential grid: <500ms
- Rust should be 5-10x faster than the JS prototype

## Constraints and boundaries

1. **Pure computation** — no I/O, no async, no database. The crate must work in WASM.
2. **No SunCalc dependency** — implement the NOAA solar position algorithm directly in Rust (~50 lines of trig).
3. **Chrono for dates** — workspace already has chrono 0.4. Use `NaiveDate`/`NaiveDateTime` for UTC calculations.
4. **No shade/tree integration** — that's future work (pt-satellite, pt-scene). Only theoretical sun hours.
5. **f64 throughout** — sun positions are float math, not monetary. No need for rust_decimal.
6. **Serde on public types** — for API serialization later. Follow existing pattern.
7. **Grid uses f32 values array** — matches prototype's Float32Array for memory efficiency on large grids.

## Risks

1. **Algorithm accuracy** — Must validate against SunCalc/timeanddate.com. The NOAA simplified algorithm has known limitations at extreme latitudes.
2. **Polar edge cases** — midnight sun and polar night detection needs careful handling (the prototype's approach is sound).
3. **Date handling** — UTC-only simplifies things but means "a day" is always UTC midnight-to-midnight. For landscaping in the SF Bay Area this is fine.
