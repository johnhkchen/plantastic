# T-018-02 Design — Deploy Pipeline & Domain

## Decision 1: Workflow Structure

### Option A: Single deploy job (sequential steps)
One job that builds, deploys infra, deploys web, deploys worker, runs smoke tests. Simple, no artifact passing between jobs.

### Option B: Multiple jobs with artifacts
Separate build/deploy/smoke jobs connected by `needs:` and artifact uploads. More parallel but adds complexity for artifact transfer (Lambda binary is ~15 MB).

### Option C: Reusable workflow called from CI
Extend `ci.yml` with a deploy job that runs after CI passes on push to main.

**Decision: Option A.** The deploy steps are inherently sequential (Lambda must deploy before Worker secret can be set). Multiple jobs would add artifact upload/download overhead with no parallelism gain. A separate `deploy.yml` file keeps concerns clean — CI validates, deploy ships.

The workflow triggers only on `push` to `main` (not PRs) and requires CI to pass first via `workflow_run` or simply runs independently (since CI also triggers on push to main, they run in parallel, but deploy failures are caught by smoke tests).

**Revised: use `needs` in the same file? No** — CI is its own workflow file. Using `workflow_run` to trigger deploy after CI succeeds adds complexity and delay. Instead, deploy.yml triggers on push to main independently. The CI workflow catches broken code on PRs before merge. If someone force-pushes broken code to main, the smoke test catches it. This matches the ticket's intent: "smoke test failure notifies but doesn't rollback."

## Decision 2: Cross-Compilation Setup in CI

### Option A: cargo-zigbuild (same as local)
Install zig + cargo-zigbuild in CI. Matches the local build script exactly.

### Option B: cross (Docker-based cross-compilation)
Uses pre-built Docker images. Heavier but more hermetic.

### Option C: Native build on AL2023 container
Run the build in an Amazon Linux 2023 container. No cross-compilation needed but different from local workflow.

**Decision: Option A.** `cargo-zigbuild` is already the local workflow (`scripts/build-lambda.sh`). Installing zig on Ubuntu is one apt/brew command. `cargo-zigbuild` installs via cargo. This keeps CI and local builds identical — same script, same toolchain. The build script already exists and works.

## Decision 3: SST Output Extraction

### Option A: Parse stdout from `sst deploy`
Grep the deploy output for the Lambda URL. Fragile — output format may change.

### Option B: Use `sst output` command after deploy
SST v3 has an output command to read stack outputs. More reliable.

### Option C: Use AWS CLI to read the Function URL
`aws lambda get-function-url-configuration`. Works but adds AWS CLI dependency.

**Decision: Option A with fallback.** Parse the SST deploy output first since we're already running the command. The output format is stable enough (`apiUrl: https://...`). If parsing fails, the workflow step fails and the deploy stops before wiring the wrong URL.

Actually — SST v3 outputs are printed as JSON-like key-value pairs. We can capture the full output and extract `apiUrl` reliably. We'll use `sst deploy` output directly since we need to run the command anyway.

## Decision 4: Smoke Test Target

The ticket says "tests through the CF Worker URL (not direct Lambda)." The Worker URL after deploy is `plantastic-api-proxy.<subdomain>.workers.dev`. We can get this from wrangler output or hardcode it as a known value.

**Decision:** Use the Worker URL. After `wrangler deploy`, the Worker URL is printed. We capture it and pass to `verify-deploy.sh`. The existing script already accepts any URL — no changes needed to the script itself.

## Decision 5: Domain Setup

DNS and custom domain binding are one-time manual operations in the Cloudflare dashboard. The workflow doesn't automate domain setup — it deploys code to projects that already have domains bound.

**Decision:** Create a `docs/active/work/T-018-02/setup-domain.md` guide documenting the manual steps. The workflow assumes the CF Pages project and Worker are already configured with custom domains. This matches how T-017-02 handled Neon setup (manual guide + automated pipeline).

## Decision 6: Notification on Smoke Failure

### Option A: GitHub Actions built-in — job failure is visible in Actions tab
### Option B: Slack/email notification step
### Option C: GitHub issue creation on failure

**Decision: Option A.** Job failure already shows in the GitHub Actions UI with email notifications for the repo owner. Adding Slack/issue creation is over-engineering for now. The ticket says "notifies but doesn't rollback" — the `continue-on-error: true` on the smoke step plus a follow-up notification step handles this.

## Decision 7: Updated Justfile Recipes

The ticket wants:
- `just deploy dev` — runs full pipeline locally
- `just smoke` — runs smoke tests against deployed environment

Current `just deploy` is missing the Lambda build step and BACKEND_URL wiring. Update it.
`just smoke` is new — takes a URL argument and runs `verify-deploy.sh`.

## Architecture Summary

```
Push to main
  → deploy.yml triggers
  → Install toolchain (Rust, zig, Node, pnpm)
  → Build Lambda (scripts/build-lambda.sh)
  → SST deploy (Lambda + S3) → capture apiUrl
  → Build SvelteKit (pnpm build)
  → Deploy CF Pages (wrangler pages deploy)
  → Set Worker secret BACKEND_URL = apiUrl
  → Deploy CF Worker (wrangler deploy)
  → Smoke test through Worker URL
  → If smoke fails: job marked as failed, notification sent, no rollback
```

## Rejected Alternatives

- **Terraform instead of SST**: SST is already in use, working, and matches the team's stack. No reason to switch.
- **AWS SAM/CDK**: Same — SST is the chosen IaC tool.
- **Docker-based Lambda**: The Rust binary + provided.al2023 runtime is simpler and faster than container images.
- **Blue/green deployment**: Over-engineering for dev stage. Revisit for production.
- **Automatic rollback on smoke failure**: Ticket explicitly says manual investigation. Rollback logic adds complexity and risk.
