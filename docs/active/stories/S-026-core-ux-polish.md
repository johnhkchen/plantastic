---
id: S-026
epic: E-012
title: Core UX Polish — Loading, Errors, Empty States
status: open
priority: medium
depends_on: [S-022]
tickets: [T-026-01, T-026-02]
---

## Goal

Every frontend page gets loading states, error handling, and empty state prompts. This is the minimum bar for ★★☆☆☆ polish.

## Pages

| Page | Loading | Error | Empty State |
|------|---------|-------|-------------|
| Dashboard `/` | Project list skeleton | Error banner + retry | "Create your first project" CTA |
| Catalog `/catalog` | Material list skeleton | Error banner + retry | "Add your first material" CTA |
| Zone Editor `/project/[id]` | Zone overlay skeleton | Error toast | "Draw your first zone" prompt |
| Quote `/project/[id]/quote` | Quote columns skeleton | Error banner | "Assign materials to see quotes" |
| Viewer `/project/[id]/viewer` | Loading overlay on iframe | Error fallback | "Upload a scan to preview" |

## Acceptance Criteria

- Every async data fetch has a visible loading state
- Every failed fetch shows a user-facing error with retry option
- Every list/collection page has an empty state with a CTA
- Affected scenarios advance to ★★☆☆☆ polish
- `just check` passes
