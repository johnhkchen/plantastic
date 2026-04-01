//! Sunset Western Garden zone lookup for Bay Area locations.
//!
//! Sunset zones are the horticultural authority for western US gardens. They
//! incorporate winter lows, summer highs, humidity, wind, and ocean influence —
//! much more nuanced than USDA zones for plant selection.
//!
//! V1 covers Bay Area only (zones 14-17) via bounding-box approximation.
//! Full polygon coverage would require shapefile data (V2).

use crate::types::SunsetZoneEntry;
use crate::{Coordinates, SunsetZone};

/// Bay Area Sunset zone lookup table.
///
/// Zones are ordered from most coastal (17) to most inland (14), tested
/// in reverse order so more specific coastal zones match first.
///
/// Zone descriptions from Sunset Western Garden Book:
/// - 17: Marine influence, mild winters, cool summers (coastal SF, Pacifica)
/// - 16: Thermal belts and slopes, some marine influence (Oakland hills, Berkeley)
/// - 15: Cold-winter areas with some marine influence (San Jose, inland valleys)
/// - 14: Inland valleys with hot summers, cold winters (Livermore, Concord)
const BAY_AREA_ZONES: &[SunsetZoneEntry] = &[
    // Zone 17 — Coastal fog belt: ocean-adjacent, very mild
    // SF coast, Pacifica, Half Moon Bay, Daly City, outer Richmond/Sunset
    SunsetZoneEntry {
        lat_min: 37.55,
        lat_max: 37.82,
        lng_min: -122.52,
        lng_max: -122.38,
        zone: 17,
        description: "Coastal fog belt — mild winters, cool summers, strong marine influence",
    },
    // Zone 17 — Pacifica / Half Moon Bay coastline
    SunsetZoneEntry {
        lat_min: 37.35,
        lat_max: 37.55,
        lng_min: -122.52,
        lng_max: -122.38,
        zone: 17,
        description: "Coastal fog belt — mild winters, cool summers, strong marine influence",
    },
    // Zone 16 — Marine-influenced but warmer: Oakland, Berkeley, San Mateo, inner SF
    SunsetZoneEntry {
        lat_min: 37.75,
        lat_max: 37.90,
        lng_min: -122.38,
        lng_max: -122.20,
        zone: 16,
        description: "Thermal belts — winter lows rarely below 25F, marine moderation",
    },
    // Zone 16 — South SF, San Bruno, San Mateo
    SunsetZoneEntry {
        lat_min: 37.55,
        lat_max: 37.75,
        lng_min: -122.45,
        lng_max: -122.20,
        zone: 16,
        description: "Thermal belts — winter lows rarely below 25F, marine moderation",
    },
    // Zone 16 — Berkeley hills, Orinda
    SunsetZoneEntry {
        lat_min: 37.82,
        lat_max: 37.95,
        lng_min: -122.30,
        lng_max: -122.10,
        zone: 16,
        description: "Thermal belts — winter lows rarely below 25F, marine moderation",
    },
    // Zone 15 — Inland with some marine influence: San Jose, Fremont, Hayward
    SunsetZoneEntry {
        lat_min: 37.20,
        lat_max: 37.55,
        lng_min: -122.10,
        lng_max: -121.70,
        zone: 15,
        description: "Cold-winter areas — some marine influence, occasional hard freezes",
    },
    // Zone 15 — Sunnyvale, Santa Clara, Milpitas
    SunsetZoneEntry {
        lat_min: 37.30,
        lat_max: 37.50,
        lng_min: -122.10,
        lng_max: -121.85,
        zone: 15,
        description: "Cold-winter areas — some marine influence, occasional hard freezes",
    },
    // Zone 15 — Hayward, Union City, Fremont
    SunsetZoneEntry {
        lat_min: 37.50,
        lat_max: 37.70,
        lng_min: -122.15,
        lng_max: -121.90,
        zone: 15,
        description: "Cold-winter areas — some marine influence, occasional hard freezes",
    },
    // Zone 14 — Inland valleys: Livermore, Concord, Walnut Creek, Antioch
    SunsetZoneEntry {
        lat_min: 37.65,
        lat_max: 38.05,
        lng_min: -122.10,
        lng_max: -121.60,
        zone: 14,
        description: "Inland valleys — hot summers, cold winters, minimal marine influence",
    },
    // Zone 14 — Tri-Valley: Dublin, Pleasanton, Livermore
    SunsetZoneEntry {
        lat_min: 37.55,
        lat_max: 37.75,
        lng_min: -121.95,
        lng_max: -121.60,
        zone: 14,
        description: "Inland valleys — hot summers, cold winters, minimal marine influence",
    },
];

/// Look up the Sunset Western Garden zone for a location.
///
/// Returns `Some(SunsetZone)` for Bay Area locations, `None` for locations
/// outside the coverage area. V1 covers zones 14-17 in the SF Bay Area only.
pub fn sunset_zone(coords: &Coordinates) -> Option<SunsetZone> {
    for entry in BAY_AREA_ZONES {
        if coords.latitude >= entry.lat_min
            && coords.latitude <= entry.lat_max
            && coords.longitude >= entry.lng_min
            && coords.longitude <= entry.lng_max
        {
            return Some(SunsetZone {
                zone: entry.zone,
                description: entry.description,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    // SF downtown (37.7749, -122.4194) — should be zone 17 (coastal fog belt)
    #[test]
    fn sf_downtown_is_zone_17() {
        timed(|| {
            let sf = Coordinates {
                latitude: 37.7749,
                longitude: -122.4194,
            };
            let zone = sunset_zone(&sf).expect("SF should have a Sunset zone");
            assert_eq!(zone.zone, 17);
        });
    }

    // Oakland (37.8044, -122.2712) — should be zone 16
    #[test]
    fn oakland_is_zone_16() {
        timed(|| {
            let oak = Coordinates {
                latitude: 37.8044,
                longitude: -122.2712,
            };
            let zone = sunset_zone(&oak).expect("Oakland should have a Sunset zone");
            assert_eq!(zone.zone, 16);
        });
    }

    // San Jose (37.3382, -121.8863) — should be zone 15
    #[test]
    fn san_jose_is_zone_15() {
        timed(|| {
            let sj = Coordinates {
                latitude: 37.3382,
                longitude: -121.8863,
            };
            let zone = sunset_zone(&sj).expect("San Jose should have a Sunset zone");
            assert_eq!(zone.zone, 15);
        });
    }

    // Livermore (37.6819, -121.7681) — should be zone 14
    #[test]
    fn livermore_is_zone_14() {
        timed(|| {
            let liv = Coordinates {
                latitude: 37.6819,
                longitude: -121.7681,
            };
            let zone = sunset_zone(&liv).expect("Livermore should have a Sunset zone");
            assert_eq!(zone.zone, 14);
        });
    }

    // Outside Bay Area — should be None
    #[test]
    fn portland_returns_none() {
        timed(|| {
            let pdx = Coordinates {
                latitude: 45.5152,
                longitude: -122.6784,
            };
            assert!(sunset_zone(&pdx).is_none());
        });
    }

    // Far outside coverage
    #[test]
    fn new_york_returns_none() {
        timed(|| {
            let nyc = Coordinates {
                latitude: 40.7128,
                longitude: -74.0060,
            };
            assert!(sunset_zone(&nyc).is_none());
        });
    }
}
