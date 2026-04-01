# T-005-01 Structure: SvelteKit Scaffold

## File Tree

After completion, the `web/` directory will contain:

```
web/
├── package.json                          # scripts: dev, build, preview, check, lint
├── pnpm-lock.yaml                        # lockfile
├── svelte.config.ts                      # adapter-cloudflare, TypeScript preprocess
├── vite.config.ts                        # Tailwind vite plugin, /api dev proxy
├── tsconfig.json                         # extends .svelte-kit/tsconfig.json
├── .prettierrc                           # Prettier config (from sv create)
├── .prettierignore                       # Prettier ignore
├── eslint.config.js                      # ESLint config (from sv create)
├── src/
│   ├── app.html                          # HTML shell (from sv create)
│   ├── app.css                           # Tailwind import + brand theme placeholders
│   ├── app.d.ts                          # SvelteKit type declarations
│   └── routes/
│       ├── +layout.svelte                # Root layout: imports app.css, renders <slot>
│       ├── +page.svelte                  # / (landing)
│       ├── dashboard/
│       │   └── +page.svelte              # /dashboard
│       ├── project/
│       │   └── [id]/
│       │       ├── +page.svelte          # /project/[id]
│       │       ├── editor/
│       │       │   └── +page.svelte      # /project/[id]/editor
│       │       ├── materials/
│       │       │   └── +page.svelte      # /project/[id]/materials
│       │       ├── quote/
│       │       │   └── +page.svelte      # /project/[id]/quote
│       │       ├── viewer/
│       │       │   └── +page.svelte      # /project/[id]/viewer
│       │       └── export/
│       │           └── +page.svelte      # /project/[id]/export
│       ├── catalog/
│       │   └── +page.svelte              # /catalog
│       ├── settings/
│       │   └── +page.svelte              # /settings
│       └── c/
│           └── [token]/
│               └── +page.svelte          # /c/[token]
└── static/
    └── favicon.png                       # (from sv create, replace later)
```

## Files Created by `sv create`

The CLI generates the base project:
- `package.json`, `svelte.config.ts`, `vite.config.ts`, `tsconfig.json`
- `src/app.html`, `src/app.d.ts`
- `src/routes/+page.svelte` (demo — we replace)
- `.prettierrc`, `.prettierignore`, `eslint.config.js`
- `static/favicon.png`

## Files Modified After Scaffold

### `svelte.config.ts`
- **Change:** Replace default adapter (`adapter-auto`) with `adapter-cloudflare`
- **Why:** Ticket AC requires Cloudflare Pages deployment

### `vite.config.ts`
- **Add:** Dev proxy config: `/api` → `process.env.API_URL || 'http://localhost:3000'`
- **Why:** Ticket AC: "Vite dev proxy: /api → configurable backend URL"

### `package.json`
- **Verify/add:** scripts `dev`, `build`, `preview`, `check`, `lint` all present
- **Add:** `deploy` script: `wrangler pages deploy .svelte-kit/cloudflare`
- **Change:** Swap `@sveltejs/adapter-auto` → `@sveltejs/adapter-cloudflare` in dependencies

### `src/app.css`
- **Add:** Brand theming placeholders in `@theme` block:
  - `--color-brand-*` (primary, secondary, accent — placeholder values)
  - `--font-display`, `--font-body` (placeholder font family references)
- **Why:** Ticket AC: "sensible defaults (colors, fonts placeholder for brand theming)"

### `src/routes/+layout.svelte`
- **Create:** Root layout importing `app.css`, rendering `{@render children()}`
- **Why:** Tailwind needs to be imported at the root. Svelte 5 uses `{@render children()}` in layouts.

### `src/routes/+page.svelte`
- **Replace:** CLI demo content with minimal landing page placeholder

## Files Created (Route Stubs)

Each route file is a minimal `+page.svelte` with a single `<h1>` identifying the page:

```svelte
<h1>Page Name</h1>
<p>Coming soon.</p>
```

Routes to create:
- `src/routes/dashboard/+page.svelte`
- `src/routes/project/[id]/+page.svelte`
- `src/routes/project/[id]/editor/+page.svelte`
- `src/routes/project/[id]/materials/+page.svelte`
- `src/routes/project/[id]/quote/+page.svelte`
- `src/routes/project/[id]/viewer/+page.svelte`
- `src/routes/project/[id]/export/+page.svelte`
- `src/routes/catalog/+page.svelte`
- `src/routes/settings/+page.svelte`
- `src/routes/c/[token]/+page.svelte`

## Files NOT Created (out of scope)

- `src/lib/api/` — T-005-03
- `src/lib/stores/` — T-005-03
- `src/lib/components/` — T-005-03
- Layout files for sub-routes (`+layout.svelte` under dashboard, project, etc.) — T-005-03
- `worker/` directory — T-005-02
- `wrangler.toml` for Worker — T-005-02

## Dependency Changes

### Add
- `@sveltejs/adapter-cloudflare` — CF Pages adapter

### Remove
- `@sveltejs/adapter-auto` — replaced by adapter-cloudflare

### Kept (from `sv create`)
- `@sveltejs/kit`, `svelte`, `vite`
- `@tailwindcss/vite`, `tailwindcss`
- `typescript`, `svelte-check`
- `prettier`, `prettier-plugin-svelte`
- `eslint`, `eslint-plugin-svelte`, etc.

## Module Boundaries

This ticket creates the project shell. There are no module boundaries to define — no `lib/` code, no components, no stores. The only boundary is:

- **Routes are stubs.** Every `+page.svelte` is a standalone placeholder with no imports. T-005-03 will add `+layout.svelte` files, lib imports, and component composition.
- **CSS theming is isolated to `app.css`.** The `@theme` block defines CSS custom properties. Components (added later) reference these via Tailwind utility classes.
