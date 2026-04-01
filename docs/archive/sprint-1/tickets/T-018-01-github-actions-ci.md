---
id: T-018-01
story: S-018
title: github-actions-ci
type: task
status: open
priority: high
phase: done
depends_on: [T-017-01]
---

## Context

Every PR must pass `just check` (format, lint, test, scenarios) before merge. This is the automated quality gate — no human needs to remember to run it.

## Acceptance Criteria

- `.github/workflows/ci.yml` runs on every PR to main
- Steps: install Rust toolchain, install just, `just fmt-check`, `just lint`, `just test`, `just scenarios`
- Rust toolchain cached (actions-rs/toolchain or similar)
- Cargo build artifacts cached between runs
- SvelteKit lint/check also runs (npm ci + npm run check in web/)
- `just scenarios` output visible in CI logs (the value dashboard)
- PR blocked if any step fails (branch protection rule)
- Runs in < 10 minutes (adjust caching if longer)
