---
id: E-008
title: Deployment Pipeline
status: open
sprint: 2
---

# E-008: Deployment Pipeline

## Goal

Deploy the existing stack to production infrastructure. A real URL that serves the SvelteKit app, proxies through the CF Worker to a Lambda-hosted Rust API backed by Neon PostGIS (us-west-2), with S3 for object storage. CI runs `just check` on every PR and deploys on merge to main.

This is not "deploy when features are ready." This is "deploy what exists now and ship incrementally." The sooner we have a real deployment, the sooner we catch deployment-specific bugs (cold starts, binary size, connection pooling under Lambda's short-lived model, CORS in prod, PostGIS wire format).

## Infrastructure

| Component | Provider | Cost |
|-----------|----------|------|
| PostgreSQL + PostGIS | Neon (Launch plan, us-west-2) | Free tier* |
| API (Rust Lambda) | AWS Lambda arm64 | Free tier |
| Object storage | S3 | Free tier |
| Frontend | Cloudflare Pages | Free |
| Proxy | Cloudflare Worker | Free |
| DNS | Cloudflare (get-plantastic.com) | Domain only |
| Secrets | Doppler (dev) + AWS SSM (prod) | Free |
| IaC | SST (Lambda + S3 only, not database) | — |

Neon manages the database — no RDS, no SST database resources, no VPC complexity. DATABASE_URL from Neon goes into Doppler/SSM. Lambda connects over the public internet with SSL. Neon provides built-in PgBouncer (pooled endpoint), database branching for CI, and co-location with Lambda in us-west-2.

*Neon Free tier includes 0.5 GiB storage, 190 compute hours/month. Scale-to-zero suspends compute after 5 min idle.

## What exists already

- SST config stub (from T-004-01)
- Axum API with full CRUD routes and Lambda runtime detection
- SvelteKit app with CF Pages adapter
- CF Worker proxy with CORS, rate limiting, SSE passthrough
- PostGIS migrations (6 up/down pairs)
- `just deploy` recipe (placeholder)
- Proven deployment pattern from HMW Workshop

## What's needed

1. **Rust cross-compilation** for Lambda arm64 (aarch64-unknown-linux-gnu) — zig cc or cross
2. **SST config** fleshed out: Lambda function (provided.al2023, arm64, RESPONSE_STREAM), S3 bucket for artifacts
3. **Neon PostGIS** — ~~enable PostGIS extension, apply migrations, extract DATABASE_URL~~ Done (T-021-01)
4. **Doppler/SSM secrets** — DATABASE_URL, S3 credentials, any API keys
5. **CF Pages** deployment of SvelteKit build
6. **CF Worker** deployment with Lambda Function URL wired in
7. **Domain** get-plantastic.com pointed at CF Pages (with staging subdomain)
8. **CI pipeline** (GitHub Actions) — `just check` + `just scenarios` on PR, deploy on merge
9. **Smoke tests** hitting the deployed stack end-to-end

## Stories

- **S-017**: Infrastructure provisioning — Lambda + S3 via SST, Railway PostGIS, secrets, domain
- **S-018**: CI/CD pipeline + deploy automation + smoke tests

## Success Criteria

- `just deploy dev` builds Rust for Lambda, deploys via SST, deploys frontend + worker
- API health check responds at the deployed Lambda Function URL
- Create a project via the deployed API → round-trips through Neon PostGIS
- SvelteKit app loads at staging.get-plantastic.com (or similar)
- CF Worker proxies requests correctly (CORS, rate limiting, auth passthrough)
- GitHub Actions runs `just check` on every PR and blocks merge on failure
- `just scenarios` runs in CI and reports the value dashboard
- S.INFRA.1 can be tested against the deployed stack (not just localhost)
