//! Polygon perimeter computation in linear feet.

use geo::line_measures::Euclidean;
use geo::{Length, MultiPolygon, Polygon};

/// Returns the perimeter of a polygon's exterior ring in linear feet.
pub fn perimeter_ft(polygon: &Polygon<f64>) -> f64 {
    polygon.exterior().length::<Euclidean>()
}

/// Returns the total perimeter of all exterior rings in a multi-polygon.
pub fn multi_perimeter_ft(mp: &MultiPolygon<f64>) -> f64 {
    mp.iter().map(perimeter_ft).sum()
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
        assert_relative_eq!(perimeter_ft(&sq), 40.0);
    }

    #[test]
    fn right_triangle_3_4_5() {
        let tri = polygon![
            (x: 0.0, y: 0.0),
            (x: 3.0, y: 0.0),
            (x: 0.0, y: 4.0),
        ];
        assert_relative_eq!(perimeter_ft(&tri), 12.0);
    }

    #[test]
    fn empty_polygon() {
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        assert_relative_eq!(perimeter_ft(&empty), 0.0);
    }

    #[test]
    fn multi_polygon_perimeter() {
        let a = polygon![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 0.0),
            (x: 5.0, y: 5.0),
            (x: 0.0, y: 5.0),
        ];
        let b = polygon![
            (x: 10.0, y: 0.0),
            (x: 13.0, y: 0.0),
            (x: 13.0, y: 4.0),
            (x: 10.0, y: 4.0),
        ];
        let mp = MultiPolygon::new(vec![a, b]);
        // 20 + 14 = 34
        assert_relative_eq!(multi_perimeter_ft(&mp), 34.0);
    }
}
