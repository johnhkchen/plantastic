---
id: S-050
epic: E-019
title: DXF API Route & Download
status: open
priority: high
depends_on: [S-049]
tickets: [T-050-01]
---

## Goal

Wire pt-dxf into the API and add a download button. GET /projects/:id/dxf returns DXF bytes. The project page has an "Export DXF" button.

## Acceptance Criteria

- API route: GET /projects/:id/dxf → DXF bytes with Content-Disposition
- Download button on project page (alongside existing quote/viewer links)
- S.4.2 scenario passes at ★★☆☆☆
- Claim pt-dxf milestone in progress.rs
