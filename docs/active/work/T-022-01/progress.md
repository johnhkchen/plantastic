# T-022-01 Progress: Polish Enum + Dashboard

## Completed

### Step 1: Add Polish enum to registry.rs
- Added `Polish` enum with `OneStar`–`FiveStar` variants
- Added `stars()` and `label()` methods
- Changed `Pass(Integration)` to `Pass(Integration, Polish)`
- Updated `status_label()`: `PASS ★★★☆☆ / ★☆☆☆☆`
- Updated `effective_minutes()`: `raw × (int.stars() + pol.stars()) / 10`
- Fixed wildcard patterns `Pass(_)` → `Pass(..)` in registry.rs, report.rs, quoting.rs
- Added `#[allow(dead_code)]` to `Integration::weight()` (no longer called by formula)

### Step 2: Update all suite files
- `site_assessment.rs`: 3 sites updated, `Polish` added to import
- `design.rs`: 3 sites updated, `Polish` added to import
- `quoting.rs`: 4 sites updated, `Polish` added to import
- `crew_handoff.rs`: no change (all NotImplemented)
- `infrastructure.rs`: no change (all NotImplemented)
- Total: 10 `Pass` return sites updated to include `Polish::OneStar`

### Step 3: Update dashboard in report.rs
- Added "Polish debt" summary line showing recoverable minutes
- Added formula explanation line: `effective = raw × (int★ + pol★) / 10`
- Added ratings legend: `integration★ / polish★ (each 1–5, weighted equally)`
- Updated "Raw passing" label to mention both weighting dimensions

### Step 4: Quality gate
- `just check` passes: format, lint, test, scenarios all green
- 8 pass, 0 fail, 9 not implemented, 0 blocked
- Effective savings: 58.0 → 44.5 min (expected decrease due to polish weighting)
- Polish debt: 62.0 min recoverable

## Deviations from plan

- Found 10 Pass sites, not 8 as ticket estimated (ticket missed 2 in quoting.rs — S.3.3/S.3.4 helper functions had Pass returns that needed updating for S.3.1 and S.3.2's helper validation)
- Had to fix `Pass(_)` wildcard patterns → `Pass(..)` — single-field wildcard doesn't match 2-field tuples in Rust
- Added `#[allow(dead_code)]` to `Integration::weight()` to suppress warning (method no longer used by formula but still useful as utility)
