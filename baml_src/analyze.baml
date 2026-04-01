// Site analysis from annotated plan view.
// Multimodal: Claude analyzes the plan-view PNG alongside classified features
// to produce zone suggestions and site observations.

class SuggestedZone {
  label string
  zone_type string
  rationale string
  approximate_area_sqft float
}

class SiteObservation {
  observation string
}

class SiteAnalysis {
  features ClassifiedFeature[]
  suggested_zones SuggestedZone[]
  site_observations SiteObservation[]
}

function AnalyzePlanView(
  plan_view: image,
  lot_dimensions: string,
  address: string,
  classified_features: ClassifiedFeature[],
) -> SiteAnalysis {
  client ProposalFallback
  prompt #"
    You are a senior landscape architect analyzing an annotated site plan.

    Address: {{ address }}
    Lot dimensions: {{ lot_dimensions }}

    Here is the annotated plan view with labeled features and bounding boxes:
    {{ plan_view }}

    Classified features from LiDAR scan:
    {{ classified_features }}

    Your task:
    1. Echo back the classified features (preserve all fields exactly)
    2. Suggest 3–5 zone placements that maximize landscape value
    3. Provide 3–4 site observations demonstrating expert spatial reasoning

    ZONE SUGGESTIONS — think spatially:
    - Reference specific features by name: "the open area between the London Plane
      and the fire hydrant would work well as a drought-tolerant planting bed"
    - Consider shade patterns, foot traffic, sightlines, and adjacency
    - Each zone should have a specific purpose and location rationale
    - Estimate area from the plan view (use lot dimensions for scale)
    - zone_type must be one of: patio, bed, edging, fill, lawn, seating, pathway, screening

    SITE OBSERVATIONS — demonstrate landscape expertise:
    - Note shade corridors, drainage patterns, exposure, wind channels
    - Identify spatial relationships that create design opportunities
    - Reference the specific features visible in the plan view
    - Examples: "the two London Planes create a natural shade corridor along the
      north edge, ideal for shade-loving understory planting" or "the grade change
      near the curb suggests a rain garden opportunity"

    {{ ctx.output_format }}
  "#
}

test powell_market_analysis {
  functions [AnalyzePlanView]
  args {
    plan_view {
      url "file://assets/scans/samples/powell-market-downsampled-planview-annotated.png"
    }
    lot_dimensions "25ft x 40ft sidewalk frontage"
    address "Powell & Market Streets, San Francisco, CA"
    classified_features [
      {
        cluster_id 0
        label "London Plane Tree"
        category "tree"
        species "Platanus × acerifolia"
        confidence 0.85
        reasoning "25.1 ft height with 2.8 ft spread, columnar profile and brown color consistent with mature street tree trunk."
        landscape_notes "Root zone requires setback for adjacent hardscape. Provides shade coverage for south- and west-facing areas."
      },
      {
        cluster_id 1
        label "London Plane Tree"
        category "tree"
        species "Platanus × acerifolia"
        confidence 0.82
        reasoning "22.6 ft height with 2.3 ft spread, columnar profile and brown color consistent with mature street tree trunk."
        landscape_notes "Root zone requires setback for adjacent hardscape. Provides shade coverage for south- and west-facing areas."
      }
    ]
  }
}
