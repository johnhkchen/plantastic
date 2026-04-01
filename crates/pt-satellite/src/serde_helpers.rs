//! Serde helpers for geo types within JSON serialization.

/// Serialize/deserialize `geo::Polygon<f64>` as GeoJSON geometry.
pub mod geojson_polygon {
    use geo::Polygon;
    use geojson::Geometry;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::convert::TryInto;

    pub fn serialize<S>(polygon: &Polygon<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let geom = Geometry::from(polygon);
        geom.serialize(serializer)
    }

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
