# T-005-01 Research: SvelteKit Scaffold

## Current State

The `web/` directory does not exist. This is a greenfield scaffold. The repo currently contains only docs (specification, tickets, stories, epics, workflow definitions) and project config files (.lisa.toml, CLAUDE.md, LICENSE, README.md). No Cargo.toml, no Rust crates, no frontend code — those are in parallel tracks.

## Ticket Boundaries

T-005-01 is the first ticket in S-005 (Frontend & Edge Layer). The dependency chain is:

```
T-005-01 (this) → T-005-02 (CF Worker proxy) → T-005-03 (route skeleton + API client)
```

**T-005-01 scope**: Project init, adapter config, Tailwind, TypeScript, route file stubs, Vite dev proxy, npm scripts. Placeholder pages only — no layouts, no API client, no state management.

**T-005-02 scope**: `worker/` directory, wrangler.toml, CORS, rate limiting, auth passthrough, SSE streaming. Separate project, depends on T-005-01 only for knowing the Pages URL.

**T-005-03 scope**: Flesh out route layouts, API client module (`web/src/lib/api/`), SSE streaming reader, session/project stores (Svelte 5 runes), sidebar navigation, workspace tab nav. This is where the skeleton becomes a real app shell.

Key boundary: T-005-01 creates route _files_ (placeholder `+page.svelte`), T-005-03 creates route _layouts_ and real components. T-005-01 should not add layouts, stores, or API client code — that would step on T-005-03.

## Technology Choices (from spec + ticket)

| Choice | Version / Detail | Source |
|--------|-----------------|--------|
| SvelteKit | 5 | Ticket AC, spec |
| Svelte | 5 (runes) | Story S-005 |
| Adapter | @sveltejs/adapter-cloudflare | Ticket AC: "Cloudflare Pages adapter" |
| CSS | Tailwind CSS v4 | Ticket AC |
| TypeScript | Strict mode | Ticket AC |
| Package manager | pnpm (available, v10.33.0) | Dev env |
| Node | v23.3.0 | Dev env |
| Deploy target | Cloudflare Pages | Spec infrastructure table |
| Dev proxy | Vite `/api → configurable backend URL` | Ticket AC |

## Route Structure (from ticket AC)

```
/                           → +page.svelte (landing)
/dashboard                  → +page.svelte (project list)
/project/[id]               → +page.svelte (project workspace layout)
/project/[id]/editor        → +page.svelte (zone drawing — future)
/project/[id]/materials     → +page.svelte (material assignment — future)
/project/[id]/quote         → +page.svelte (quote review — future)
/project/[id]/viewer        → +page.svelte (3D viewer — future)
/project/[id]/export        → +page.svelte (PDF/DXF — future)
/catalog                    → +page.svelte (material catalog management)
/settings                   → +page.svelte (tenant branding/account)
/c/[token]                  → +page.svelte (client-facing branded view)
```

The spec shows `client/[token]/` in the route tree, but the ticket AC says `/c/[token]`. The ticket takes precedence — it's the more specific document.

## SvelteKit 5 + Svelte 5 Runes Context

SvelteKit 5 uses Svelte 5 which introduces runes (`$state`, `$derived`, `$effect`) replacing the `$:` reactive syntax and stores API. Key implications for scaffold:

- `svelte.config.js` doesn't need special runes config — it's the default in Svelte 5
- TypeScript support is built-in via `<script lang="ts">`
- New `+page.svelte` files use `<script>` (no `<script context="module">` for load — that's `+page.ts`)
- Adapter-cloudflare handles platform-specific hooks and SSR on CF Pages

## Cloudflare Pages Adapter

`@sveltejs/adapter-cloudflare` builds for the Cloudflare Pages platform. Config in `svelte.config.js`:

```js
import adapter from '@sveltejs/adapter-cloudflare';
export default { kit: { adapter: adapter() } };
```

Deploy via: `npx wrangler pages deploy .svelte-kit/cloudflare`

No wrangler.toml needed for Pages (that's Workers). The build output directory is `.svelte-kit/cloudflare`.

## Tailwind CSS v4

Tailwind v4 uses a CSS-first configuration approach:
- Import via `@import "tailwindcss"` in the main CSS file
- Configuration via `@theme` blocks in CSS instead of `tailwind.config.js`
- Vite plugin: `@tailwindcss/vite`
- No `content` array needed — auto-detection via Vite

This is a significant change from v3. The scaffold should use v4 patterns.

## Vite Dev Proxy

SvelteKit uses Vite under the hood. The proxy is configured in `vite.config.ts`:

```ts
server: {
  proxy: {
    '/api': {
      target: process.env.API_URL || 'http://localhost:3000',
      changeOrigin: true
    }
  }
}
```

This allows the frontend to call `/api/*` during development and have requests forwarded to the backend. In production, the CF Worker handles routing.

## Package.json Scripts (from ticket AC)

- `dev` — Vite dev server
- `build` — Production build for CF Pages
- `preview` — Preview production build locally
- `check` — `svelte-check` (type checking)
- `lint` — ESLint + Prettier (or just Prettier for Svelte 5)

## Deployment

The ticket AC says: "Builds and deploys to CF Pages (wrangler pages deploy)". This means:
1. `pnpm build` produces the CF Pages output
2. `npx wrangler pages deploy .svelte-kit/cloudflare` deploys it

No wrangler.toml is needed for Pages — the project name is passed as a CLI arg or configured in a Pages project.

## Constraints & Assumptions

1. **No Rust dependencies** — this track is pure JS/TS, runs in parallel with E-001
2. **Placeholder pages only** — T-005-03 adds real layouts and components
3. **No API calls** — no backend exists yet; T-005-03 adds mock mode
4. **Brand theming** — ticket says "colors, fonts placeholder for brand theming" in Tailwind config
5. **pnpm** — available in the environment, preferred for monorepo compatibility
6. **The `web/` directory** — spec places the frontend here, ticket AC confirms

## Open Questions

1. Should we add a `.node-version` or `.nvmrc` file? The repo doesn't have one yet.
2. ESLint configuration — Svelte 5 ecosystem is still stabilizing eslint-plugin-svelte. A minimal setup (Prettier only) may be more appropriate for the scaffold.
3. The spec mentions `svelte.config.js` but SvelteKit 5 init generates `svelte.config.ts` by default when TypeScript is selected. Either works.
