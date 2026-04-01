---
id: T-030-02
story: S-030
title: proposal-api-route
type: task
status: open
priority: high
phase: ready
depends_on: [T-030-01]
---

## Context

Wire the proposal generation pipeline into the API: load project data, compute quotes, generate narrative (mocked in tests), render PDF, return bytes.

## Acceptance Criteria

- `GET /projects/:id/proposal` route in plantastic-api
- Pipeline: load project + zones + materials + assignments → compute 3 quotes → generate narrative → render PDF → return
- Response: `Content-Type: application/pdf`, `Content-Disposition: attachment; filename="proposal-{project_name}.pdf"`
- Optional: upload to S3 and return presigned URL instead (for caching)
- Error handling: 404 (no project), 400 (no assignments), 500 (render failure)
- S.3.3 scenario test: create project → add zones → add materials → assign → GET /proposal → verify PDF
  - PDF starts with `%PDF-`
  - PDF contains expected dollar totals as strings (grep the raw bytes)
  - Uses MockProposalGenerator — zero LLM calls
- S.3.3 passes at ★★☆☆☆ integration (API returns valid PDF)
- Claim "pt-pdf: branded quote PDF generation" milestone in progress.rs
- `just check` passes

## Implementation Notes

- Same data loading pattern as existing GET /quote/:tier route, but loads all 3 tiers
- The mock narrative is injected via AppState's ProposalNarrativeGenerator trait object
- Consider caching: if proposal already exists in S3 for this project version, serve cached
- The PDF total verification in tests: search PDF bytes for the formatted total string (Typst embeds text as-is in the PDF content stream)
