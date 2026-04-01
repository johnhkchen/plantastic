---
id: S-030
epic: E-013
title: Typst PDF Rendering
status: open
priority: high
depends_on: [S-029]
tickets: [T-030-01, T-030-02]
---

## Goal

Create pt-proposal crate that merges pt-quote output with BAML narrative and renders a branded PDF via Typst. Add API route for PDF download.

## Acceptance Criteria

- pt-proposal crate: takes Quote (×3 tiers) + ProposalContent + TenantBranding → PDF bytes
- Typst template: professional 3-tier comparison layout with line items, zone details, totals
- API route: GET /projects/:id/proposal → PDF (with mock narrative in tests)
- S.3.3 scenario passes at ★★☆☆☆ (API returns valid PDF with correct totals)
- Template supports tenant branding hooks (logo placeholder, primary color)
