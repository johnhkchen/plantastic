// Planter estimation for measured gaps between features.
// The LLM picks plants and explains design reasoning.
// Code computes quantities and costs from spacing/depth values.

class PlantSelection {
  common_name string
  botanical_name string
  spacing_inches float
  why_this_plant string
}

class PlanterStyle {
  style_name string
  description string
  plant_selections PlantSelection[]
  soil_depth_inches float
  design_rationale string
}

class PlanterEstimate {
  styles PlanterStyle[]
}

function EstimatePlanter(
  gap_width_ft: float,
  gap_length_ft: float,
  area_sqft: float,
  adjacent_features: string[],
  sun_hours: int?,
  climate_zone: string,
  address: string,
) -> PlanterEstimate {
  client ProposalFallback
  prompt #"
    You are an expert landscape designer selecting plants for a measured gap
    between existing features in an urban landscape.

    Site: {{ address }}
    Climate: {{ climate_zone }}

    Gap dimensions:
    - Width: {{ gap_width_ft }} ft
    - Length: {{ gap_length_ft }} ft
    - Area: {{ area_sqft }} sq ft

    Adjacent features: {{ adjacent_features }}
    {% if sun_hours %}Estimated sun hours: {{ sun_hours }} hours/day{% endif %}

    Design exactly 3 planter styles ranging from budget-friendly to premium.
    Each style should feel like a real landscape design option, not generic filler.

    For each style, provide:
    - style_name: Evocative name (e.g., "Mediterranean Groundcover", "Woodland Edge")
    - description: 1-2 sentences on the look and feel
    - plant_selections: 2-4 plants, each with:
      - common_name: Standard horticultural name
      - botanical_name: Full Latin binomial
      - spacing_inches: Center-to-center spacing for this plant in this context
      - why_this_plant: 1 sentence on why it works HERE (reference sun, traffic, adjacent features)
    - soil_depth_inches: Required soil depth for this planting scheme (typically 4-12 inches)
    - design_rationale: Why this combination works in this specific gap

    Rules:
    - All plants must be appropriate for the climate zone and site conditions
    - Consider foot traffic, shade from adjacent features, and urban context
    - Spacing values must be realistic for the species (affects quantity computation)
    - Style 1 should be lowest maintenance/cost, Style 3 should be premium/statement
    - Do NOT compute quantities or costs — just pick plants and specify spacing/depth

    {{ ctx.output_format }}
  "#
}

test powell_market_gap {
  functions [EstimatePlanter]
  args {
    gap_width_ft 8.0
    gap_length_ft 2.3
    area_sqft 18.4
    adjacent_features ["London Plane Tree", "London Plane Tree"]
    sun_hours 4
    climate_zone "USDA 10b / Sunset 17 — Mediterranean maritime"
    address "Powell & Market Streets, San Francisco, CA"
  }
}
