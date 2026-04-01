---
id: E-015
title: Scenario Test Fixes & Database Integration
status: open
priority: critical
sprint: 2
---

## Context

Docker Compose (PG17 + PostGIS 3.5) is running and unblocked 5 previously-blocked scenarios. S.INFRA.1 passed immediately. Two scenarios still fail due to test assertion bugs — the API works correctly, but the test payloads or assertions are wrong. Fixing these flips 2 more scenarios to green, reaching 12/17 passing.

## Failures

| Scenario | Error | Root Cause |
|----------|-------|------------|
| S.3.3 | PDF doesn't contain '1530' | Typst PDF content stream encodes text as glyph IDs, not raw ASCII — `from_utf8_lossy` can't find it. Need to use a PDF text extraction library or verify the quote data at the API level instead. |
| S.INFRA.2 | POST /zones returns 422 | Test sends GeoJSON in wrong format — likely needs `geometry` wrapper or coordinate format mismatch. |

## Stories

- S-038: Fix Remaining Scenario Failures

## Success Criteria

- S.3.3 passes at ★★☆☆☆
- S.INFRA.2 passes at ★★☆☆☆
- 12 scenarios passing, 0 failing, 5 not implemented
- `DATABASE_URL=... just scenarios` clean (exit 0)
- `just check` (without DATABASE_URL) still passes (no regressions)
