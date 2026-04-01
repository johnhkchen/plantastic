# T-005-01 Design: SvelteKit Scaffold

## Decision: Manual Init vs `sv create`

### Option A: `sv create` (SvelteKit CLI)

Run `npx sv create web` with interactive prompts to generate a SvelteKit 5 project, then add Tailwind, adapter, and routes.

**Pros:** Generates the latest recommended project structure. Handles tsconfig, vite config, svelte config boilerplate. Stays current with SvelteKit defaults.

**Cons:** CLI is interactive — hard to reproduce deterministically. Generates opinionated defaults we may need to undo (e.g., demo pages, extra config). The integrations for Tailwind and other add-ons are available through the CLI's checklist.

### Option B: Manual file creation

Write every file by hand — package.json, svelte.config.ts, vite.config.ts, tsconfig.json, app.html, routes.

**Pros:** Full control, nothing to delete.

**Cons:** Easy to miss a config field. Tedious. May not match SvelteKit's expected internal structure, causing subtle build issues.

### Decision: Option A — `sv create` with add-ons

Use `sv create` in non-interactive mode to generate the project, selecting Tailwind and TypeScript. Then customize: swap the adapter, add the route stubs, configure the dev proxy, add brand theming placeholders.

**Rationale:** The CLI generates a correct, current baseline. Customization on top is simpler and less error-prone than building from scratch. If the CLI doesn't support full non-interactive mode, we run it interactively once and commit the result.

---

## Decision: Tailwind v4 CSS-first vs v3 config file

### Option A: Tailwind v4 (CSS-first, `@tailwindcss/vite`)

Use the new v4 approach: `@import "tailwindcss"` in CSS, `@theme` blocks for customization, Vite plugin.

### Option B: Tailwind v3 (`tailwind.config.js`, PostCSS)

Use the traditional approach with a config file and PostCSS plugin.

### Decision: Option A — Tailwind v4

**Rationale:** v4 is current. The `sv create` CLI's Tailwind add-on installs v4 by default. Using v3 would mean fighting the tooling. The CSS-first approach is simpler for a scaffold — brand theming placeholders go directly in the CSS file as `@theme` custom properties.

---

## Decision: Adapter configuration

### Option A: `@sveltejs/adapter-cloudflare`

The Pages-specific adapter. Handles SSR on CF Pages, platform bindings, etc.

### Option B: `@sveltejs/adapter-static`

Build a static site (SPA mode). Simpler but no SSR.

### Decision: Option A — adapter-cloudflare

**Rationale:** Ticket AC explicitly says "Cloudflare Pages adapter configured." The spec anticipates SSR for SEO on the landing page and server-side auth handling. Static would work for now but would need to be replaced later.

---

## Decision: Route file content

### Option A: Minimal placeholder (just `<h1>` with page name)

Each `+page.svelte` contains a heading and "Coming soon" text. No logic, no imports.

### Option B: Slightly richer placeholder (heading + description of what the page will do)

Include a brief description of the page's future purpose as inline text.

### Decision: Option A — Minimal placeholders

**Rationale:** T-005-03 will replace all page content with real layouts and components. Anything we write here gets deleted. Keep it minimal: a heading tag identifying the route, nothing more. This avoids merge conflicts when T-005-03 rewrites the pages.

---

## Decision: Linting setup

### Option A: ESLint + Prettier

Full linting with eslint-plugin-svelte for Svelte 5.

### Option B: Prettier only

Format-only, no lint rules.

### Option C: Defer to `sv create` defaults

Let the CLI pick, accept what it generates.

### Decision: Option C — Accept CLI defaults

**Rationale:** The `sv create` Prettier add-on sets up Prettier with the Svelte plugin. ESLint add-on configures eslint-plugin-svelte. Both are battle-tested. Accept whatever the CLI generates — refinement can happen later. The ticket just says "lint" needs to work.

---

## Decision: Dev proxy configuration

The ticket says `/api → configurable backend URL`. Two sub-decisions:

1. **Where to configure:** Vite config (`vite.config.ts`) is the standard place for dev server proxying in SvelteKit.
2. **How to make it configurable:** Use an environment variable (e.g., `API_URL`) with a sensible default (`http://localhost:3000`).

### Decision: Vite config with env var

Configure in `vite.config.ts` under `server.proxy`. Read `API_URL` from environment. Default to `http://localhost:3000`.

---

## Decision: Package manager lockfile

### Option A: pnpm (pnpm-lock.yaml)

pnpm is available (v10.33.0). Efficient disk usage, strict dependency resolution.

### Option B: npm (package-lock.json)

Universal, no extra tooling.

### Decision: Option A — pnpm

**Rationale:** Available in the dev environment. Better for monorepo scenarios (the Rust crates + web + worker are all under one repo). Strict hoisting prevents phantom dependency issues.

---

## Summary of Decisions

| Decision | Choice | Key Reason |
|----------|--------|------------|
| Project init | `sv create` with add-ons | Correct baseline, less error-prone |
| Tailwind | v4 (CSS-first) | Current default from CLI |
| Adapter | adapter-cloudflare | Ticket requirement, future SSR |
| Route content | Minimal `<h1>` placeholders | T-005-03 replaces everything |
| Linting | CLI defaults (Prettier + ESLint) | Works out of the box |
| Dev proxy | Vite config + `API_URL` env var | Standard SvelteKit pattern |
| Package manager | pnpm | Available, monorepo-friendly |
