# T-018-01 Structure — GitHub Actions CI

## Files Created

### `.github/workflows/ci.yml`

The single new file. Contains one workflow with two jobs.

```
name: CI

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]         # Cache writes only happen on main pushes

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - checkout
      - setup rust toolchain (stable, clippy, rustfmt)
      - rust cache (Swatinem/rust-cache)
      - install just (taiki-e/install-action)
      - just fmt-check
      - just lint
      - just test
      - just scenarios

  web:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: web
    steps:
      - checkout
      - setup pnpm (pnpm/action-setup)
      - setup node (with pnpm caching, cwd: web)
      - pnpm install --frozen-lockfile
      - pnpm run check (svelte-check)
      - pnpm run lint (prettier + eslint)
```

## Files Modified

None.

## Files Deleted

None.

## Module Boundaries

This is a CI-only change. No Rust or TypeScript source code is modified. The workflow exercises existing commands (`just check` decomposed into individual steps for granular failure reporting, and `pnpm run check` / `pnpm run lint` for the frontend).

## Key Design Decisions in Structure

### Decomposed `just check` Instead of Single Step

The `just check` recipe runs `fmt-check`, `lint`, `test`, `scenarios` sequentially and stops on first failure. In CI, decomposing into separate steps provides:
- Granular failure indication (developer sees which step failed without reading logs)
- Parallel step timing visibility
- The ability to see scenarios output even if formatting fails

### Push + PR Trigger

- `pull_request` trigger runs on every PR to main (the primary gate)
- `push` to main trigger ensures the cache is populated on merge (Swatinem/rust-cache writes cache on push events, reads on PR events)

### pnpm Version Detection

`pnpm/action-setup` will auto-detect the pnpm version from `package.json`'s `packageManager` field if present. Since `web/package.json` doesn't have this field, we'll specify a version explicitly.

### No `timeout-minutes` Override Needed

Default GitHub Actions timeout is 360 minutes. The acceptance criteria say < 10 minutes. Rather than setting a global timeout, we rely on:
- The `just test` recipe's built-in 120s timeout
- The natural speed of the other steps
- We set `timeout-minutes: 15` on each job as a safety net against hung processes
