//! Polygon simplification using the Ramer-Douglas-Peucker algorithm.

use geo::{MultiPolygon, Polygon, Simplify as _};

/// Simplifies a polygon by removing vertices within `epsilon` distance
/// of the simplified line. Epsilon is in coordinate units (feet).
pub fn simplify(polygon: &Polygon<f64>, epsilon: f64) -> Polygon<f64> {
    polygon.simplify(&epsilon)
}

/// Simplifies a multi-polygon.
pub fn simplify_multi(mp: &MultiPolygon<f64>, epsilon: f64) -> MultiPolygon<f64> {
    mp.simplify(&epsilon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{polygon, Area};

    #[test]
    fn reduces_point_count() {
        // A jagged polygon with extra points along edges
        let complex = polygon![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 0.1),  // nearly collinear
            (x: 10.0, y: 0.0),
            (x: 10.0, y: 5.0),
            (x: 10.1, y: 10.0), // nearly collinear
            (x: 0.0, y: 10.0),
        ];
        let simplified = simplify(&complex, 0.5);
        let orig_points = complex.exterior().0.len();
        let simp_points = simplified.exterior().0.len();
        assert!(
            simp_points < orig_points,
            "simplified should have fewer points"
        );
    }

    #[test]
    fn epsilon_zero_preserves() {
        let sq = polygon![
            (x: 0.0, y: 0.0),
            (x: 10.0, y: 0.0),
            (x: 10.0, y: 10.0),
            (x: 0.0, y: 10.0),
        ];
        let simplified = simplify(&sq, 0.0);
        assert_eq!(simplified.exterior().0.len(), sq.exterior().0.len(),);
    }

    #[test]
    fn area_approximately_preserved() {
        let complex = polygon![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 0.1),
            (x: 10.0, y: 0.0),
            (x: 10.0, y: 5.0),
            (x: 10.1, y: 10.0),
            (x: 0.0, y: 10.0),
        ];
        let original_area = complex.unsigned_area();
        let simplified = simplify(&complex, 0.5);
        let simplified_area = simplified.unsigned_area();
        // Area should be within 5% of original
        let ratio = (simplified_area - original_area).abs() / original_area;
        assert!(ratio < 0.05, "area changed by {:.1}%", ratio * 100.0);
    }
}
