# T-018-01 Research — GitHub Actions CI

## Current State

No `.github/` directory exists. No CI is configured. The quality gate is entirely manual: developers run `just check` locally before marking tickets done.

## Quality Gate (`just check`)

The `justfile` defines the pre-commit gate as four sequential steps:

1. **`just fmt-check`** — `cargo fmt --all -- --check`
2. **`just lint`** — `cargo clippy --workspace --all-targets -- -D warnings`
3. **`just test`** — Compiles with `--no-run`, then runs `cargo test --workspace` with a 120s hard timeout (uses `timeout` on Linux, `gtimeout` on macOS)
4. **`just scenarios`** — `cargo run -p pt-scenarios` (value delivery dashboard)

Steps are pure Rust — no database, no network, no external services needed for the current test suite. The scenario harness is a standalone binary in `tests/scenarios/`.

## SvelteKit Frontend (`web/`)

- Package manager: **pnpm** (lockfile is `pnpm-lock.yaml`, lockfileVersion 9.0)
- Scripts relevant to CI:
  - `npm run check` → `svelte-kit sync && svelte-check --tsconfig ./tsconfig.json`
  - `npm run lint` → `prettier --check . && eslint .`
- Dev dependencies only — no runtime deps. This is a SvelteKit app on Cloudflare Pages adapter.
- Node version not pinned. TypeScript 5.9, Svelte 5, Vite 7.

## Rust Workspace

- **Workspace root**: `Cargo.toml` with 11 crates in `crates/*` plus `tests/scenarios`
- **Excludes**: `apps/viewer` (Bevy WASM viewer, not part of workspace tests)
- **MSRV**: `rust-version = "1.75"` in workspace package config
- **Key dependencies**: sqlx (Postgres), tokio, axum, geo, aws-sdk-s3
- **sqlx**: Uses compile-time query checking. May need `SQLX_OFFLINE=true` or `sqlx-data.json` in CI if there are compile-time queries. Need to verify.
- **Cargo.lock**: Checked into repo (correct for applications)

## Dependencies (T-017-01)

T-017-01 (rust-lambda-deploy) is `phase: done`. It set up cross-compilation with cargo-zigbuild and SST deployment. CI doesn't need cross-compilation — it only needs native compilation for testing.

## Caching Considerations

- **Cargo build artifacts**: `~/.cargo/registry` (crate sources) and `target/` (build artifacts). The `target/` directory is the expensive one — full workspace build from scratch takes minutes.
- **pnpm store**: `~/.local/share/pnpm/store` or project `node_modules/`
- **Rust toolchain**: `actions-rust-lang/setup-rust-toolchain` handles installation and caching of the toolchain itself. Separate from build artifact caching.

## GitHub Actions Ecosystem (Relevant Actions)

- `actions/checkout@v4` — repo checkout
- `actions-rust-lang/setup-rust-toolchain@v1` — installs Rust, handles caching of toolchain
- `Swatinem/rust-cache@v2` — the standard Cargo build cache action (caches registry, target dir). Widely used, handles cache key rotation.
- `pnpm/action-setup@v4` — installs pnpm
- `actions/setup-node@v4` — installs Node.js with built-in pnpm caching support
- `taiki-e/install-action@just` — installs just

## Branch Protection

GitHub branch protection rules are configured via the GitHub UI or API, not via workflow files. The workflow defines the check; the protection rule gates on it. The acceptance criteria mention "PR blocked if any step fails" — this requires a branch protection rule on `main` requiring the CI workflow to pass.

## sqlx Offline Mode

If any crate uses `sqlx::query!()` macros, compilation requires either a live database or `SQLX_OFFLINE=true` with cached query metadata. Need to check if `.sqlx/` directory exists with cached queries.

## Timing Constraints

Acceptance criteria: runs in < 10 minutes. Key timing factors:
- Rust compilation from cold cache: 3-5 minutes for a workspace this size
- Cached compilation (dependency changes only): 30-60 seconds
- Tests: should be fast (120s hard timeout, most tests are compute-only)
- pnpm install + svelte-check: ~30-60 seconds
- Scenario dashboard: seconds (it's a small binary)

With proper caching, a typical CI run should be well under 10 minutes.
