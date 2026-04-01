//! Annotated plan-view overlays: bounding boxes, labels, and category colors.
//!
//! Takes a base plan-view PNG (from [`crate::export::generate_terrain`]) plus
//! classified feature data and produces a professional-looking annotated site plan.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use ab_glyph::{FontRef, PxScale};
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use std::io::Cursor;

use crate::error::ScanError;
use crate::export::world_to_pixel;
use crate::feature::FeatureCandidate;
use crate::types::BoundingBox;

/// Embedded monospace font for label rendering.
const FONT_BYTES: &[u8] = include_bytes!("../../../assets/fonts/DejaVuSansMono.ttf");

/// Spatial + label data for one feature annotation overlay.
#[derive(Debug, Clone)]
pub struct FeatureAnnotation {
    /// Formatted display label, e.g. "London Plane (0.92)".
    pub label: String,
    /// Feature category: "tree", "structure", "hardscape", "utility".
    pub category: String,
    /// World-space XY bounding box minimum.
    pub bbox_min: [f64; 2],
    /// World-space XY bounding box maximum.
    pub bbox_max: [f64; 2],
    /// World-space XY centroid.
    pub centroid: [f64; 2],
}

/// Configuration for annotation rendering.
#[derive(Debug, Clone)]
pub struct AnnotationConfig {
    /// Font pixel scale (default: 14.0).
    pub font_scale: f32,
    /// Bounding box outline width in pixels (default: 2).
    pub box_line_width: u32,
    /// Padding around label text in pixels (default: 4).
    pub label_padding: u32,
}

impl Default for AnnotationConfig {
    fn default() -> Self {
        Self {
            font_scale: 14.0,
            box_line_width: 2,
            label_padding: 4,
        }
    }
}

/// Build a [`FeatureAnnotation`] by joining candidate geometry with classified labels.
///
/// The label is formatted as `"{label} ({confidence:.2})"`.
/// Spatial data comes from the candidate; display data from the classified feature.
///
/// # Panics
///
/// Does not panic. If `cluster_id` doesn't match, the function still produces
/// output using the provided data (caller is responsible for matching).
pub fn feature_annotation<C: ClassifiedFeatureRef>(
    candidate: &FeatureCandidate,
    classified: &C,
) -> FeatureAnnotation {
    FeatureAnnotation {
        label: format!("{} ({:.2})", classified.label(), classified.confidence()),
        category: classified.category().to_string(),
        bbox_min: [candidate.bbox_min[0], candidate.bbox_min[1]],
        bbox_max: [candidate.bbox_max[0], candidate.bbox_max[1]],
        centroid: [candidate.centroid[0], candidate.centroid[1]],
    }
}

/// Trait to abstract over ClassifiedFeature without depending on baml_client types.
pub trait ClassifiedFeatureRef {
    fn label(&self) -> &str;
    fn category(&self) -> &str;
    fn confidence(&self) -> f64;
    fn cluster_id(&self) -> i64;
}

/// Map a category string to its display color.
pub fn category_color(category: &str) -> Rgb<u8> {
    match category {
        "tree" | "planting" => Rgb([34, 197, 94]), // #22c55e
        "structure" => Rgb([107, 114, 128]),       // #6b7280
        "hardscape" => Rgb([217, 119, 6]),         // #d97706
        "utility" => Rgb([239, 68, 68]),           // #ef4444
        _ => Rgb([156, 163, 175]),                 // #9ca3af (fallback)
    }
}

/// Overlay annotations on a base plan-view PNG.
///
/// Decodes the base PNG, draws bounding boxes and labels for each feature,
/// then re-encodes to PNG. Features are drawn in order; label collision
/// avoidance shifts overlapping labels downward.
///
/// # Errors
///
/// Returns `ScanError::ExportError` if PNG decoding or encoding fails.
pub fn annotate_plan_view_png(
    base_png: &[u8],
    features: &[FeatureAnnotation],
    bbox: &BoundingBox,
    config: &AnnotationConfig,
) -> Result<Vec<u8>, ScanError> {
    let img = image::load_from_memory(base_png)
        .map_err(|e| ScanError::ExportError(format!("failed to decode base PNG: {e}")))?
        .to_rgb8();

    let (img_w, img_h) = (img.width(), img.height());

    if features.is_empty() {
        return encode_png(&img);
    }

    let font = FontRef::try_from_slice(FONT_BYTES)
        .map_err(|e| ScanError::ExportError(format!("failed to load font: {e}")))?;
    let scale = PxScale::from(config.font_scale);

    let mut img = img;
    let mut placed_labels: Vec<(i32, i32, i32, i32)> = Vec::new();

    for feature in features {
        let color = category_color(&feature.category);

        // Draw bounding box
        let (min_px, min_py) = world_to_pixel(
            feature.bbox_min[0] as f32,
            feature.bbox_max[1] as f32, // max Y in world = min Y in pixel (flipped)
            bbox,
            img_w,
            img_h,
        );
        let (max_px, max_py) = world_to_pixel(
            feature.bbox_max[0] as f32,
            feature.bbox_min[1] as f32, // min Y in world = max Y in pixel (flipped)
            bbox,
            img_w,
            img_h,
        );

        let rect_x = (min_px as i32).max(0);
        let rect_y = (min_py as i32).max(0);
        let rect_w = ((max_px - min_px) as u32)
            .max(1)
            .min(img_w.saturating_sub(rect_x as u32));
        let rect_h = ((max_py - min_py) as u32)
            .max(1)
            .min(img_h.saturating_sub(rect_y as u32));

        // Draw filled bbox with semi-transparent effect (blend with existing pixels)
        draw_bbox_tinted(&mut img, rect_x, rect_y, rect_w, rect_h, color);

        // Draw outline
        for thickness in 0..config.box_line_width {
            let t = thickness as i32;
            if rect_w > 2 * thickness && rect_h > 2 * thickness {
                draw_hollow_rect_mut(
                    &mut img,
                    Rect::at(rect_x + t, rect_y + t)
                        .of_size(rect_w - 2 * thickness, rect_h - 2 * thickness),
                    color,
                );
            }
        }

        // Draw label above bounding box
        let label = &feature.label;
        let char_width = (config.font_scale * 0.6) as i32;
        let label_w = (label.len() as i32) * char_width + (config.label_padding as i32) * 2;
        let label_h = config.font_scale as i32 + (config.label_padding as i32) * 2;

        // Default position: centered above bbox
        let default_x = rect_x + (rect_w as i32 - label_w) / 2;
        let default_y = rect_y - label_h - 2;

        let (label_x, label_y) = resolve_label_position(
            default_x,
            default_y,
            label_w,
            label_h,
            &placed_labels,
            img_w,
            img_h,
        );

        // Label background
        let bg_x = label_x.max(0);
        let bg_y = label_y.max(0);
        let bg_w = (label_w as u32).min(img_w.saturating_sub(bg_x as u32));
        let bg_h = (label_h as u32).min(img_h.saturating_sub(bg_y as u32));

        if bg_w > 0 && bg_h > 0 {
            draw_filled_rect_mut(
                &mut img,
                Rect::at(bg_x, bg_y).of_size(bg_w, bg_h),
                darken_color(color),
            );

            // Label text in white
            let text_x = bg_x + config.label_padding as i32;
            let text_y = bg_y + config.label_padding as i32;
            draw_text_mut(
                &mut img,
                Rgb([255u8, 255, 255]),
                text_x,
                text_y,
                scale,
                &font,
                label,
            );
        }

        placed_labels.push((label_x, label_y, label_x + label_w, label_y + label_h));

        // Category icon at centroid
        let (cx, cy) = world_to_pixel(
            feature.centroid[0] as f32,
            feature.centroid[1] as f32,
            bbox,
            img_w,
            img_h,
        );
        draw_category_icon(&mut img, cx as i32, cy as i32, &feature.category, color);
    }

    encode_png(&img)
}

/// Tint the bounding box area with the category color at ~20% opacity.
fn draw_bbox_tinted(img: &mut RgbImage, x: i32, y: i32, w: u32, h: u32, color: Rgb<u8>) {
    let alpha = 0.15_f32;
    for py in y..(y + h as i32) {
        for px in x..(x + w as i32) {
            if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height() {
                let pixel = img.get_pixel_mut(px as u32, py as u32);
                pixel[0] = blend(pixel[0], color[0], alpha);
                pixel[1] = blend(pixel[1], color[1], alpha);
                pixel[2] = blend(pixel[2], color[2], alpha);
            }
        }
    }
}

fn blend(base: u8, overlay: u8, alpha: f32) -> u8 {
    let result = f32::from(base) * (1.0 - alpha) + f32::from(overlay) * alpha;
    result.clamp(0.0, 255.0) as u8
}

/// Darker version of a color for label backgrounds.
fn darken_color(color: Rgb<u8>) -> Rgb<u8> {
    Rgb([
        (f32::from(color[0]) * 0.5) as u8,
        (f32::from(color[1]) * 0.5) as u8,
        (f32::from(color[2]) * 0.5) as u8,
    ])
}

/// Draw a small category icon at the centroid.
/// Circle (5px radius) for tree, square (8px) for structure, diamond for others.
fn draw_category_icon(img: &mut RgbImage, cx: i32, cy: i32, category: &str, color: Rgb<u8>) {
    let (w, h) = (img.width() as i32, img.height() as i32);

    match category {
        "tree" | "planting" => {
            // Filled circle, radius 5
            for dy in -5..=5_i32 {
                for dx in -5..=5_i32 {
                    if dx * dx + dy * dy <= 25 {
                        let px = cx + dx;
                        let py = cy + dy;
                        if px >= 0 && py >= 0 && px < w && py < h {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
        "structure" | "hardscape" => {
            // Filled square, 8×8
            for dy in -4..=4_i32 {
                for dx in -4..=4_i32 {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && py >= 0 && px < w && py < h {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
        _ => {
            // Diamond shape
            for dy in -4..=4_i32 {
                for dx in -4..=4_i32 {
                    if dx.abs() + dy.abs() <= 4 {
                        let px = cx + dx;
                        let py = cy + dy;
                        if px >= 0 && py >= 0 && px < w && py < h {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }
}

/// Resolve label position with collision avoidance.
/// Shifts label down if it overlaps previously placed labels (up to 5 attempts).
fn resolve_label_position(
    default_x: i32,
    default_y: i32,
    label_w: i32,
    label_h: i32,
    placed: &[(i32, i32, i32, i32)],
    img_w: u32,
    img_h: u32,
) -> (i32, i32) {
    let x = default_x.clamp(0, (img_w as i32 - label_w).max(0));
    let mut y = default_y;

    for _ in 0..5 {
        let overlaps = placed.iter().any(|&(px1, py1, px2, py2)| {
            x < px2 && (x + label_w) > px1 && y < py2 && (y + label_h) > py1
        });
        if !overlaps {
            break;
        }
        y += label_h + 4;
    }

    // Clamp to image bounds
    y = y.clamp(0, (img_h as i32 - label_h).max(0));
    (x, y)
}

fn encode_png(img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Vec<u8>, ScanError> {
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(Cursor::new(&mut buf));
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgb8,
    )
    .map_err(|e| ScanError::ExportError(format!("PNG encoding failed: {e}")))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    fn make_test_png(width: u32, height: u32) -> Vec<u8> {
        let img = ImageBuffer::from_pixel(width, height, Rgb([200u8, 200, 200]));
        encode_png(&img).unwrap()
    }

    fn make_test_bbox() -> BoundingBox {
        BoundingBox {
            min: [0.0, 0.0, 0.0],
            max: [10.0, 10.0, 2.0],
        }
    }

    fn make_test_annotation(label: &str, category: &str) -> FeatureAnnotation {
        FeatureAnnotation {
            label: label.to_string(),
            category: category.to_string(),
            bbox_min: [2.0, 2.0],
            bbox_max: [4.0, 4.0],
            centroid: [3.0, 3.0],
        }
    }

    #[test]
    fn test_category_colors() {
        timed(|| {
            assert_eq!(category_color("tree"), Rgb([34, 197, 94]));
            assert_eq!(category_color("planting"), Rgb([34, 197, 94]));
            assert_eq!(category_color("structure"), Rgb([107, 114, 128]));
            assert_eq!(category_color("hardscape"), Rgb([217, 119, 6]));
            assert_eq!(category_color("utility"), Rgb([239, 68, 68]));
            assert_eq!(category_color("unknown"), Rgb([156, 163, 175]));
        });
    }

    #[test]
    fn test_feature_annotation_from_pair() {
        timed(|| {
            let candidate = FeatureCandidate {
                cluster_id: 0,
                centroid: [3.0, 4.0, 1.5],
                bbox_min: [2.0, 3.0, 0.5],
                bbox_max: [4.0, 5.0, 2.5],
                height_ft: 25.1,
                spread_ft: 6.5,
                point_count: 1247,
                dominant_color: "brown".to_string(),
                vertical_profile: "columnar".to_string(),
                density: 150.0,
            };

            struct MockClassified;
            impl ClassifiedFeatureRef for MockClassified {
                fn label(&self) -> &str {
                    "London Plane Tree"
                }
                fn category(&self) -> &str {
                    "tree"
                }
                fn confidence(&self) -> f64 {
                    0.92
                }
                fn cluster_id(&self) -> i64 {
                    0
                }
            }

            let annotation = feature_annotation(&candidate, &MockClassified);
            assert_eq!(annotation.label, "London Plane Tree (0.92)");
            assert_eq!(annotation.category, "tree");
            assert_eq!(annotation.bbox_min, [2.0, 3.0]);
            assert_eq!(annotation.bbox_max, [4.0, 5.0]);
            assert_eq!(annotation.centroid, [3.0, 4.0]);
        });
    }

    #[test]
    fn test_annotate_empty_features() {
        timed(|| {
            let base = make_test_png(200, 200);
            let bbox = make_test_bbox();
            let config = AnnotationConfig::default();

            let result = annotate_plan_view_png(&base, &[], &bbox, &config).unwrap();

            // Should produce valid PNG
            assert!(result.len() >= 8);
            assert_eq!(&result[1..4], b"PNG");

            // Decode to verify dimensions match
            let decoded = image::load_from_memory(&result).unwrap().to_rgb8();
            assert_eq!(decoded.width(), 200);
            assert_eq!(decoded.height(), 200);
        });
    }

    #[test]
    fn test_annotate_with_features() {
        timed(|| {
            let base = make_test_png(300, 300);
            let bbox = make_test_bbox();
            let config = AnnotationConfig::default();

            let features = vec![
                make_test_annotation("London Plane (0.92)", "tree"),
                make_test_annotation("Concrete Curb (0.65)", "hardscape"),
            ];

            let result = annotate_plan_view_png(&base, &features, &bbox, &config).unwrap();

            // Valid PNG
            assert!(result.len() >= 8);
            assert_eq!(&result[1..4], b"PNG");

            // Annotated should be different from base (annotations change pixels)
            assert_ne!(
                result.len(),
                base.len(),
                "annotated PNG should differ from base"
            );

            // Decode to verify dimensions preserved
            let decoded = image::load_from_memory(&result).unwrap().to_rgb8();
            assert_eq!(decoded.width(), 300);
            assert_eq!(decoded.height(), 300);
        });
    }

    #[test]
    fn test_annotation_config_defaults() {
        timed(|| {
            let config = AnnotationConfig::default();
            assert!((config.font_scale - 14.0).abs() < f32::EPSILON);
            assert_eq!(config.box_line_width, 2);
            assert_eq!(config.label_padding, 4);
        });
    }

    #[test]
    fn test_resolve_label_no_collision() {
        timed(|| {
            let (x, y) = resolve_label_position(10, 10, 50, 20, &[], 300, 300);
            assert_eq!((x, y), (10, 10));
        });
    }

    #[test]
    fn test_resolve_label_with_collision() {
        timed(|| {
            let placed = vec![(10, 10, 60, 30)]; // existing label at (10,10)-(60,30)
            let (_, y) = resolve_label_position(10, 10, 50, 20, &placed, 300, 300);
            // Should shift down: 10 + 20 + 4 = 34
            assert_eq!(y, 34);
        });
    }
}
