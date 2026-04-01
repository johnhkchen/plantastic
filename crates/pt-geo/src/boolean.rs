//! Boolean operations on polygons: union and difference.

use geo::{BooleanOps, MultiPolygon, Polygon};

/// Returns the union of two polygons.
pub fn union(a: &Polygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64> {
    a.union(b)
}

/// Returns the difference `a - b` (the part of `a` not covered by `b`).
pub fn difference(a: &Polygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64> {
    a.difference(b)
}

/// Returns the union of a multi-polygon with a polygon.
pub fn multi_union(a: &MultiPolygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64> {
    a.union(b)
}

/// Returns the difference of a multi-polygon minus a polygon.
pub fn multi_difference(a: &MultiPolygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64> {
    a.difference(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use geo::{polygon, Area};

    fn square(x: f64, y: f64, size: f64) -> Polygon<f64> {
        polygon![
            (x: x, y: y),
            (x: x + size, y: y),
            (x: x + size, y: y + size),
            (x: x, y: y + size),
        ]
    }

    #[test]
    fn overlapping_union() {
        // Two 10×10 squares overlapping by 5×10
        let a = square(0.0, 0.0, 10.0);
        let b = square(5.0, 0.0, 10.0);
        let result = union(&a, &b);
        // Union = 150 sqft (200 - 50 overlap)
        assert_relative_eq!(result.unsigned_area(), 150.0, epsilon = 0.01);
    }

    #[test]
    fn non_overlapping_union() {
        let a = square(0.0, 0.0, 10.0);
        let b = square(20.0, 0.0, 10.0);
        let result = union(&a, &b);
        assert_relative_eq!(result.unsigned_area(), 200.0, epsilon = 0.01);
    }

    #[test]
    fn subtract_inner_from_outer() {
        // 10×10 outer, 4×4 inner centered at (3,3)
        let outer = square(0.0, 0.0, 10.0);
        let inner = square(3.0, 3.0, 4.0);
        let result = difference(&outer, &inner);
        // 100 - 16 = 84
        assert_relative_eq!(result.unsigned_area(), 84.0, epsilon = 0.01);
    }

    #[test]
    fn non_overlapping_difference() {
        let a = square(0.0, 0.0, 10.0);
        let b = square(20.0, 0.0, 10.0);
        let result = difference(&a, &b);
        // No overlap, so difference = a unchanged
        assert_relative_eq!(result.unsigned_area(), 100.0, epsilon = 0.01);
    }

    #[test]
    fn multi_union_chained() {
        let a = square(0.0, 0.0, 10.0);
        let b = square(5.0, 0.0, 10.0);
        let ab = union(&a, &b);
        let c = square(10.0, 0.0, 10.0);
        let result = multi_union(&ab, &c);
        // Three squares spanning 0..20 × 0..10 = 200 sqft
        assert_relative_eq!(result.unsigned_area(), 200.0, epsilon = 0.01);
    }
}
