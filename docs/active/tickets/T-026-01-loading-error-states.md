---
id: T-026-01
story: S-026
title: loading-error-states
type: task
status: open
priority: medium
phase: open
depends_on: [T-022-01]
---

## Context

No frontend page currently shows loading indicators or error messages. Async fetches either show a blank screen (loading) or silently fail (error). This is the single biggest polish gap.

## Acceptance Criteria

- Loading states:
  - Dashboard project list: skeleton placeholder while fetching
  - Catalog material list: skeleton placeholder while fetching
  - Quote comparison: column skeleton while fetching tier quotes
  - Zone editor: overlay indicator while zones load
  - Viewer: loading overlay while iframe + glTF load
- Error states:
  - All API fetch failures show a user-visible error banner
  - Error banner includes retry button that re-fetches
  - Network errors distinguished from server errors in message
- Implementation uses a shared `LoadingState` pattern (not ad-hoc per page)
- Affected scenarios advance to ★★☆☆☆ polish (from ★☆☆☆☆)
- `just check` passes

## Implementation Notes

- Create reusable Svelte components: `LoadingSkeleton.svelte`, `ErrorBanner.svelte`
- Use SvelteKit's `{#await}` blocks or `$state` tracking pattern
- Don't over-engineer — simple spinners/skeletons, not shimmer animations
- Error messages should be human-readable: "Couldn't load materials. Check your connection and try again." not "Error: 500"
