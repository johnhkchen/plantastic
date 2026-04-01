//! Spatial grid computation for sun exposure heatmaps.
//!
//! Computes average sun hours across a grid of lat/lng points within
//! a bounding box, sampling representative days from a date range.

use chrono::NaiveDate;

use crate::sun_hours::daily_sun_hours;
use crate::types::{Coordinates, ExposureGrid, GridConfig, LatLngBounds, METERS_PER_DEGREE_LAT};

/// Computes a sun exposure grid over the given bounds and date range.
///
/// Samples `config.sample_days` representative days evenly distributed
/// across the date range. For each grid cell, computes daily sun hours
/// for each sample day and averages the results.
#[allow(clippy::cast_possible_truncation)] // f64→f32 for sun hours is intentional (< 24.0)
pub fn radiance_grid(
    bounds: &LatLngBounds,
    date_range: (NaiveDate, NaiveDate),
    config: &GridConfig,
) -> ExposureGrid {
    let (width, height) = grid_dimensions(bounds, config.resolution_meters);
    let sample_dates = generate_sample_dates(date_range.0, date_range.1, config.sample_days);
    let total_cells = (width as usize) * (height as usize);
    let mut values = vec![0.0f32; total_cells];
    let n_samples = sample_dates.len();

    for row in 0..height {
        for col in 0..width {
            let (lat, lng) = cell_center(bounds, row, col, width, height);
            let coords = Coordinates {
                latitude: lat,
                longitude: lng,
            };

            let mut total_sun_hours = 0.0;
            for &sample_date in &sample_dates {
                let data = daily_sun_hours(&coords, sample_date);
                total_sun_hours += data.sun_hours;
            }

            let avg = if n_samples == 0 {
                0.0
            } else {
                total_sun_hours / n_samples as f64
            };

            let idx = (row as usize) * (width as usize) + (col as usize);
            values[idx] = avg as f32;
        }
    }

    ExposureGrid {
        bounds: *bounds,
        resolution_meters: config.resolution_meters,
        width,
        height,
        values,
        sample_days_used: u32::try_from(n_samples).unwrap_or(u32::MAX),
    }
}

/// Calculates grid dimensions in cells for given bounds and resolution.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // dimensions are small positive
pub fn grid_dimensions(bounds: &LatLngBounds, resolution_meters: f64) -> (u32, u32) {
    let center_lat = (bounds.south + bounds.north) / 2.0;

    let height_meters = (bounds.north - bounds.south) * METERS_PER_DEGREE_LAT;
    let lng_scale = METERS_PER_DEGREE_LAT * (center_lat.to_radians().cos());
    let width_meters = (bounds.east - bounds.west) * lng_scale;

    let height = 1.max((height_meters / resolution_meters).ceil() as u32);
    let width = 1.max((width_meters / resolution_meters).ceil() as u32);

    (width, height)
}

/// Returns the (lat, lng) of a grid cell's center.
/// Row 0 is southernmost, col 0 is westernmost.
fn cell_center(bounds: &LatLngBounds, row: u32, col: u32, width: u32, height: u32) -> (f64, f64) {
    let lat_frac = (f64::from(row) + 0.5) / f64::from(height);
    let lng_frac = (f64::from(col) + 0.5) / f64::from(width);

    let lat = bounds.south + lat_frac * (bounds.north - bounds.south);
    let lng = bounds.west + lng_frac * (bounds.east - bounds.west);
    (lat, lng)
}

/// Generates evenly-spaced sample dates across a range.
fn generate_sample_dates(start: NaiveDate, end: NaiveDate, count: u32) -> Vec<NaiveDate> {
    let total_days = (end - start).num_days();
    if total_days <= 0 || count == 0 {
        return vec![start];
    }

    let count_f = f64::from(count);
    (0..count)
        .map(|i| {
            let frac = (f64::from(i) + 0.5) / count_f;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let day_offset = (frac * total_days as f64) as u64;
            start + chrono::Days::new(day_offset)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    #[test]
    fn grid_dimensions_known_bounds() {
        timed(|| {
            // ~100m x ~100m area near the equator
            let bounds = LatLngBounds {
                south: 0.0,
                west: 0.0,
                // 100m in lat degrees = 100 / 111320 ≈ 0.000898
                north: 100.0 / METERS_PER_DEGREE_LAT,
                east: 100.0 / METERS_PER_DEGREE_LAT, // at equator, lat scale == lng scale
            };
            let (w, h) = grid_dimensions(&bounds, 10.0);
            assert_eq!(w, 10);
            assert_eq!(h, 10);
        });
    }

    #[test]
    fn cell_center_corners() {
        timed(|| {
            let bounds = LatLngBounds {
                south: 0.0,
                west: 0.0,
                north: 1.0,
                east: 1.0,
            };
            // 2x2 grid: cell (0,0) center should be at (0.25, 0.25)
            let (lat, lng) = cell_center(&bounds, 0, 0, 2, 2);
            assert!((lat - 0.25).abs() < 1e-10);
            assert!((lng - 0.25).abs() < 1e-10);

            // cell (1,1) center should be at (0.75, 0.75)
            let (lat, lng) = cell_center(&bounds, 1, 1, 2, 2);
            assert!((lat - 0.75).abs() < 1e-10);
            assert!((lng - 0.75).abs() < 1e-10);
        });
    }

    #[test]
    fn small_grid_sf_bay_area() {
        timed(|| {
            // Small 3x3 grid over a ~150m area in SF
            let center_lat: f64 = 37.7749;
            let center_lng: f64 = -122.4194;
            let offset_lat = 75.0 / METERS_PER_DEGREE_LAT;
            let lng_scale = METERS_PER_DEGREE_LAT * center_lat.to_radians().cos();
            let offset_lng = 75.0 / lng_scale;

            let bounds = LatLngBounds {
                south: center_lat - offset_lat,
                west: center_lng - offset_lng,
                north: center_lat + offset_lat,
                east: center_lng + offset_lng,
            };

            let config = GridConfig {
                resolution_meters: 50.0,
                sample_days: 4,
            };

            let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            let end = NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();

            let grid = radiance_grid(&bounds, (start, end), &config);

            // Grid should be approximately 3x3 (150m / 50m), but floating-point
            // roundtrip through degrees may produce 3 or 4 in either dimension.
            assert!(
                grid.width >= 3 && grid.width <= 4,
                "unexpected width: {}",
                grid.width
            );
            assert!(
                grid.height >= 3 && grid.height <= 4,
                "unexpected height: {}",
                grid.height
            );
            assert_eq!(grid.values.len(), (grid.width * grid.height) as usize);

            // All cells should have reasonable Bay Area growing-season sun hours
            for (i, &val) in grid.values.iter().enumerate() {
                assert!(val > 8.0, "cell {i} too low: {val}");
                assert!(val < 18.0, "cell {i} too high: {val}");
            }
        });
    }
}
