# T-018-01 Design — GitHub Actions CI

## Decision: Single Workflow, Two Jobs (Rust + Web)

### Approach Chosen

One workflow file `.github/workflows/ci.yml` triggered on PRs to `main`. Two parallel jobs:

1. **`rust`** — Install toolchain, install just, run `just fmt-check`, `just lint`, `just test`, `just scenarios`. Uses `Swatinem/rust-cache@v2` for Cargo build artifacts and `taiki-e/install-action` for just.

2. **`web`** — Install pnpm + Node.js, `pnpm install --frozen-lockfile`, `pnpm run check`, `pnpm run lint`. Uses `actions/setup-node@v4` with built-in pnpm caching.

Both jobs must pass for the workflow to be green.

### Why Two Jobs Instead of One

The Rust build takes 3-5 minutes cold. The web checks take ~30 seconds. Running them serially wastes time. Running them in parallel means the web job finishes quickly and gives early feedback on frontend issues while the Rust job compiles.

The jobs are independent — no shared state, no shared artifacts. Parallel is the natural fit.

### Why Not Separate Workflows

A single workflow with two jobs is simpler to reason about: one status check to require in branch protection, one file to maintain. Separate workflows would require configuring multiple required checks.

### Alternatives Rejected

**Option A: Single job, sequential steps.** Simpler YAML but slower. Rust compilation dominates, web checks wait for no reason. Rejected for speed.

**Option B: Matrix strategy over Rust/Web.** Adds complexity (conditional step logic) for no benefit. Matrix is for "same steps, different configs." Rust and Web have completely different steps. Rejected for complexity.

**Option C: Reusable workflows / composite actions.** Over-engineering for a two-job workflow. Rejected for YAGNI.

**Option D: Docker-based CI (devcontainer).** Too slow for cold starts, harder to cache. Rejected.

### Caching Strategy

**Rust (Swatinem/rust-cache@v2):**
- Caches `~/.cargo/registry/index`, `~/.cargo/registry/cache`, `~/.cargo/git/db`, and `target/`
- Cache key: hash of `Cargo.lock` + toolchain version
- Shared across PRs (read-only for PRs, write on main)
- Expected savings: 2-4 minutes on warm cache

**Node/pnpm (actions/setup-node@v4):**
- Built-in caching of pnpm store when `cache: 'pnpm'` is set with `cwd: web/`
- pnpm install with `--frozen-lockfile` ensures reproducibility

### Toolchain Pinning

The workspace declares `rust-version = "1.75"` but there's no `rust-toolchain.toml`. For CI stability, we'll use `actions-rust-lang/setup-rust-toolchain@v1` with `toolchain: stable` and let it track latest stable. This matches what developers use locally.

We add `components: clippy, rustfmt` explicitly to ensure they're available.

### sqlx Compile-Time Queries

Research confirmed: all sqlx queries use runtime string-based `query_scalar` / `query_as`, not the `query!` macro. **No database connection or SQLX_OFFLINE needed at compile time.** If this changes in the future, we'll need to add `.sqlx/` cached query data or a CI database.

### Runner

`ubuntu-latest` (currently Ubuntu 22.04 or 24.04). Standard, fast provisioning, free for public repos. No macOS runner needed — the codebase doesn't have platform-specific code beyond the `gtimeout` fallback in the test recipe, which isn't needed on Linux (GNU `timeout` is available).

### Branch Protection

Not configured in the workflow file — needs to be set up in GitHub UI:
- Require status checks to pass: `ci` workflow (both `rust` and `web` jobs)
- This is documented in the PR/review as a manual step for the repo owner.

### Scenario Dashboard Visibility

`just scenarios` output goes to stdout by default. GitHub Actions captures all stdout in the job log. No special formatting needed — the dashboard output will be visible in the "Run scenarios" step log.
