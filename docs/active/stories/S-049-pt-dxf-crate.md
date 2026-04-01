---
id: S-049
epic: E-019
title: pt-dxf Crate
status: open
priority: high
tickets: [T-049-01, T-049-02]
---

## Goal

Create the pt-dxf crate that converts project zones + material assignments into DXF format. Zones become LWPOLYLINE entities on layers, materials become TEXT annotations, dimensions become DIMENSION entities.

## Acceptance Criteria

- Zone polygons → LWPOLYLINE on layer per zone type
- Material names + SKUs as TEXT entities near zone centroids
- Area and perimeter as DIMENSION annotations
- Title block with project name, date, company
- Output opens correctly in LibreCAD/QCAD (free, testable)
- Unit tests verify DXF structure without needing a CAD tool
