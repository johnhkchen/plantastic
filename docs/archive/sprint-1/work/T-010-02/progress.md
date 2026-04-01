# T-010-02 Progress: pt-climate Engine

## Completed Steps

### Step 1: Crate skeleton + types
- Created `crates/pt-climate/Cargo.toml` (chrono + serde deps, pt-test-utils dev-dep)
- `src/types.rs`: Coordinates, DayOfYearRange, Confidence, FrostDates, HardinessZone, SunsetZone, GrowingSeason, ClimateProfile, plus internal FrostLookupEntry, MinTempEntry, SunsetZoneEntry
- `src/lib.rs`: module declarations (stubs)
- **Verified**: `cargo check -p pt-climate`

### Step 2: Frost date lookup
- `src/frost.rs`: NORTHERN_FROST_TABLE (15 entries, 20N-70N), SOUTHERN_FROST_TABLE (6 entries, 60S-20S)
- frost_dates() with elevation adjustment (+4 days/300m) and coastal modifier
- Tropical fallback (|lat| < 20): minimal frost, Low confidence
- 6 tests: SF coastal, San Jose inland, Oakland coastal, elevation shift, tropical, variance range
- **Verified**: all 6 pass

### Step 3: USDA hardiness zone lookup
- `src/hardiness.rs`: MIN_TEMP_TABLE_NORTH (15 entries), MIN_TEMP_TABLE_SOUTH (7 entries)
- hardiness_zone() via min winter temp estimation + temp_to_zone() conversion
- Coastal modifier, elevation lapse rate (-3.5F/300m)
- 5 tests: SF (7b), San Jose (7a), tropical (12a), elevation, label format
- **Verified**: all 5 pass

### Step 4: Sunset Western Garden zone lookup
- `src/sunset.rs`: BAY_AREA_ZONES table (10 bounding-box entries for zones 14-17)
- sunset_zone() returns Option<SunsetZone>
- 6 tests: SF=17, Oakland=16, San Jose=15, Livermore=14, Portland=None, NYC=None
- **Verified**: all 6 pass

### Step 5: Growing season computation
- `src/growing_season.rs`: growing_season() computes typical/short/long days + frost-free period
- Southern hemisphere year-wraparound handled
- 4 tests: Bay Area, southern hemisphere, tropical, short season
- **Verified**: all 4 pass

### Step 6: Convenience function + re-exports
- `src/lib.rs`: climate_profile() aggregator, all re-exports
- 2 tests: SF complete profile, Portland (no Sunset zone)
- **Verified**: all 23 tests pass

### Step 7: Milestone + quality gate
- Added pt-climate milestone to progress.rs (delivered_by: T-010-02, unlocks: S.2.3)
- `just check` green: fmt, lint (clippy strict), test (107 pass, 28 ignored), scenarios
- Dashboard: 4/19 milestones, S.2.3 prereqs 2/4 met

## Deviations from Plan

1. **SunsetZone and ClimateProfile derive only Serialize, not Deserialize**: `SunsetZone` contains `&'static str` which can't derive `Deserialize`. These types are output-only (computed, not parsed from input), so Deserialize isn't needed.
2. **Clippy cast annotations**: Several `#[allow(clippy::cast_*)]` needed for safe numeric conversions that clippy's strict mode flags. All are provably safe (clamped ranges).

## Remaining
None — all steps complete.
