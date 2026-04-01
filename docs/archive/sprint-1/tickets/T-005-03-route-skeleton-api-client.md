---
id: T-005-03
story: S-005
title: route-skeleton-api-client
type: task
status: open
priority: high
phase: done
depends_on: [T-005-02]
---

## Context

Flesh out the SvelteKit route skeleton with layout components and build the API client module that all pages will use to talk to the backend. The API client follows the pattern from HMW Workshop — fetch wrapper with auth headers, SSE streaming support, typed responses, error handling.

## Acceptance Criteria

### API Client (web/src/lib/api/)
- apiFetch wrapper: injects auth token, handles errors, returns typed JSON
- SSE streaming reader: reads `data: {json}\n\n` events, calls onPartial callback
- Error types: RateLimitError (429 with retry info), AuthError (401), ValidationError (422)
- Mock mode toggle (VITE_MOCK_API env var) for development without backend

### Layout & Components
- App layout: navigation sidebar (dashboard, catalog, settings), header with tenant name placeholder
- Dashboard layout: project list shell with create button
- Project workspace layout: tab navigation (editor, materials, quote, viewer, export)
- Catalog layout: material list shell with add button
- All layouts render with placeholder content — no API calls yet (that's T-006-01)

### State Management
- Session store (Svelte 5 runes): current tenant, auth token, active project
- Project store: project data, zones, tiers — populated when API is connected
