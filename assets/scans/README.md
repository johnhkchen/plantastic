# LiDAR Scan Assets

## Directory Structure

```
scans/
├── samples/          ← Drop PLY files here for development and demos
│   └── *.ply         ← iPhone LiDAR / public dataset scans
└── README.md
```

## Sample Scans

Place PLY files in `samples/`. These are used by:
- `pt-scan` unit and integration tests
- The scan-to-design proof of concept pipeline
- Demo recordings and screenshots

## Format

PLY files should contain:
- Vertex positions (x, y, z as float)
- Optional vertex colors (red, green, blue as uchar)
- Binary little-endian or ASCII format

iPhone LiDAR scans from apps like Polycam, 3d Scanner App, or SiteScape
export in this format natively.

## Size guidance

- Small test scans: < 1 MB (synthetic or heavily downsampled)
- Real residential scans: 5-50 MB (typical iPhone LiDAR output)
- Large files: add to `.gitignore` and use S3 for CI

Files over 50 MB should not be committed to git.
