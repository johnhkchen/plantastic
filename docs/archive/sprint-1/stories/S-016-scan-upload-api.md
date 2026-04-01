---
id: S-016
epic: E-007
title: Scan Upload & Processing API
status: open
priority: high
dependencies:
  - S-015
  - S-004
---

# S-016: Scan Upload & Processing API

## Purpose

Wire pt-scan into the API. A PLY file uploaded to the API gets processed into terrain mesh + plan view, stored in object storage, and linked to the project. This is the bridge between the phone scan and the zone editor.

## Scope

- POST /projects/:id/scan — upload PLY file (multipart form)
- Trigger pt-scan processing (async — could be seconds to minutes for large scans)
- GET /projects/:id/scan/status — job status (processing, complete, failed)
- Store outputs in S3/Minio: terrain.glb, planview.png, metadata.json
- Link scan_ref on the project record
- Plan view image served to zone editor as the drawing background

## Tickets

- T-016-01: Scan upload endpoint + async processing job
