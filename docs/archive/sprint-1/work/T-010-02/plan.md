# T-010-02 Plan: pt-climate Engine

## Step 1: Create crate skeleton + types

- `crates/pt-climate/Cargo.toml`
- `src/lib.rs` — module declarations (stubs)
- `src/types.rs` — all public types: Coordinates, DayOfYearRange, Confidence, FrostDates, HardinessZone, SunsetZone, GrowingSeason, ClimateProfile
- **Verify**: `cargo check -p pt-climate`

## Step 2: Frost date lookup

- `src/frost.rs` — NORTHERN_FROST_TABLE, SOUTHERN_FROST_TABLE, frost_dates() function
- Latitude-band matching, elevation adjustment (+4 days/300m), coastal modifier
- Tropical fallback (|lat| < 20), polar fallback (|lat| > 70)
- Tests: SF (37.77N, coastal) → ~Feb last frost, ~Dec first frost; San Jose (37.34N, inland) → later spring frost; Portland-latitude band sanity check
- **Verify**: `cargo test -p pt-climate`

## Step 3: USDA hardiness zone lookup

- `src/hardiness.rs` — hardiness_zone() function
- Min winter temp from latitude, zone calculation, coastal modifier
- Tests: SF → 10b, Oakland → 10a/10b, San Jose → 9b, known tropical → 13a/13b
- **Verify**: `cargo test -p pt-climate`

## Step 4: Sunset Western Garden zone lookup

- `src/sunset.rs` — BAY_AREA_ZONES table, sunset_zone() function
- Bounding-box entries for zones 14-17 covering Bay Area subregions
- Tests: SF downtown → 17, Oakland hills → 16, San Jose → 15, Livermore → 14, outside Bay Area → None
- **Verify**: `cargo test -p pt-climate`

## Step 5: Growing season computation

- `src/growing_season.rs` — growing_season() function
- Compute typical/short/long days from frost date ranges
- Handle southern hemisphere (year wraparound)
- Tests: Bay Area frost dates → ~280-365 day growing season; northern latitude → shorter season
- **Verify**: `cargo test -p pt-climate`

## Step 6: Convenience function + re-exports

- `src/lib.rs` — climate_profile() function, all re-exports
- Tests: climate_profile() for SF returns complete, consistent profile
- **Verify**: `cargo test -p pt-climate`

## Step 7: Claim milestone + quality gate

- Update `tests/scenarios/src/progress.rs`: add pt-climate milestone, set delivered_by to T-010-02
- Run `just check`
- **Verify**: all gates pass

## Testing Strategy

| Layer | What | Count (est) |
|-------|------|-------------|
| frost.rs unit | Bay Area + edge cases (tropical, polar) | 5-6 |
| hardiness.rs unit | Bay Area cities + edge cases | 4-5 |
| sunset.rs unit | Bay Area cities + outside coverage | 4-5 |
| growing_season.rs unit | Bay Area + short season + wraparound | 3-4 |
| lib.rs integration | climate_profile() end-to-end | 1-2 |

All tests use `pt_test_utils::timed()`. No database, no I/O. Expected runtime: <100ms total.

Reference values for test assertions:
- SF: last frost ~Feb (DOY ~45-60), first frost ~Dec (DOY ~340-355), zone 10b, Sunset 17, ~300+ day season
- San Jose: later spring frost than SF, zone 9b, Sunset 15, ~280 day season
- Oakland: between SF and San Jose, zone 10a/10b, Sunset 16
