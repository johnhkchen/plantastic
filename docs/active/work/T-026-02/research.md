# T-026-02 Research: Empty States

## Current State of Empty Handling Across Pages

### 1. Dashboard (`web/src/routes/(app)/dashboard/+page.svelte`)
- **Lines 98-101**: Basic inline empty state exists
- Shows: `<div class="rounded-lg border...">No projects yet. Create your first project to get started.</div>`
- No icon, no CTA button, just plain text in a bordered box
- The "New Project" button is already in the page header (line 84-89) but visually disconnected from the empty state message

### 2. Catalog (`web/src/routes/(app)/catalog/+page.svelte`)
- **Lines 149-154**: Basic inline empty state for zero materials
- Shows: `No materials in the catalog. Add your first material to get started.`
- **Lines 155-158**: Separate empty state for filtered results showing no matches
- "Add Material" button exists in header (line 131-134) but not in the empty state
- Filter empty state is a different concern (search found nothing vs. truly empty catalog)

### 3. Editor (`web/src/routes/(app)/project/[id]/editor/+page.svelte`)
- **Lines 117-142**: Zone info panel only renders `{#if zones.length > 0}`
- When no zones exist, the right sidebar simply doesn't appear — blank space
- No message, no CTA, no guidance for drawing the first zone
- The ZoneEditor canvas still renders (good), but user gets no onboarding hint

### 4. Quote (`web/src/routes/(app)/project/[id]/quote/+page.svelte`)
- **Lines 92-99 of QuoteComparison.svelte**: Has an inline empty state
- Shows: "No quotes to compare" + link to Materials tab
- Already has a CTA ("Assign materials in the Materials tab")
- But style is ad-hoc (not reusable), text-only, no icon/illustration

### 5. Materials Assignment (`web/src/routes/(app)/project/[id]/materials/+page.svelte`)
- **Lines 178-206**: Goes straight to three-column layout when not loading
- No handling for: zero zones, zero materials, zero assignments
- ZoneList, MaterialPicker, QuoteSummary all render even when empty — each likely shows its own blank state or nothing

### 6. Viewer (`web/src/routes/(app)/project/[id]/viewer/+page.svelte`)
- No empty state at all — immediately renders the Viewer component with a test scene URL
- The test scene always exists (test_scene.glb), so there's no "no scan" condition today
- Future: when real scans replace the test scene, an empty state will be needed

## Existing Component Patterns

### From T-026-01
- `ErrorBanner.svelte`: `{message, onretry?}` — red banner with optional retry
- `LoadingSkeleton.svelte`: `{rows?, variant?}` — three variants (card/row/column)
- Both are simple prop-based components, no slots
- Pattern: dedicated component imported where needed, not a generic wrapper

### Component library style
- Tailwind CSS throughout, no component framework (no Skeleton UI, no shadcn)
- Components are flat `.svelte` files in `web/src/lib/components/`
- Subdirectories for domain-specific groups: `assignment/`, `catalog/`, `quote/`, `viewer/`, `zone-editor/`
- Props via `$props()` (Svelte 5 runes), no context API usage visible

## Data Flow for Empty Detection

| Page | Empty when | Data source |
|------|-----------|-------------|
| Dashboard | `projectStore.projects.length === 0` | `apiFetch('/projects')` |
| Catalog | `materials.length === 0` | `apiFetch('/materials')` |
| Editor | `zones.length === 0` | `fetchZones(projectId)` |
| Quote | `isEmpty` derived (all tier line_items empty) | `fetchQuote(id, tier)` |
| Materials | `zones.length === 0` or `materials.length === 0` | Multiple API calls |
| Viewer | No scan uploaded (future) | Currently always has test scene |

## Key Constraint

The ticket says "Empty states should disappear immediately when data exists — no stale empty state after first add." All pages use reactive state (`$state`, `$derived`), so adding data to the local arrays will automatically re-render and hide the empty state. No special invalidation logic needed.

## Scenario Impact

The polish dimension of these scenarios reflects UX completeness. T-026-01 moved S.2.1, S.2.4, S.3.1, S.3.2 from Polish::OneStar to Polish::TwoStar. Empty states could justify advancing to Polish::ThreeStar for pages that get them (dashboard has no direct scenario, but editor/catalog/quote/materials do).

Current polish ratings after T-026-01:
- S.2.1 (zone drawing): ★★☆☆☆ integration / ★★☆☆☆ polish
- S.2.2 (material catalog): ★☆☆☆☆ integration / ★★★★★ polish (pure computation)
- S.2.4 (3D preview): ★★☆☆☆ integration / ★★☆☆☆ polish
- S.3.1 (quantity): ★★★☆☆ integration / ★★☆☆☆ polish
- S.3.2 (three-tier): ★★★☆☆ integration / ★★☆☆☆ polish
