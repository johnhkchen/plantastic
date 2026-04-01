# T-005-01 Plan: SvelteKit Scaffold

## Step 1: Create SvelteKit project via `sv create`

Run `npx sv create web` with SvelteKit 5, TypeScript, Tailwind CSS, Prettier, ESLint add-ons.

**Verification:** `cd web && pnpm install && pnpm dev` starts a dev server on localhost.

**Commit:** "feat(web): scaffold SvelteKit 5 project with Tailwind and TypeScript"

## Step 2: Swap adapter to Cloudflare Pages

1. `pnpm remove @sveltejs/adapter-auto`
2. `pnpm add -D @sveltejs/adapter-cloudflare`
3. Update `svelte.config.ts`: import and use `adapter-cloudflare`

**Verification:** `pnpm build` completes successfully, output in `.svelte-kit/cloudflare/`.

**Commit:** "feat(web): configure Cloudflare Pages adapter"

## Step 3: Configure Vite dev proxy

Add `/api` proxy to `vite.config.ts` pointing to `process.env.API_URL || 'http://localhost:3000'`.

**Verification:** `pnpm dev` starts without errors. Proxy config visible in vite config.

**Commit:** Combine with Step 2 if small — otherwise "feat(web): add /api dev proxy to Vite config"

## Step 4: Add brand theming placeholders to Tailwind

Edit `src/app.css` to add `@theme` block with placeholder brand colors and fonts:
- `--color-brand-primary`, `--color-brand-secondary`, `--color-brand-accent`
- `--font-display`, `--font-body`

**Verification:** A page using `class="text-brand-primary"` picks up the color.

**Commit:** "feat(web): add Tailwind brand theming placeholders"

## Step 5: Create route stubs

Create all route directories and `+page.svelte` files:

| Route | File |
|-------|------|
| `/` | `src/routes/+page.svelte` (replace demo) |
| `/dashboard` | `src/routes/dashboard/+page.svelte` |
| `/project/[id]` | `src/routes/project/[id]/+page.svelte` |
| `/project/[id]/editor` | `src/routes/project/[id]/editor/+page.svelte` |
| `/project/[id]/materials` | `src/routes/project/[id]/materials/+page.svelte` |
| `/project/[id]/quote` | `src/routes/project/[id]/quote/+page.svelte` |
| `/project/[id]/viewer` | `src/routes/project/[id]/viewer/+page.svelte` |
| `/project/[id]/export` | `src/routes/project/[id]/export/+page.svelte` |
| `/catalog` | `src/routes/catalog/+page.svelte` |
| `/settings` | `src/routes/settings/+page.svelte` |
| `/c/[token]` | `src/routes/c/[token]/+page.svelte` |

Each file:
```svelte
<h1>Page Name</h1>
<p>Coming soon.</p>
```

Also ensure the root `+layout.svelte` imports `app.css` and renders children.

**Verification:** `pnpm dev` — navigate to each route, confirm the heading renders.

**Commit:** "feat(web): add route stubs for all pages"

## Step 6: Verify package.json scripts

Ensure these scripts exist and work:
- `dev` → `vite dev`
- `build` → `vite build`
- `preview` → `vite preview`
- `check` → `svelte-kit sync && svelte-check --tsconfig ./tsconfig.json`
- `lint` → `prettier --check . && eslint .`

**Verification:** Run each script, confirm no errors.

**Commit:** Only if changes needed — likely already correct from `sv create`.

## Step 7: Final verification

1. `pnpm build` — clean production build
2. `pnpm check` — TypeScript passes
3. `pnpm lint` — no lint errors (fix any formatting issues)
4. Navigate all routes in dev mode

**Commit:** "feat(web): final scaffold verification and cleanup" (only if fixes needed)

## Testing Strategy

This is a scaffold ticket — there is no application logic to unit test. Verification is:

1. **Build succeeds:** `pnpm build` exits 0 with CF Pages output
2. **Type check passes:** `pnpm check` exits 0
3. **Lint passes:** `pnpm lint` exits 0
4. **Routes render:** Each route serves its placeholder content in dev mode
5. **Proxy configured:** `/api` proxy entry exists in vite config

No Playwright E2E tests at this stage — those come with T-005-03 when there's actual behavior to test.

## Commit Strategy

Aim for 2-3 atomic commits:
1. Initial scaffold (Steps 1-3: project init + adapter + proxy)
2. Tailwind theming + route stubs (Steps 4-5)
3. Final fixes if needed (Steps 6-7)

This keeps the history clean while allowing bisect if something breaks later.
