---
id: T-005-01
story: S-005
title: sveltekit-scaffold
type: task
status: open
priority: high
phase: done
depends_on: []
---

## Context

The SvelteKit frontend is the primary UI for all user roles — landscaper, admin, client, crew. This ticket bootstraps the project with the proven stack from HMW Workshop: SvelteKit 5 with Svelte runes, Cloudflare Pages adapter, Tailwind CSS, TypeScript strict mode.

No Rust dependencies — this track runs in parallel with E-001.

## Acceptance Criteria

- SvelteKit 5 initialized in web/ directory
- Cloudflare Pages adapter configured (svelte.config.js)
- Tailwind CSS + sensible defaults (colors, fonts placeholder for brand theming)
- TypeScript strict mode
- Route structure created (files can be placeholder/empty):
  - / (landing)
  - /dashboard (project list)
  - /project/[id] (project workspace layout)
  - /project/[id]/editor (zone drawing — future)
  - /project/[id]/materials (material assignment — future)
  - /project/[id]/quote (quote review — future)
  - /project/[id]/viewer (3D viewer — future)
  - /project/[id]/export (PDF/DXF — future)
  - /catalog (material catalog management)
  - /settings (tenant branding/account)
  - /c/[token] (client-facing branded view — future)
- Vite dev proxy: /api → configurable backend URL
- package.json scripts: dev, build, preview, check, lint
- Builds and deploys to CF Pages (wrangler pages deploy)
