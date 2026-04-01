---
id: S-040
epic: E-016
title: Classification Event Logging
status: open
priority: high
tickets: [T-040-01]
---

## Goal

Every BAML classification call (and every user correction) produces a structured JSONL event. This is the training data pipeline for model distillation — no extra collection effort, just a byproduct of normal operation.

## Event Types

```jsonl
{"type":"classification","scan_id":"...","candidates":[...],"results":[...],"context":{...},"timestamp":"..."}
{"type":"correction","scan_id":"...","feature_id":0,"before":{"label":"structure"},"after":{"label":"tree_trunk"},"context":{...}}
{"type":"zone_suggestion","project_id":"...","suggested":[...],"accepted":[...],"rejected":[...],"context":{...}}
```

## Acceptance Criteria

- Logging module with async, non-blocking writes
- Configurable via env var: `PLANTASTIC_EVENT_LOG=data/events/` (off by default in prod)
- Per-tenant isolation (events tagged with tenant_id, deletable per GDPR)
- JSONL format (one event per line, trivial to process with jq/Python)
- Works in both local dev and Lambda (Lambda: log to CloudWatch, local: log to file)
