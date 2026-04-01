# Plan — T-010-01: pt-solar engine

## Implementation steps

### Step 1: Scaffold crate
- Create `crates/pt-solar/Cargo.toml` and `crates/pt-solar/src/lib.rs`
- Verify `cargo check -p pt-solar` compiles
- Commit: "Add pt-solar crate skeleton"

### Step 2: Types module
- Write `types.rs` with all shared types: Coordinates, SolarPosition, PolarCondition, DailySunData, SeasonalSummary, constants
- All types derive Serialize, Deserialize, Debug, Clone, Copy where appropriate
- No tests needed — these are data definitions

### Step 3: Sun position algorithm
- Write `position.rs` implementing NOAA solar position formulas
- `sun_position(coords, datetime) -> SolarPosition`
- Internal: julian_date, solar_mean_anomaly, ecliptic_longitude, right_ascension, declination, hour_angle, altitude, azimuth
- Tests against SunCalc reference values:
  - Portland summer solstice noon: altitude 65-72 deg, azimuth 170-190 deg
  - Portland winter solstice noon: altitude 18-25 deg
  - Portland midnight: altitude < 0
  - Singapore noon: altitude > 60 (near-equatorial)

### Step 4: Sun hours integration
- Write `sun_hours.rs` with `daily_sun_hours(coords, date) -> DailySunData`
- 288 samples at 5-min intervals, count altitude > 0, convert to hours
- Polar detection: all positive → MidnightSun, none positive → PolarNight
- Tests:
  - Portland summer solstice: 15.0-16.5 hours
  - Portland winter solstice: 8.0-9.5 hours
  - Portland equinox: 11.5-12.75 hours
  - Singapore: 11.5-12.5 all year, <0.5h seasonal variation
  - Tromso summer: 24h (midnight sun)
  - Tromso winter: 0h (polar night)

### Step 5: Light classification
- Write `classification.rs` with LightCategory enum and classify()
- Thresholds: FullSun >= 6, PartSun >= 4, PartShade >= 2, FullShade < 2
- Tests: exact boundary values (6.0 → FullSun, 5.99 → PartSun, etc.)

### Step 6: Seasonal aggregation
- Write `seasonal.rs` with `annual_sun_hours(coords, start, end) -> SeasonalSummary`
- Iterates each day, aggregates min/max/average
- Tests: Portland full year (average ~12h, summer max >15, winter min <9)

### Step 7: Radiance grid
- Write `grid.rs` with LatLngBounds, GridConfig, ExposureGrid, radiance_grid()
- Meters-to-degrees conversion, grid dimension calculation, cell center computation
- Sample N representative days, average sun hours per cell
- Tests:
  - Grid dimensions: known bounds + resolution → expected width/height
  - Small grid (3x3) over SF: all cells have reasonable sun hours (8-16h average)
  - Performance: residential-sized grid completes within timeout

### Step 8: Wire up lib.rs re-exports
- Add all pub mod declarations and pub use re-exports
- Verify `cargo doc -p pt-solar` generates clean docs

### Step 9: Register scenario and milestone
- Add `pt-solar` to `tests/scenarios/Cargo.toml`
- Implement S.1.3 in `site_assessment.rs`: create grid over SF, validate results
- Claim milestone in `progress.rs`: set delivered_by, write note

### Step 10: Quality gate
- `just fmt` — auto-format
- `just lint` — clippy strict
- `just test` — all tests pass
- `just scenarios` — S.1.3 passes at OneStar, no regressions

## Testing strategy

**Unit tests** (inline `#[cfg(test)]`):
- position: 4 tests (solstice noon, winter noon, night, equatorial)
- sun_hours: 7 tests (Portland 4 seasons, Singapore, Tromso summer/winter)
- classification: 5 tests (each category + boundaries)
- seasonal: 2 tests (Portland year, short range)
- grid: 3 tests (dimensions, cell centers, small grid)
Total: ~21 unit tests

**Scenario test** (S.1.3):
- End-to-end: location → grid → classifications → validate
- Cross-crate integration (uses the real pt-solar, not mocks)

**Validation approach**:
- Sun position: compared against SunCalc/timeanddate.com reference values
- Sun hours: compared against known day lengths from timeanddate.com with 15-min tolerance
- Grid: validated against expected Bay Area sun hour ranges
- All expected values computed independently in tests, not derived from the code under test

**Performance validation**:
- `timed()` wrapper on all unit tests (10s timeout)
- Grid test verifies completion within reasonable time
- The 5ms/500ms targets from the ticket are architectural constraints, not test assertions (would be flaky in CI)

## Risk mitigations
- If NOAA algorithm doesn't match SunCalc closely enough: widen tolerances to match prototype test ranges (already generous: +/- 3 degrees for position, +/- 0.75 hours for day length)
- If grid is too slow: reduce default sample_days from 12 to 6, or increase default resolution
