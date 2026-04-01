---
id: T-018-02
story: S-018
title: deploy-pipeline-domain
type: task
status: open
priority: high
phase: done
depends_on: [T-017-02, T-018-01]
---

## Context

Automated deployment on merge to main, domain setup, and smoke tests. After this ticket, pushing to main results in a live deployment at get-plantastic.com (or staging subdomain) within minutes.

## Acceptance Criteria

### Deploy pipeline
- `.github/workflows/deploy.yml` runs on push to main
- Builds Rust for Lambda via `scripts/build-lambda.sh`
- Deploys Lambda + S3 via `npx sst deploy --stage dev`
- Deploys SvelteKit to CF Pages via `npx wrangler pages deploy`
- Deploys CF Worker via `npx wrangler deploy`
- Extracts Lambda Function URL and wires it to CF Worker's LAMBDA_URL secret

### Domain
- get-plantastic.com DNS on Cloudflare
- CF Pages custom domain: staging.get-plantastic.com (or app.get-plantastic.com)
- SSL/TLS configured (Cloudflare handles this)

### Smoke tests
- `scripts/verify-deploy.sh` runs after deploy
- Hits health endpoint → 200
- Creates a project → 201
- Fetches it back → 200 with matching data
- Deletes it → cleanup
- Tests through the CF Worker URL (not direct Lambda)
- Smoke test failure notifies but doesn't rollback (manual investigation)

### Updated recipes
- `just deploy dev` runs the full pipeline locally (for manual deploys)
- `just smoke` runs smoke tests against a deployed environment
