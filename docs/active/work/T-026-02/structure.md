# T-026-02 Structure: Empty States

## Files Created (1)

### `web/src/lib/components/EmptyState.svelte`
Reusable empty state component.

Props:
- `icon: string` — emoji or short text, rendered at ~2.5rem
- `message: string` — primary guidance text
- `submessage?: string` — optional secondary text

Slots:
- Default slot: action area (buttons, links). Renders below submessage with `mt-4`.

Template structure:
```
<div class="rounded-lg border border-gray-200 bg-white px-6 py-12 text-center">
  <div class="text-4xl mb-3">{icon}</div>
  <p class="text-lg font-medium text-gray-600">{message}</p>
  {#if submessage}
    <p class="mt-1 text-sm text-gray-400">{submessage}</p>
  {/if}
  {#if has default slot content}
    <div class="mt-4">{@render children()}</div>
  {/if}
</div>
```

## Files Modified (5 pages + 1 scenario file)

### `web/src/routes/(app)/dashboard/+page.svelte`
- Import `EmptyState`
- Replace lines 98-101 (inline div) with `<EmptyState>` + "New Project" button
- Button triggers existing `showCreateModal = true`

### `web/src/routes/(app)/catalog/+page.svelte`
- Import `EmptyState`
- Replace lines 149-154 (inline div for empty catalog) with `<EmptyState>` + "Add Material" button
- Button triggers existing `openCreateModal()`
- Keep lines 155-158 (filtered results empty state) unchanged — different concern

### `web/src/routes/(app)/project/[id]/editor/+page.svelte`
- Import `EmptyState`
- In the `{:else}` block (line 109), add empty state when `zones.length === 0`
- EmptyState replaces the zone info panel (right sidebar) when no zones
- The ZoneEditor canvas still renders (user needs it to draw the first zone)
- Structure: wrap the zone info panel conditional to show EmptyState instead

### `web/src/routes/(app)/project/[id]/quote/+page.svelte`
- No changes needed — QuoteComparison.svelte already handles the empty state
- But update QuoteComparison to use EmptyState component for consistency

### `web/src/lib/components/quote/QuoteComparison.svelte`
- Import `EmptyState`
- Replace lines 92-99 (inline empty div) with `<EmptyState>` + materials link
- Keep the same copy and link target

### `web/src/routes/(app)/project/[id]/materials/+page.svelte`
- Import `EmptyState`
- Add empty state check before three-column layout (after loading, before `:else`)
- Condition: `zones.length === 0` → "Draw zones first" + link to editor
- Condition: `materials.length === 0` → "Add materials first" + link to catalog
- These replace the three-column layout entirely (no point showing assignment UI without prerequisites)

### `tests/scenarios/src/suites/design.rs`
- S.2.1 `s_2_1_zone_drawing`: Polish::TwoStar → Polish::ThreeStar
- Update comment explaining polish rating

### `tests/scenarios/src/suites/quoting.rs`
- S.3.1 `s_3_1_quantity_from_geometry`: Polish::TwoStar → Polish::ThreeStar
- S.3.2 `s_3_2_three_tier_quote`: Polish::TwoStar → Polish::ThreeStar
- Update comments explaining polish rating

## Module boundaries

EmptyState is a leaf component — no imports beyond Svelte primitives. Same pattern as ErrorBanner and LoadingSkeleton: no stores, no API calls, pure presentation.

## No files deleted

All changes are additive (new component) or in-place replacements (inline div → EmptyState component).
