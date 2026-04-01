# T-026-02 Plan: Empty States

## Pre-flight
- [x] Run `cargo run -p pt-scenarios` — baseline: 76.5 min / 240.0 min (31.9%)

## Step 1: Create EmptyState.svelte
Create `web/src/lib/components/EmptyState.svelte` with icon/message/submessage props and default slot for actions. Pure presentation component.

**Verify**: File exists with correct props and renders markup.

## Step 2: Update Dashboard
Replace inline empty state div (lines 98-101) with `<EmptyState>` component. Add a "New Project" button in the action slot that triggers `showCreateModal = true`.

**Verify**: Import added, inline div replaced, button wired to existing handler.

## Step 3: Update Catalog
Replace inline empty catalog div (lines 149-154) with `<EmptyState>` component. Add "Add Material" button that triggers `openCreateModal()`. Keep the filtered results empty state as-is.

**Verify**: Import added, only the "truly empty" state replaced, filter empty state unchanged.

## Step 4: Update Editor
Add empty state in the zone info panel area when `zones.length === 0`. The ZoneEditor canvas continues to render (user draws on it). The empty state appears where the zone list sidebar would be, plus a subtle overlay hint on the canvas area.

**Verify**: Canvas renders when empty, empty state visible in sidebar area.

## Step 5: Update QuoteComparison
Replace inline empty state (lines 92-99) with `<EmptyState>` component. Keep the materials link.

**Verify**: Import added, same copy preserved, link works.

## Step 6: Update Materials Assignment
Add prerequisite checks before three-column layout:
- If `zones.length === 0`: EmptyState with "Draw zones first" + link to editor
- Else if `materials.length === 0`: EmptyState with "Add materials first" + link to catalog
Three-column layout only renders when both prerequisites are met.

**Verify**: Both conditions tested, links point to correct pages.

## Step 7: Advance scenario polish ratings
- `design.rs` S.2.1: Polish::TwoStar → Polish::ThreeStar
- `quoting.rs` S.3.1, S.3.2: Polish::TwoStar → Polish::ThreeStar
Update comments to explain the advancement reason.

**Verify**: `cargo run -p pt-scenarios` shows increased effective savings.

## Step 8: Quality gate
- `just check` (fmt + lint + test + scenarios)
- Verify no regressions in scenario dashboard

## Testing strategy
This is frontend UI work — no new Rust unit tests. Verification is:
1. Scenario dashboard effective savings increases (polish advancement)
2. `just check` passes all 4 gates
3. No scenario regressions
