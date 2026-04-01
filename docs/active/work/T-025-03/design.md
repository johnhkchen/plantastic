# T-025-03 Design: Claim Unclaimed Milestones

## Problem

Four milestones in `progress.rs` are attributed to the wrong tickets. T-025-02
and T-025-01 (verification tickets) claimed milestones that were originally
delivered by Sprint 1 tickets. This misattributes engineering credit and makes
the milestone tracker less useful as a historical record.

## Approach: Direct Edit

Only one viable approach — edit the four milestone entries in `progress.rs`:

1. Change `delivered_by` to reference the original delivering ticket(s)
2. Update `note` text to accurately describe what the original tickets built
3. Keep `label` and `unlocks` unchanged

### Format for multi-ticket attribution

The `delivered_by` field is `Option<&'static str>`. For milestones delivered by
multiple tickets, use comma-separated IDs in a single string:
`Some("T-005-02, T-005-03")`. This matches the pattern already used in other
milestone notes (e.g., "T-015-01 delivered... T-015-02 adds...").

## Changes

### Milestone 1: SvelteKit frontend + CF Worker proxy
- `delivered_by`: `"T-025-02"` → `"T-005-02, T-005-03"`
- Note: describe CF Worker proxy (T-005-02) and route skeleton + API client (T-005-03)

### Milestone 2: pt-project: Project/Zone/Tier model + GeoJSON serde
- `delivered_by`: `"T-025-02"` → `"T-002-01, T-003-02"`
- Note: describe domain types (T-002-01) and repo layer (T-003-02)

### Milestone 3: pt-quote: quantity takeoff engine
- `delivered_by`: `"T-025-02"` → `"T-002-02"`
- Note: describe compute_quote() function and what it computes

### Milestone 4: pt-tenant: multi-tenant model + auth context
- `delivered_by`: `"T-025-01"` → `"T-003-02, T-004-02"`
- Note: describe TenantRepo (T-003-02) and X-Tenant-Id extractor (T-004-02)

## Rejected Alternatives

None — this is a straightforward metadata correction with no design ambiguity.

## Risks

- **Low:** Changing notes could accidentally break the Rust string literal syntax.
  Mitigation: `just check` validates compilation.
- **Low:** Milestone count could regress if `delivered_by` is accidentally set to `None`.
  Mitigation: verify dashboard output after changes.
