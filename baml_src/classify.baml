// LiDAR feature classification
// Takes geometric feature summaries and returns domain-expert classifications.

class FeatureCandidateInput {
  cluster_id int
  centroid_x float
  centroid_y float
  centroid_z float
  height_ft float
  spread_ft float
  point_count int
  dominant_color string
  vertical_profile string
  density float
}

class ClassifiedFeature {
  cluster_id int
  label string
  category string
  species string?
  confidence float
  reasoning string
  landscape_notes string
}

function ClassifyFeatures(
  candidates: FeatureCandidateInput[],
  address: string,
  climate_zone: string,
) -> ClassifiedFeature[] {
  client ProposalFallback
  prompt #"
    You are a landscape design expert and certified arborist analyzing LiDAR scan data.
    Your task: classify each detected feature from its geometric profile.

    Site: {{ address }}
    Climate: {{ climate_zone }}

    Each candidate below is a cluster of 3D points from an iPhone LiDAR scan.
    You have geometric summaries only — no images, no raw points.

    Candidates:
    {{ candidates }}

    For each candidate, provide:
    - label: Human-readable name (e.g., "London Plane Tree", "Brick Pathway", "Street Light Pole")
    - category: One of "tree", "structure", "hardscape", "planting", "utility"
    - species: Botanical name for vegetation (e.g., "Platanus × acerifolia"), null for non-vegetation
    - confidence: 0.0–1.0. Calibrate honestly:
      - >0.8: Distinctive profile matches a known species/type for this region
      - 0.5–0.8: Plausible match but ambiguous (could be multiple species/types)
      - <0.5: Geometric cues are insufficient for confident classification
    - reasoning: Explain WHY your classification matches the geometric profile.
      Reference specific measurements (height, spread, density, color, profile shape).
      This is the "wow" text — make it demonstrate expert knowledge.
    - landscape_notes: Design implications for a landscaper working this site.
      Examples: shade patterns, root zone setbacks, maintenance considerations, hardscape compatibility.

    Regional context matters:
    - Use address and climate zone to narrow species candidates
    - San Francisco street trees are commonly London Plane, Brisbane Box, Ficus microcarpa
    - Consider urban infrastructure: poles, bollards, planters, curb edges, fire hydrants
    - Trunk-only scans (no canopy) are common — reason from cylinder diameter, bark texture cues in color, and height

    {{ ctx.output_format }}
  "#
}

test powell_market_features {
  functions [ClassifyFeatures]
  args {
    candidates [
      {
        cluster_id 0
        centroid_x -3.21
        centroid_y 0.42
        centroid_z 3.81
        height_ft 25.1
        spread_ft 2.8
        point_count 1247
        dominant_color "brown"
        vertical_profile "columnar"
        density 892.3
      },
      {
        cluster_id 1
        centroid_x 1.85
        centroid_y -0.31
        centroid_z 3.44
        height_ft 22.6
        spread_ft 2.3
        point_count 983
        dominant_color "brown"
        vertical_profile "columnar"
        density 756.1
      }
    ]
    address "Powell & Market Streets, San Francisco, CA"
    climate_zone "USDA 10b / Sunset 17 — Mediterranean maritime"
  }
}
