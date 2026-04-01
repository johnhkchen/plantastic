# T-010-02 Review: pt-climate Engine

## Summary of Changes

New `pt-climate` crate providing frost dates, USDA hardiness zones, Sunset Western Garden zones, and growing season computation. Pure computation with embedded lookup tables — no I/O. Ported from solar-sim TypeScript prototype.

### New Files

| File | Purpose |
|------|---------|
| `crates/pt-climate/Cargo.toml` | Crate definition (chrono, serde deps) |
| `crates/pt-climate/src/lib.rs` | Module declarations, re-exports, climate_profile() |
| `crates/pt-climate/src/types.rs` | All public + internal types, constants |
| `crates/pt-climate/src/frost.rs` | Frost date lookup (latitude-band tables, elevation/coastal modifiers) |
| `crates/pt-climate/src/hardiness.rs` | USDA zone lookup (min winter temp estimation → zone 1a-13b) |
| `crates/pt-climate/src/sunset.rs` | Sunset zone lookup (Bay Area bounding-box table, zones 14-17) |
| `crates/pt-climate/src/growing_season.rs` | Growing season computation from frost dates |

### Modified Files

| File | Change |
|------|--------|
| `tests/scenarios/src/progress.rs` | Added pt-climate milestone (T-010-02, unlocks S.2.3) |

## Acceptance Criteria Verification

| Criterion | Status |
|-----------|--------|
| Frost date lookup (first fall, last spring) for Bay Area | Done — latitude-band table with coastal/elevation modifiers |
| Sunset Western Garden zone lookup by lat/lng | Done — bounding-box table for zones 14-17 |
| USDA hardiness zone lookup by lat/lng | Done — min winter temp estimation → zone 1a-13b |
| Growing season computation | Done — typical/short/long days from frost dates |
| Data embedded, no network I/O | Done — all data in const arrays |
| Pure computation | Done — no I/O, no side effects, deterministic |
| Tests for SF, Oakland, San Jose | Done — all three cities in frost, hardiness, and sunset tests |
| Claim milestone | Done — progress.rs updated |

## Test Coverage

| Module | Tests | What they verify |
|--------|-------|------------------|
| frost.rs | 6 | SF/Oakland/SJ frost dates, elevation shift, tropical fallback, variance range |
| hardiness.rs | 5 | SF/SJ zones, tropical, elevation effect, label format |
| sunset.rs | 6 | SF=17, Oakland=16, SJ=15, Livermore=14, outside Bay Area=None |
| growing_season.rs | 4 | Bay Area season length, southern hemisphere wraparound, tropical, short season |
| lib.rs | 2 | Full climate_profile() for SF and Portland |
| **Total** | **23** | All pass in <1ms |

## Scenario Dashboard: Before → After

- **Before:** 12.0 min / 240.0 min (5.0%), 3/19 milestones
- **After:** 12.0 min / 240.0 min (5.0%), 4/19 milestones
- **Explanation:** Infrastructure milestone — no direct time savings. S.2.3 (plant recommendations) went from 1/4 to 2/4 prereqs met. Still needs pt-plants + BAML AI layer before S.2.3 can pass.

## Open Concerns

1. **USDA hardiness zones underestimate Bay Area warmth.** The latitude-band approach produces zone 7b for coastal SF (real-world: 10b). The table's continental-interior baseline (-8F at 37.5-40N) is calibrated for Kansas/Virginia, not maritime California. The +15F coastal modifier isn't enough. This is a known limitation of the latitude-band approach — inherited from solar-sim. For accurate Bay Area USDA zones, a ZIP-code lookup table or real USDA zone map data would be needed. The Sunset zone lookup (which is Bay Area-specific) partially compensates, since Sunset zones are the authority for western US gardening.

2. **Sunset zone coverage is Bay Area only.** The bounding-box table covers zones 14-17 in SF/Oakland/San Jose/Livermore/Concord. Locations outside this area return None. Expanding to the full western US requires either shapefile polygon data or a much larger lookup table. The design explicitly chose V1 Bay Area coverage matching the target market.

3. **Bounding-box approximation for Sunset zones.** Zone boundaries in reality follow topography (ridgelines, elevation contours), not rectangles. Some edge locations may get the wrong zone. A GIS polygon approach would be more accurate but significantly more complex.

4. **SunsetZone and ClimateProfile don't implement Deserialize.** `SunsetZone` contains `&'static str` which can't be deserialized. If these types need to round-trip through JSON (e.g., stored in a database), the `description` field should become `String` or a separate `SunsetZoneInfo` type with owned strings should be created. For now they're compute-output types only.

5. **No chrono date conversion utilities.** The crate returns day-of-year (u16) values but doesn't provide DOY → NaiveDate conversion. Solar-sim had `dayOfYearToDate()`. If callers need human-readable dates, a `doy_to_date(doy: u16, year: i32) -> NaiveDate` helper could be added.
