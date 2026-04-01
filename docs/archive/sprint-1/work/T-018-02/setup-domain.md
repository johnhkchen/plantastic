# Domain & Secrets Setup Guide — T-018-02

One-time manual steps required before the deploy pipeline works.

## 1. GitHub Repository Secrets

Go to **Settings → Secrets and variables → Actions** in the GitHub repo.

Add these repository secrets:

| Secret | Value | Source |
|--------|-------|--------|
| `AWS_ACCESS_KEY_ID` | IAM user access key | AWS IAM console — create user `plantastic-deploy` with Lambda, S3, CloudFormation, SSM permissions |
| `AWS_SECRET_ACCESS_KEY` | IAM user secret key | Same IAM user |
| `CLOUDFLARE_API_TOKEN` | API token with Pages + Workers edit | Cloudflare dashboard → My Profile → API Tokens → Create Token |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare account ID | Cloudflare dashboard → any domain → Overview → right sidebar |

### IAM Policy (minimum)

The deploy IAM user needs these AWS managed policies (or equivalent custom policy):
- `AWSLambda_FullAccess`
- `AmazonS3FullAccess`
- `AWSCloudFormationFullAccess`
- `AmazonSSMReadOnlyAccess`
- `IAMFullAccess` (SST manages Lambda execution roles)

For production, scope these down to the `plantastic-*` resource prefix.

### Cloudflare API Token Permissions

Create a custom token with:
- **Account** → Cloudflare Pages → Edit
- **Account** → Workers Scripts → Edit
- **Account** → Workers Routes → Edit
- **Zone** → DNS → Edit (for get-plantastic.com zone)

## 2. SST Secrets (per stage)

Set the database URL for each deployment stage:

```bash
cd infra

# Dev stage
npx sst secret set DatabaseUrl "postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/neondb?sslmode=require&connect_timeout=5" --stage dev

# Production stage (when ready)
npx sst secret set DatabaseUrl "postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/neondb?sslmode=require&connect_timeout=5" --stage production
```

## 3. Cloudflare Pages Project

```bash
# Create the Pages project (one-time)
cd web
npx wrangler pages project create plantastic

# Bind custom domain (via dashboard)
# Cloudflare dashboard → Pages → plantastic → Custom domains → Add
# staging.get-plantastic.com → Add domain
```

Or via the Cloudflare dashboard:
1. Go to **Workers & Pages → Create → Pages**
2. Connect to Git is optional — we deploy via wrangler CLI
3. Project name: `plantastic`
4. After creation: **Custom domains → Add custom domain → staging.get-plantastic.com**

## 4. Cloudflare DNS

Domain `get-plantastic.com` must be on Cloudflare DNS.

If the domain is already registered elsewhere:
1. Add the domain in Cloudflare dashboard
2. Update nameservers at the registrar to Cloudflare's NS

DNS records (added automatically by CF Pages custom domain binding):
- `staging` CNAME → `plantastic.pages.dev` (proxied)

Optional additional records:
- `api` CNAME → `plantastic-api-proxy.<account>.workers.dev` (if you want api.get-plantastic.com)
- `@` A/CNAME → landing page (future)

## 5. SSL/TLS

Cloudflare handles SSL automatically for proxied records:
1. Go to **SSL/TLS → Overview** for get-plantastic.com
2. Set mode to **Full (strict)**
3. Edge certificates are auto-provisioned

No additional cert management needed.

## 6. Database Migrations

Run migrations against the Neon database before first deploy:

```bash
# Via Doppler (if configured)
doppler run -- ./scripts/migrate.sh

# Or directly
DATABASE_URL="postgres://..." ./scripts/migrate.sh
```

## 7. Verify Setup

After completing all steps:

```bash
# Manual deploy
just deploy dev

# Smoke test
just smoke https://plantastic-api-proxy.<account-subdomain>.workers.dev
```

If smoke tests pass, push to main and verify the GitHub Actions deploy workflow runs successfully.
