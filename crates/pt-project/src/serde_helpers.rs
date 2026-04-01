//! Serde helpers for geo types within JSON serialization.

/// Serialize/deserialize `geo::Polygon<f64>` as GeoJSON geometry within a JSON struct.
pub mod geojson_polygon {
    use geo::Polygon;
    use geojson::Geometry;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::convert::TryInto;

    /// Serialize a polygon as GeoJSON geometry.
    ///
    /// # Errors
    /// Returns the serializer's error type on failure.
    pub fn serialize<S>(polygon: &Polygon<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let geom = Geometry::from(polygon);
        geom.serialize(serializer)
    }

    /// Deserialize a polygon from GeoJSON geometry.
    ///
    /// # Errors
    /// Returns a deserialization error if the geometry is invalid or not a polygon.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Polygon<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let geom = Geometry::deserialize(deserializer)?;
        let geo_geom: geo::Geometry<f64> = geom
            .try_into()
            .map_err(|e| serde::de::Error::custom(format!("invalid geometry: {e}")))?;
        match geo_geom {
            geo::Geometry::Polygon(p) => Ok(p),
            other => Err(serde::de::Error::custom(format!(
                "expected Polygon, got {:?}",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::polygon;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestWrapper {
        #[serde(with = "geojson_polygon")]
        geom: geo::Polygon<f64>,
    }

    #[test]
    fn polygon_serde_round_trip() {
        let poly = polygon![
            (x: 0.0, y: 0.0),
            (x: 12.0, y: 0.0),
            (x: 12.0, y: 15.0),
            (x: 0.0, y: 15.0),
        ];
        let wrapper = TestWrapper { geom: poly };
        let json = serde_json::to_string(&wrapper).unwrap();
        let back: TestWrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(wrapper, back);
    }

    #[test]
    fn polygon_coordinates_preserved() {
        let poly = polygon![
            (x: 1.123456789, y: 2.987654321),
            (x: 3.0, y: 4.0),
            (x: 5.0, y: 6.0),
        ];
        let wrapper = TestWrapper { geom: poly.clone() };
        let json = serde_json::to_string(&wrapper).unwrap();
        let back: TestWrapper = serde_json::from_str(&json).unwrap();
        // Coordinates should survive the round-trip
        let orig_coords: Vec<_> = poly.exterior().points().collect::<Vec<_>>();
        let back_coords: Vec<_> = back.geom.exterior().points().collect::<Vec<_>>();
        assert_eq!(orig_coords.len(), back_coords.len());
        for (a, b) in orig_coords.iter().zip(back_coords.iter()) {
            assert_eq!(a.x(), b.x());
            assert_eq!(a.y(), b.y());
        }
    }
}
