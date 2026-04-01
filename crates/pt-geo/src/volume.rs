//! Volume computation from area and depth.

/// Cubic feet from area (sq ft) and depth (ft).
pub fn volume_cuft(area_sqft: f64, depth_ft: f64) -> f64 {
    area_sqft * depth_ft
}

/// Cubic yards from area (sq ft) and depth (ft).
///
/// One cubic yard = 27 cubic feet.
pub fn volume_cuyd(area_sqft: f64, depth_ft: f64) -> f64 {
    volume_cuft(area_sqft, depth_ft) / 27.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn cuft_basic() {
        assert_relative_eq!(volume_cuft(100.0, 0.5), 50.0);
    }

    #[test]
    fn cuyd_basic() {
        // 100 sqft × 0.5 ft = 50 cuft / 27 ≈ 1.8519
        assert_relative_eq!(volume_cuyd(100.0, 0.5), 50.0 / 27.0);
    }

    #[test]
    fn zero_area() {
        assert_relative_eq!(volume_cuft(0.0, 3.0), 0.0);
        assert_relative_eq!(volume_cuyd(0.0, 3.0), 0.0);
    }

    #[test]
    fn zero_depth() {
        assert_relative_eq!(volume_cuft(200.0, 0.0), 0.0);
        assert_relative_eq!(volume_cuyd(200.0, 0.0), 0.0);
    }
}
