# T-018-01 Plan — GitHub Actions CI

## Step 1: Create `.github/workflows/ci.yml`

Write the complete CI workflow file with:
- Trigger: `pull_request` to main + `push` to main
- Two parallel jobs: `rust` and `web`
- Rust job: checkout → setup-rust-toolchain → rust-cache → install just → fmt-check → lint → test → scenarios
- Web job: checkout → setup pnpm → setup node → pnpm install → check → lint
- Both jobs: `timeout-minutes: 15`

**Verification:** YAML is valid. Can be verified by reading the file and checking structure. Full verification requires pushing and opening a PR, which happens at review time.

## Step 2: Validate Workflow Locally

Run `just check` locally to confirm the commands the CI will execute still pass. This ensures we're not setting up CI that will immediately fail.

Run the web commands too: `cd web && pnpm install && pnpm run check && pnpm run lint`

**Verification:** Both command sets pass without errors.

## Step 3: Document Branch Protection Setup

Add a note to the review artifact about the manual branch protection step needed in GitHub UI. This can't be automated via the workflow file.

## Testing Strategy

This ticket's deliverable is a CI configuration file, not application code. Testing is:

1. **Syntactic correctness:** YAML is well-formed (validated by reading)
2. **Semantic correctness:** Commands match what `just check` and web scripts run (verified by manual comparison in review)
3. **Runtime correctness:** The workflow actually runs on GitHub Actions — this is verified when the first PR with this file is opened
4. **Performance:** < 10 minute target — verified after first run

No unit tests, integration tests, or scenario changes are needed. This ticket adds infrastructure, not application capabilities. No new scenarios are registered — CI doesn't directly deliver customer value, it protects it.
