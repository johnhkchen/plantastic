//! Polygon area computation in square feet.

use geo::{Area, MultiPolygon, Polygon};

/// Returns the area of a polygon in square feet.
///
/// Uses unsigned (absolute) area, so the result is always non-negative
/// regardless of vertex winding order.
pub fn area_sqft(polygon: &Polygon<f64>) -> f64 {
    polygon.unsigned_area()
}

/// Returns the total area of a multi-polygon in square feet.
pub fn multi_area_sqft(mp: &MultiPolygon<f64>) -> f64 {
    mp.unsigned_area()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use geo::{polygon, LineString, MultiPolygon};

    #[test]
    fn square_10x10() {
        let sq = polygon![
            (x: 0.0, y: 0.0),
            (x: 10.0, y: 0.0),
            (x: 10.0, y: 10.0),
            (x: 0.0, y: 10.0),
        ];
        assert_relative_eq!(area_sqft(&sq), 100.0);
    }

    #[test]
    fn right_triangle() {
        // Triangle with base=6, height=4 → area = 12
        let tri = polygon![
            (x: 0.0, y: 0.0),
            (x: 6.0, y: 0.0),
            (x: 0.0, y: 4.0),
        ];
        assert_relative_eq!(area_sqft(&tri), 12.0);
    }

    #[test]
    fn l_shape() {
        // L-shape: 10×10 square minus a 5×5 corner = 75 sqft
        let l = polygon![
            (x: 0.0, y: 0.0),
            (x: 10.0, y: 0.0),
            (x: 10.0, y: 5.0),
            (x: 5.0, y: 5.0),
            (x: 5.0, y: 10.0),
            (x: 0.0, y: 10.0),
        ];
        assert_relative_eq!(area_sqft(&l), 75.0);
    }

    #[test]
    fn empty_polygon() {
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        assert_relative_eq!(area_sqft(&empty), 0.0);
    }

    #[test]
    fn multi_polygon_area() {
        let a = polygon![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 0.0),
            (x: 5.0, y: 5.0),
            (x: 0.0, y: 5.0),
        ];
        let b = polygon![
            (x: 10.0, y: 0.0),
            (x: 20.0, y: 0.0),
            (x: 20.0, y: 10.0),
            (x: 10.0, y: 10.0),
        ];
        let mp = MultiPolygon::new(vec![a, b]);
        assert_relative_eq!(multi_area_sqft(&mp), 125.0); // 25 + 100
    }
}
