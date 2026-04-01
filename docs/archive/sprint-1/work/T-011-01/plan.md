# T-011-01 Plan: pt-satellite crate

## Step 1: Create crate skeleton

- Create `crates/pt-satellite/Cargo.toml` with workspace deps
- Create `crates/pt-satellite/src/lib.rs` with module stubs
- Verify: `cargo check -p pt-satellite` compiles

## Step 2: Define types and error

- Create `src/types.rs` with `ProjectBaseline`, `LotBoundary`, `DetectedTree`, `DataSourceLabel`
- Create `src/error.rs` with `SatelliteError` enum
- Re-export `pt_solar::Coordinates` from lib.rs
- Verify: `cargo check -p pt-satellite` compiles

## Step 3: Define traits

- Create `src/traits.rs` with `Geocoder`, `ParcelSource`, `CanopySource` traits
- Verify: `cargo check -p pt-satellite` compiles

## Step 4: Implement EmbeddedSource

- Create `src/embedded.rs` with `EmbeddedSource` struct
- Implement `Geocoder` for `EmbeddedSource` — address lookup to lat/lng
- Implement `ParcelSource` for `EmbeddedSource` — hardcoded lot polygon
- Implement `CanopySource` for `EmbeddedSource` — hardcoded tree data
- Unit tests: geocode known address, geocode unknown address errors, lot boundary area,
  tree count and plausibility
- Verify: `cargo test -p pt-satellite` passes

## Step 5: Implement BaselineBuilder

- Create `src/builder.rs` with `BaselineBuilder` struct
- Implement `build()` pipeline: geocode → parcel → canopy → solar → assemble
- Unit test: build with EmbeddedSource produces valid baseline
- Verify: `cargo test -p pt-satellite` passes

## Step 6: Wire up S.1.2 scenario

- Add `pt-satellite` dep to `tests/scenarios/Cargo.toml`
- Replace `s_1_2_satellite_prepopulation()` body in `site_assessment.rs`
- Assert: coordinates reasonable, lot area in range, trees plausible, grid valid
- Verify: `cargo run -p pt-scenarios` shows S.1.2 PASS ★☆☆☆☆

## Step 7: Claim milestone

- Update `progress.rs`: set `delivered_by: Some("T-011-01")` on pt-satellite milestone
- Write descriptive note documenting what was delivered and what it enables

## Step 8: Quality gate

- Run `just fmt` to format
- Run `just lint` to check clippy
- Run `just test` to verify all workspace tests pass
- Run `just scenarios` to verify no regressions, S.1.2 now passing

## Testing Strategy

**Unit tests** (in pt-satellite crate):
- `embedded::tests` — geocode hit/miss, lot boundary correctness, tree data plausibility
- `builder::tests` — full pipeline produces valid baseline, unknown address errors

**Scenario test** (in pt-scenarios):
- `s_1_2_satellite_prepopulation` — end-to-end: address → baseline with all assertions
- Independent expected values: lot area computed from known polygon coords, not from
  system's `area_sqft()` function

**What NOT to test**:
- pt-solar internals (already tested by pt-solar and S.1.3)
- Serialization round-trips for types with no custom serde logic (serde derive is reliable)
