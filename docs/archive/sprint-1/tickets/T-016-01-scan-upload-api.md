---
id: T-016-01
story: S-016
title: scan-upload-processing-api
type: task
status: open
priority: high
phase: done
depends_on: [T-015-02, T-004-02]
---

## Context

Wire pt-scan to the API. A PLY file uploaded to a project triggers processing, stores the outputs in object storage, and links them to the project. The zone editor can then load the plan view PNG as its background.

## Acceptance Criteria

- POST /projects/:id/scan — accepts multipart PLY upload, stores raw PLY in S3
- Triggers pt-scan processing (async — enqueue job, return 202 with job ID)
- GET /projects/:id/scan/status — returns processing/complete/failed + output URLs
- On completion: terrain.glb, planview.png, metadata.json stored in S3
- Project.scan_ref updated with S3 keys for all three artifacts
- Plan view image accessible via GET /projects/:id/planview (serves PNG from S3)
- Zone editor loads plan view as canvas background when available
- Error handling: invalid PLY format, processing timeout, S3 upload failure
