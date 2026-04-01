---
id: E-019
title: DXF Export for Crew Handoff
status: open
priority: high
sprint: 3
---

## Context

Landscaping crews work with printed plans or CAD files. DXF is the universal exchange format — every CAD tool reads it. A DXF export with zone boundaries as LWPOLYLINE entities, material callouts as TEXT, and dimension annotations closes the loop from digital design to physical installation.

This is the last major gap in the Crew Handoff area (0.0 / 30.0 min currently). DXF export (S.4.2) plus the existing viewer (S.4.1 prereqs met) would push this area from 3% to meaningful coverage.

## Architecture

```
Project zones + material assignments + measurements
        │
        ▼
  pt-dxf: generate_dxf(project) → DxfOutput
    - Layer per zone type (PATIO, BED, EDGING, FILL)
    - LWPOLYLINE entities matching zone geometry
    - TEXT entities: zone labels, material names, supplier SKUs
    - DIMENSION entities: area (sqft), perimeter (ft)
    - Title block: project name, date, company, scale
        │
        ▼
  API route: GET /projects/:id/dxf → DXF bytes
  Download button on project page
```

## Stories

- S-049: pt-dxf Crate (zone geometry → DXF entities)
- S-050: DXF API Route & Download (wire into API, add UI button)

## Success Criteria

- DXF opens in AutoCAD, LibreCAD, QCAD with correct layers and geometry
- Zone boundaries match pt-geo measurements exactly
- Material callouts readable as text annotations
- Dimension annotations present for area and perimeter
- S.4.2 (DXF export) passes at ★★☆☆☆ (API returns valid DXF)
- Crew Handoff area advances from 1.0 to 11.0+ effective minutes
