// Site data reconciliation.
// Merges scan features, satellite baseline, and plan-view analysis
// into a unified site model with discrepancy detection.

class SatelliteTree {
  height_ft float
  spread_ft float
  confidence float
}

class SatelliteBaseline {
  lot_area_sqft float
  trees SatelliteTree[]
  avg_sun_hours float
}

class ReconciledFeature {
  label string
  category string
  source string
  confidence float
  reasoning string
}

class Discrepancy {
  description string
  possible_explanation string
  design_implication string
}

class RecommendedZone {
  label string
  zone_type string
  rationale string
  approximate_area_sqft float
  sun_exposure_hours int?
  data_sources string
}

class ReconciledSite {
  confirmed_features ReconciledFeature[]
  scan_only_features ReconciledFeature[]
  satellite_only_features ReconciledFeature[]
  discrepancies Discrepancy[]
  recommended_zones RecommendedZone[]
}

function ReconcileSiteData(
  scan_features: ClassifiedFeature[],
  satellite_baseline: SatelliteBaseline,
  plan_view_analysis: SiteAnalysis,
  address: string,
) -> ReconciledSite {
  client ProposalFallback
  prompt #"
    You are a senior landscape architect with GIS expertise, reconciling
    multiple data sources for a site assessment.

    Address: {{ address }}

    === SATELLITE BASELINE (pre-visit remote sensing) ===
    Lot area: {{ satellite_baseline.lot_area_sqft }} sq ft
    Average sun exposure: {{ satellite_baseline.avg_sun_hours }} hours/day (growing season)
    Detected trees from canopy data:
    {{ satellite_baseline.trees }}

    === LIDAR SCAN FEATURES (current ground truth) ===
    {{ scan_features }}

    === PLAN-VIEW ANALYSIS (spatial reasoning from annotated plan view) ===
    Suggested zones: {{ plan_view_analysis.suggested_zones }}
    Site observations: {{ plan_view_analysis.site_observations }}

    Your task: reconcile these three data sources into a unified site model.

    FEATURE RECONCILIATION:
    - Match scan features to satellite trees by category and approximate size
    - confirmed_features: seen in BOTH scan and satellite (high confidence)
    - scan_only_features: detected by LiDAR but not in satellite data
      (new plantings, hardscape, utilities — satellite can't detect these)
    - satellite_only_features: in satellite data but NOT in scan
      (possible recent removal — this is high-value intelligence)
    - source field: "scan", "satellite", or "both"

    TEMPORAL REASONING — this is critical:
    - Satellite data is historical; the LiDAR scan is current ground truth
    - A tree in satellite but not scan suggests recent removal
    - A tree in scan but not satellite suggests recent planting or growth
    - Size differences between sources suggest growth or measurement variance

    DISCREPANCIES:
    - Flag conflicts between data sources with explanations
    - Include design implications: how should the landscaper respond?
    - Examples: height mismatches, missing features, boundary conflicts

    RECOMMENDED ZONES:
    - Build on the plan-view analysis suggestions but now informed by ALL sources
    - Reference sun exposure data: "this area receives {{ satellite_baseline.avg_sun_hours }}
      hours — suitable for [sun-loving/shade-tolerant] species"
    - Reference specific features by label
    - zone_type must be one of: patio, bed, edging, fill, lawn, seating, pathway, screening
    - sun_exposure_hours: estimated hours for this specific zone (null if unknown)
    - data_sources: which sources informed this recommendation (e.g., "scan, satellite, plan_view")

    {{ ctx.output_format }}
  "#
}

test powell_market_reconciliation {
  functions [ReconcileSiteData]
  args {
    scan_features [
      {
        cluster_id 0
        label "London Plane Tree"
        category "tree"
        species "Platanus × acerifolia"
        confidence 0.85
        reasoning "25.1 ft height with 2.8 ft spread, columnar profile."
        landscape_notes "Root zone requires setback for adjacent hardscape."
      },
      {
        cluster_id 1
        label "London Plane Tree"
        category "tree"
        species "Platanus × acerifolia"
        confidence 0.82
        reasoning "22.6 ft height with 2.3 ft spread, columnar profile."
        landscape_notes "Root zone requires setback for adjacent hardscape."
      },
      {
        cluster_id 2
        label "Fire Hydrant"
        category "utility"
        species null
        confidence 0.91
        reasoning "2.5 ft height, compact spread, gray color, columnar profile."
        landscape_notes "3 ft clearance required per municipal code."
      }
    ]
    satellite_baseline {
      lot_area_sqft 1000.0
      trees [
        { height_ft 30.0, spread_ft 15.0, confidence 0.78 },
        { height_ft 28.0, spread_ft 12.0, confidence 0.72 },
        { height_ft 20.0, spread_ft 10.0, confidence 0.55 }
      ]
      avg_sun_hours 8.5
    }
    plan_view_analysis {
      features [
        {
          cluster_id 0
          label "London Plane Tree"
          category "tree"
          species "Platanus × acerifolia"
          confidence 0.85
          reasoning "25.1 ft height with 2.8 ft spread, columnar profile."
          landscape_notes "Root zone requires setback for adjacent hardscape."
        }
      ]
      suggested_zones [
        {
          label "Entry Patio"
          zone_type "patio"
          rationale "Clear area between the two London Plane trunks."
          approximate_area_sqft 18.4
        },
        {
          label "East Shade Garden"
          zone_type "bed"
          rationale "East side of the larger London Plane receives afternoon shade."
          approximate_area_sqft 24.0
        }
      ]
      site_observations [
        {
          observation "The two London Planes create a natural shade corridor along the north edge."
        }
      ]
    }
    address "Powell & Market Streets, San Francisco, CA"
  }
}
