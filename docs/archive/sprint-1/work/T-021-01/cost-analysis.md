# Cost Analysis: Neon PostgreSQL for Plantastic

## Neon Pricing Tiers (as of 2026-Q1)

### Free Tier
- **Cost:** $0/mo
- **Storage:** 0.5 GiB
- **Compute:** 191.9 compute-hours/month (~0.25 CU always-on equivalent)
- **Branches:** 10
- **Scale-to-zero:** Yes (5-min default suspend)
- **PostGIS:** Yes
- **Limitations:** Single region, no IP allow-lists, community support only

### Launch Plan
- **Cost:** $19/mo
- **Storage:** 10 GiB included ($3.50/GiB-month overage)
- **Compute:** 300 compute-hours/month included ($0.16/CU-hour overage)
- **Branches:** 500
- **Scale-to-zero:** Yes (configurable suspend timeout)
- **Additional:** Logical replication, IP allow-lists, 24-hour support

### Scale Plan
- **Cost:** $69/mo
- **Storage:** 50 GiB included
- **Compute:** 750 compute-hours/month included
- **Branches:** 500
- **Additional:** Read replicas, higher connection limits

## Expected Monthly Cost for Plantastic

### Current Usage Pattern (dev stage)
- **Traffic:** Low — dev/staging only, sporadic API calls
- **Storage:** < 50 MiB (schema only, minimal test data)
- **Compute:** < 10 hours/month active (mostly suspended)
- **CI branches:** T-021-02 will create/destroy ephemeral branches per CI run

### Recommendation: Free Tier (now) → Launch (when needed)

**Free tier is sufficient** for the current dev stage:
- 0.5 GiB storage covers schema + dev data easily
- 191 compute-hours/month is more than enough for sporadic dev traffic
- 10 branches covers CI needs (branches are ephemeral, created and deleted per run)
- PostGIS included at all tiers

**Upgrade trigger:** When any of these occur:
- Storage exceeds 0.5 GiB (real customer data)
- CI branch usage exceeds 10 concurrent (unlikely with ephemeral pattern)
- Need IP allow-lists for production security
- Customer-facing traffic requires SLA guarantees

## Comparison to Railway

| Factor | Railway (grandfathered) | Neon Free | Neon Launch |
|---|---|---|---|
| Monthly cost | $5/mo | $0/mo | $19/mo |
| Storage | 1 GiB | 0.5 GiB | 10 GiB |
| PostGIS | Yes (manual) | Yes (first-class) | Yes |
| Scale-to-zero | No | Yes | Yes |
| Database branching | No | Yes (10) | Yes (500) |
| Co-location with Lambda | No (Railway infra) | Yes (us-west-2) | Yes |
| Connection pooling | Manual PgBouncer | Built-in (`-pooler`) | Built-in |

**Net effect:** Moving from Railway $5/mo to Neon free saves $5/mo while gaining branching (CI), scale-to-zero (Lambda cold starts), and region co-location (latency).

## Scale-to-Zero Behavior

Neon suspends compute after a configurable idle timeout (default: 5 minutes).

- **Suspend:** After 5 minutes with no active connections, compute suspends. No compute charges while suspended.
- **Wake:** First connection triggers compute wake. Takes 0.5-3s typically, up to 8s worst case.
- **Impact on Lambda:** Lambda cold-start may coincide with Neon wake. T-020-02's retry logic (15s timeout, 3 retries, exponential backoff) handles this. T-021-03 will validate timing.
- **Free tier:** Suspend timeout fixed at 5 minutes. Launch plan allows configuration (0 = always-on).

## Summary

Start on Neon Free tier. Current dev-stage usage is well within limits. The $5/mo Railway savings is minor, but the real wins are branching for CI, scale-to-zero for cost efficiency, and us-west-2 co-location for Lambda latency. Upgrade to Launch ($19/mo) when production traffic or security requirements demand it.
