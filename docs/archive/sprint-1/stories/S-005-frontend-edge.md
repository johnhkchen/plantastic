---
id: S-005
epic: E-002
title: Frontend & Edge Layer
status: open
priority: high
---

# S-005: Frontend & Edge Layer

## Purpose

Stand up the SvelteKit frontend and Cloudflare Worker proxy so there's a deployable UI and edge layer ready to connect to the API. This track runs in parallel with E-001's Rust critical path — no Rust dependencies.

## Scope

- SvelteKit 5 with Cloudflare Pages adapter, Tailwind CSS, TypeScript
- Route skeleton: landing, dashboard, project workspace (editor, materials, quote, viewer, export), catalog management, settings, client-facing branded view
- Cloudflare Worker: CORS handling, per-IP + per-session rate limiting, auth token passthrough, SSE streaming passthrough
- API client module with SSE streaming support (pattern from HMW Workshop)
- Svelte 5 runes for reactive state management

## Tickets

- T-005-01: SvelteKit scaffold + CF Pages
- T-005-02: CF Worker proxy
- T-005-03: Route skeleton + API client module
