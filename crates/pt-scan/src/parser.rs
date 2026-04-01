//! PLY file parsing.
//!
//! Reads binary (little-endian, big-endian) and ASCII PLY files, extracting
//! vertex positions (x, y, z) and optional colors (red, green, blue).

use std::io::{BufRead, BufReader, Read};

use ply_rs_bw::parser::Parser;
use ply_rs_bw::ply::{DefaultElement, Property};

use crate::error::ScanError;
use crate::types::Point;

/// Parse a PLY file and return vertices as Points.
///
/// Expects a "vertex" element with float properties "x", "y", "z".
/// Optionally reads "red", "green", "blue" as uchar color properties.
///
/// # Errors
///
/// Returns `ScanError::InvalidPly` if the PLY header is malformed or the
/// vertex element is missing required properties.
pub fn parse_ply(reader: impl Read) -> Result<Vec<Point>, ScanError> {
    let mut reader = BufReader::new(reader);
    parse_ply_buffered(&mut reader)
}

fn parse_ply_buffered(reader: &mut impl BufRead) -> Result<Vec<Point>, ScanError> {
    let vertex_parser = Parser::<DefaultElement>::new();
    let header = vertex_parser
        .read_header(reader)
        .map_err(|e| ScanError::InvalidPly(format!("failed to read PLY header: {e}")))?;

    let mut points = Vec::new();

    for (name, element) in &header.elements {
        if name == "vertex" {
            let payload = vertex_parser
                .read_payload_for_element(reader, element, &header)
                .map_err(|e| ScanError::InvalidPly(format!("failed to read vertex data: {e}")))?;

            points.reserve(payload.len());

            for vertex in &payload {
                let x = extract_float(vertex, "x")?;
                let y = extract_float(vertex, "y")?;
                let z = extract_float(vertex, "z")?;

                let color = match (
                    extract_uchar(vertex, "red"),
                    extract_uchar(vertex, "green"),
                    extract_uchar(vertex, "blue"),
                ) {
                    (Ok(r), Ok(g), Ok(b)) => Some([r, g, b]),
                    _ => None,
                };

                points.push(Point {
                    position: [x, y, z],
                    color,
                });
            }

            // Only process the first vertex element
            break;
        }
    }

    Ok(points)
}

fn extract_float(
    props: &indexmap::IndexMap<String, Property>,
    name: &str,
) -> Result<f32, ScanError> {
    match props.get(name) {
        Some(Property::Float(v)) => Ok(*v),
        Some(Property::Double(v)) => {
            #[allow(clippy::cast_possible_truncation)]
            let val = *v as f32;
            Ok(val)
        }
        Some(other) => Err(ScanError::InvalidPly(format!(
            "property '{name}' has unexpected type: {other:?}"
        ))),
        None => Err(ScanError::InvalidPly(format!(
            "missing required property '{name}'"
        ))),
    }
}

fn extract_uchar(
    props: &indexmap::IndexMap<String, Property>,
    name: &str,
) -> Result<u8, ScanError> {
    match props.get(name) {
        Some(Property::UChar(v)) => Ok(*v),
        _ => Err(ScanError::InvalidPly(format!(
            "missing or invalid uchar property '{name}'"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;
    use std::io::Cursor;

    /// Generate a binary little-endian PLY file in memory.
    fn make_binary_ply(points: &[(f32, f32, f32, u8, u8, u8)]) -> Vec<u8> {
        let mut buf = Vec::new();

        // Header
        let header = format!(
            "ply\n\
             format binary_little_endian 1.0\n\
             element vertex {}\n\
             property float x\n\
             property float y\n\
             property float z\n\
             property uchar red\n\
             property uchar green\n\
             property uchar blue\n\
             end_header\n",
            points.len()
        );
        buf.extend_from_slice(header.as_bytes());

        // Binary vertex data
        for &(x, y, z, r, g, b) in points {
            buf.extend_from_slice(&x.to_le_bytes());
            buf.extend_from_slice(&y.to_le_bytes());
            buf.extend_from_slice(&z.to_le_bytes());
            buf.push(r);
            buf.push(g);
            buf.push(b);
        }

        buf
    }

    /// Generate an ASCII PLY file in memory.
    fn make_ascii_ply(points: &[(f32, f32, f32, u8, u8, u8)]) -> Vec<u8> {
        let mut s = format!(
            "ply\n\
             format ascii 1.0\n\
             element vertex {}\n\
             property float x\n\
             property float y\n\
             property float z\n\
             property uchar red\n\
             property uchar green\n\
             property uchar blue\n\
             end_header\n",
            points.len()
        );
        for &(x, y, z, r, g, b) in points {
            s.push_str(&format!("{x} {y} {z} {r} {g} {b}\n"));
        }
        s.into_bytes()
    }

    /// Generate a binary PLY without color properties.
    fn make_binary_ply_no_color(points: &[(f32, f32, f32)]) -> Vec<u8> {
        let mut buf = Vec::new();

        let header = format!(
            "ply\n\
             format binary_little_endian 1.0\n\
             element vertex {}\n\
             property float x\n\
             property float y\n\
             property float z\n\
             end_header\n",
            points.len()
        );
        buf.extend_from_slice(header.as_bytes());

        for &(x, y, z) in points {
            buf.extend_from_slice(&x.to_le_bytes());
            buf.extend_from_slice(&y.to_le_bytes());
            buf.extend_from_slice(&z.to_le_bytes());
        }

        buf
    }

    #[test]
    fn test_parse_binary_ply() {
        timed(|| {
            let data: Vec<(f32, f32, f32, u8, u8, u8)> = (0..100)
                .map(|i| {
                    let f = i as f32;
                    (f * 0.1, f * 0.2, f * 0.05, 128, 64, 32)
                })
                .collect();

            let ply_bytes = make_binary_ply(&data);
            let points = parse_ply(Cursor::new(ply_bytes)).unwrap();

            // Independently computed: we created 100 points
            assert_eq!(points.len(), 100);

            // First point: (0.0, 0.0, 0.0) with color (128, 64, 32)
            assert_eq!(points[0].position, [0.0, 0.0, 0.0]);
            assert_eq!(points[0].color, Some([128, 64, 32]));

            // Last point: index 99 → (9.9, 19.8, 4.95)
            let last = &points[99];
            assert!((last.position[0] - 9.9).abs() < 0.01);
            assert!((last.position[1] - 19.8).abs() < 0.01);
            assert!((last.position[2] - 4.95).abs() < 0.01);
        });
    }

    #[test]
    fn test_parse_ascii_ply() {
        timed(|| {
            let data: Vec<(f32, f32, f32, u8, u8, u8)> = (0..50)
                .map(|i| {
                    let f = i as f32;
                    (f, f * 2.0, f * 0.5, 255, 0, 128)
                })
                .collect();

            let ply_bytes = make_ascii_ply(&data);
            let points = parse_ply(Cursor::new(ply_bytes)).unwrap();

            assert_eq!(points.len(), 50);
            assert_eq!(points[0].position, [0.0, 0.0, 0.0]);
            assert_eq!(points[0].color, Some([255, 0, 128]));

            // Point 10: (10.0, 20.0, 5.0)
            assert!((points[10].position[0] - 10.0).abs() < 0.01);
            assert!((points[10].position[1] - 20.0).abs() < 0.01);
            assert!((points[10].position[2] - 5.0).abs() < 0.01);
        });
    }

    #[test]
    fn test_parse_missing_color() {
        timed(|| {
            let data = vec![(1.0_f32, 2.0_f32, 3.0_f32), (4.0, 5.0, 6.0)];
            let ply_bytes = make_binary_ply_no_color(&data);
            let points = parse_ply(Cursor::new(ply_bytes)).unwrap();

            assert_eq!(points.len(), 2);
            assert_eq!(points[0].position, [1.0, 2.0, 3.0]);
            assert_eq!(points[0].color, None);
            assert_eq!(points[1].position, [4.0, 5.0, 6.0]);
            assert_eq!(points[1].color, None);
        });
    }

    #[test]
    fn test_parse_empty_ply() {
        timed(|| {
            let ply_bytes = make_binary_ply(&[]);
            let points = parse_ply(Cursor::new(ply_bytes)).unwrap();
            assert!(points.is_empty());
        });
    }
}
