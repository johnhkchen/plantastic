---
id: S-015
epic: E-007
title: pt-scan — PLY Processing Pipeline
status: open
priority: critical
---

# S-015: pt-scan — PLY Processing Pipeline

## Purpose

Build the Rust point cloud processing pipeline that replaces the Python+Open3D approach. This is the entry point of the entire product — without it, there's no terrain mesh, no plan view, no geometry to draw zones on.

## Scope

### PLY parsing
- Read binary and ASCII PLY formats (SiteScape exports binary)
- Extract vertex positions (x, y, z) and optionally colors/normals

### Point cloud processing
- Statistical outlier removal (remove noise points)
- Voxel downsampling (reduce density while preserving shape)
- RANSAC ground plane fitting (separate ground from objects)

### Mesh generation
- Delaunay triangulation of ground points → triangle mesh
- Mesh decimation to target triangle count (~50k)

### Output generation
- glTF binary export of terrain mesh (with vertex colors if available)
- Top-down orthographic projection → PNG image (plan view)
- Metadata JSON: bounding box, elevation range, original/decimated point counts

### Crate dependencies to evaluate
- `ply-rs` for PLY parsing
- `nalgebra` for linear algebra
- `delaunator` for Delaunay triangulation
- `gltf` crate for glTF export
- `image` crate for PNG output

## Tickets

- T-015-01: PLY parsing + point cloud filtering
- T-015-02: Mesh generation + glTF/PNG export
