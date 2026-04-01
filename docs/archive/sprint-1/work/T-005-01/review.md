# T-005-01 Review: SvelteKit Scaffold

## Summary of Changes

### Files Created

**Project configuration (web/):**
- `package.json` — SvelteKit 5 project with scripts: dev, build, preview, check, lint, format
- `svelte.config.js` — Cloudflare Pages adapter, Svelte 5 runes mode
- `vite.config.ts` — Tailwind v4 vite plugin, `/api` dev proxy (configurable via `API_URL` env var)
- `tsconfig.json` — TypeScript strict mode, extends SvelteKit generated config
- `eslint.config.js` — ESLint flat config with typescript-eslint + eslint-plugin-svelte
- `.prettierrc` — Tabs, single quotes, svelte plugin
- `.prettierignore` — Excludes .svelte-kit, build, node_modules
- `.npmrc` — From sv create (engine-strict=true)
- `.gitignore` — SvelteKit defaults
- `pnpm-lock.yaml` — Lockfile

**Source files (web/src/):**
- `app.html` — HTML shell (from sv create)
- `app.css` — Tailwind v4 import + `@theme` brand placeholders (primary/secondary/accent colors, display/body fonts)
- `app.d.ts` — SvelteKit app type declarations

**Route stubs (web/src/routes/):**
- `+layout.svelte` — Root layout: imports app.css, renders children via `{@render children()}`
- `+page.svelte` — Landing page with brand-primary colored heading
- `dashboard/+page.svelte` — Dashboard placeholder
- `project/[id]/+page.svelte` — Project workspace (renders project ID from params)
- `project/[id]/+page.ts` — Load function extracting `params.id`
- `project/[id]/editor/+page.svelte` — Zone editor placeholder
- `project/[id]/materials/+page.svelte` — Materials placeholder
- `project/[id]/quote/+page.svelte` — Quote placeholder
- `project/[id]/viewer/+page.svelte` — 3D viewer placeholder
- `project/[id]/export/+page.svelte` — Export placeholder
- `catalog/+page.svelte` — Catalog placeholder
- `settings/+page.svelte` — Settings placeholder
- `c/[token]/+page.svelte` — Client-facing branded view placeholder

**Static assets:**
- `static/favicon.png` — Default from sv create (replace with Plantastic branding later)
- `src/lib/assets/favicon.svg` — Default SVG favicon from sv create

### Files Modified
None — this is a greenfield scaffold. All files are new.

### Files Deleted
None.

## Acceptance Criteria Verification

| Criterion | Status | Notes |
|-----------|--------|-------|
| SvelteKit 5 initialized in web/ | ✅ | Svelte 5.55.1, SvelteKit 2.55.0 |
| Cloudflare Pages adapter configured | ✅ | `@sveltejs/adapter-cloudflare` in svelte.config.js |
| Tailwind CSS + brand theming placeholders | ✅ | Tailwind v4 CSS-first, `@theme` with brand colors/fonts |
| TypeScript strict mode | ✅ | `strict: true` in tsconfig.json |
| Route structure (all 11 routes) | ✅ | All present as placeholder pages |
| Vite dev proxy /api → configurable URL | ✅ | `API_URL` env var, defaults to localhost:3000 |
| Scripts: dev, build, preview, check, lint | ✅ | All present and verified |
| Builds and deploys to CF Pages | ✅ | `pnpm build` produces `.svelte-kit/cloudflare/` output |

## Test Coverage

**No unit or E2E tests.** This is expected — the ticket is a scaffold with no application logic. Testing strategy:

- `pnpm build` — Production build succeeds ✅
- `pnpm check` — TypeScript type-check passes (0 errors, 295 files) ✅
- `pnpm lint` — Prettier + ESLint pass ✅

E2E tests (Playwright) will be appropriate starting with T-005-03 when there's layout navigation and component behavior to verify.

## Dependencies Installed

| Package | Version | Purpose |
|---------|---------|---------|
| @sveltejs/adapter-cloudflare | ^7.2.8 | CF Pages deployment |
| @sveltejs/kit | ^2.50.2 | SvelteKit framework |
| svelte | ^5.54.0 | Svelte 5 with runes |
| tailwindcss | ^4.2.2 | CSS framework |
| @tailwindcss/vite | ^4.2.2 | Tailwind Vite integration |
| typescript | ^5.9.3 | Type checking |
| svelte-check | ^4.4.2 | Svelte type checker |
| @types/node | ^25.5.0 | Node.js types (for process.env in vite config) |
| prettier | ^3.8.1 | Code formatting |
| prettier-plugin-svelte | ^3.5.1 | Svelte formatting |
| eslint | ^10.1.0 | Linting |
| eslint-plugin-svelte | ^3.16.0 | Svelte lint rules |
| typescript-eslint | ^8.58.0 | TS lint rules |
| globals | ^17.4.0 | ESLint browser/node globals |
| vite | ^7.3.1 | Build tool |

## Open Concerns

1. **Node version compatibility:** Node 23.3.0 (current in dev env) is incompatible with `@sveltejs/vite-plugin-svelte`. Required `fnm use 22`. Consider adding a `.node-version` file at the repo root (`22`) so all contributors use a compatible version. This affects CI configuration too.

2. **esbuild/workerd build scripts:** pnpm flagged ignored build scripts for `esbuild` and `workerd`. These are native addons needed by Vite and the Cloudflare adapter. Running `pnpm approve-builds` may be needed in CI, or configure `.npmrc` to allow them.

3. **`svelte.config.js` not `.ts`:** The CLI generated `.js` with JSDoc types, not `.ts`. This is fine — SvelteKit supports both. The spec mentioned `.js`, so this aligns.

4. **No `wrangler.toml` for Pages:** Cloudflare Pages doesn't need a wrangler.toml (that's for Workers). Deploy command is `npx wrangler pages deploy .svelte-kit/cloudflare --project-name=plantastic`. A deploy script could be added to package.json once the Pages project name is decided.

5. **Tailwind `@theme` naming:** Used `--color-brand-*` which generates `text-brand-primary`, `bg-brand-secondary`, etc. T-005-03 should verify these utility names work in practice and adjust if needed.

6. **`src/lib/index.ts`:** Generated by sv create as an empty barrel file. Left as-is — T-005-03 will populate `lib/` with API client, stores, and components.
