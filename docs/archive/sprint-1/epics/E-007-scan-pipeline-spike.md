---
id: E-007
title: Scan Pipeline Foundation
status: open
sprint: 2
---

# E-007: Scan Pipeline Foundation

## Goal

Prove that we can process a PLY file from SiteScape into a usable terrain mesh and plan view image using Rust — no Python, no Open3D. Get to ★☆☆☆☆ for S.1.1 (scan processing).

This is the other critical technical risk. The Groundwork design doc assumed Python+Open3D for point cloud processing. We've decided Python is too slow for production. The Rust ecosystem for point cloud processing is thinner — we need to prove the pipeline works before building features that depend on terrain meshes.

## Target

- S.1.1 Scan processing: — → ★☆☆☆☆ (0.0 → 6.0 effective min)
- PLY file → decimated terrain mesh (glTF) + top-down image (PNG) + metadata (JSON)
- Processing completes in < 30 seconds for a typical residential scan (~5M points)

## Stories

- **S-015**: pt-scan — PLY parsing + point cloud processing + mesh generation
- **S-016**: Scan upload + processing job via API

## Success Criteria

- PLY parser reads SiteScape output files (binary and ASCII PLY formats)
- Point cloud filtering: statistical outlier removal, voxel downsampling
- Ground plane fitting (RANSAC or similar)
- Mesh generation: Delaunay triangulation of ground points
- Mesh decimation to ~50k triangles
- glTF binary export of terrain mesh
- Top-down orthographic projection → PNG (the plan view background for the zone editor)
- Metadata output: bounding box, elevation range, point count
- End-to-end test with a real SiteScape PLY file (we'll need a test fixture)
- S.1.1 scenario registered and passing at ★☆☆☆☆
