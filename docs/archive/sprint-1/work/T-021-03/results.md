# T-021-03 Results: Lambda → Neon Connection Validation

> Fill in after running `just validate-neon-lambda <api-url>`

## Test environment

- **Date:** YYYY-MM-DD
- **Lambda region:** us-west-2
- **Lambda memory:** 256 MB
- **Neon project:** (project ID)
- **Neon plan:** Free / Launch
- **Neon region:** aws-us-west-2
- **Connection:** pooled endpoint (`-pooler` suffix)

## Cold start timing

| Phase | Time (ms) | Notes |
|-------|-----------|-------|
| Lambda init (INIT duration from CloudWatch) | | Rust binary load |
| Neon compute wake | | If suspended; 0 if warm |
| TLS handshake | | `sslnegotiation=direct` saves ~50-100ms |
| First query (SELECT 1) | | Via pool.rs retry logic |
| **Total /health/ready** | | End-to-end from client |

## Warm request timing

| Metric | Time (ms) |
|--------|-----------|
| /health/ready (warm) | |

## Concurrent cold starts (10 parallel)

| Instance | HTTP code | Total (ms) |
|----------|-----------|------------|
| 1 | | |
| 2 | | |
| 3 | | |
| 4 | | |
| 5 | | |
| 6 | | |
| 7 | | |
| 8 | | |
| 9 | | |
| 10 | | |
| **Max** | | |

## Idle recovery (10-minute idle)

| Metric | Time (ms) | Notes |
|--------|-----------|-------|
| Post-idle /health/ready | | Neon should have suspended |

## Retry behavior

- Retries observed in CloudWatch? (yes/no)
- If yes, how many attempts before success?
- Log snippet:

```
(paste relevant CloudWatch log lines)
```

## Tuning applied

- [ ] `sslnegotiation=direct` in connection string
- [ ] `statement_cache_size=0` for pooled endpoint
- [ ] Neon suspend timeout adjusted (default 5 min)
- [ ] Lambda provisioned concurrency (if needed)
- [ ] Pool `connect_timeout` adjusted from 15s default
- Other:

## Acceptance criteria verdict

| Criterion | Pass/Fail | Notes |
|-----------|-----------|-------|
| Cold start health < 5s | | |
| Retry handles Neon cold-start | | |
| No hangs (sqlx/tokio-postgres issue) | | |
| 10 concurrent cold starts succeed | | |
| Post-idle recovery works | | |

## Conclusion

(Summary: pass/fail, any concerns, recommended tuning)
