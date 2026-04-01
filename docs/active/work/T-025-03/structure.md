# T-025-03 Structure: Claim Unclaimed Milestones

## Files Modified

### `tests/scenarios/src/progress.rs`

The only file touched. Four `Milestone` struct entries are updated:

1. **Lines ~171-181** — "pt-project" milestone
   - `delivered_by`: `Some("T-025-02")` → `Some("T-002-01, T-003-02")`
   - `note`: rewrite to describe T-002-01 domain types and T-003-02 repo layer

2. **Lines ~183-192** — "pt-quote" milestone
   - `delivered_by`: `Some("T-025-02")` → `Some("T-002-02")`
   - `note`: rewrite to describe T-002-02's compute_quote()

3. **Lines ~315-323** — "SvelteKit frontend" milestone
   - `delivered_by`: `Some("T-025-02")` → `Some("T-005-02, T-005-03")`
   - `note`: rewrite to describe CF Worker proxy and route skeleton

4. **Lines ~325-335** — "pt-tenant" milestone
   - `delivered_by`: `Some("T-025-01")` → `Some("T-003-02, T-004-02")`
   - `note`: rewrite to describe TenantRepo and X-Tenant-Id extractor

## Files NOT Modified

- No new files created
- No files deleted
- No other milestones touched
- No scenario files changed
- Ticket frontmatter NOT updated (Lisa handles phase transitions)

## Module Boundaries

No module boundaries affected. This is a data-only change within a single
`static` array in `progress.rs`. No public API changes.

## Ordering

All four edits are independent — no ordering constraints between them.
