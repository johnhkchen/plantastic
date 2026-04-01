// Quote proposal narrative generation
// The LLM generates human-readable text ONLY — all numbers come from pt-quote.

class ZoneSummary {
  label string
  zone_type string        // "patio", "bed", "edging", "fill"
  area_sqft float
  materials string[]      // material names assigned in this tier
}

class TierInput {
  tier_level string       // "Good", "Better", "Best"
  total string            // formatted dollar amount from pt-quote
  zones ZoneSummary[]
}

class TierNarrative {
  tier_level string       // echoed back for alignment
  headline string         // e.g. "Entry-level curb appeal refresh"
  description string      // 2-3 sentences: what this tier includes and why
  differentiators string[] // 2-4 bullet points on what sets this tier apart
}

class ZoneCallout {
  zone_label string
  note string             // e.g. "The north bed uses shade-tolerant groundcover..."
}

class ProposalContent {
  intro_paragraph string       // greeting + project overview (2-3 sentences)
  tier_narratives TierNarrative[]
  zone_callouts ZoneCallout[]
  closing_paragraph string     // CTA + next steps (2-3 sentences)
}

function GenerateProposalNarrative(
  company_name: string,
  project_name: string,
  project_address: string,
  tiers: TierInput[],
) -> ProposalContent {
  client ProposalFallback
  prompt #"
    You are writing a professional landscaping proposal on behalf of {{ company_name }}.
    The project is "{{ project_name }}" at {{ project_address }}.

    Write warm, professional prose. The homeowner will read this.
    Do NOT invent or recalculate dollar amounts — use the exact totals provided.
    Focus on explaining what each tier includes, why the materials were chosen,
    and what differentiates each tier.

    Tiers:
    {{ tiers }}

    {{ ctx.output_format }}
  "#
}

// Mock test — uses canned data, no real LLM call needed
test sample_proposal {
  functions [GenerateProposalNarrative]
  args {
    company_name "Bay Area Landscapes"
    project_name "Johnson Backyard Refresh"
    project_address "1234 Oak Ave, San Francisco, CA"
    tiers [
      {
        tier_level "Good"
        total "$4,230.00"
        zones [
          {
            label "Main Patio"
            zone_type "patio"
            area_sqft 180.0
            materials ["Concrete Pavers"]
          },
          {
            label "Front Bed"
            zone_type "bed"
            area_sqft 160.0
            materials ["Standard Mulch"]
          }
        ]
      },
      {
        tier_level "Better"
        total "$6,890.00"
        zones [
          {
            label "Main Patio"
            zone_type "patio"
            area_sqft 180.0
            materials ["Travertine Pavers"]
          },
          {
            label "Front Bed"
            zone_type "bed"
            area_sqft 160.0
            materials ["Premium Mulch", "Steel Edging"]
          }
        ]
      },
      {
        tier_level "Best"
        total "$11,450.00"
        zones [
          {
            label "Main Patio"
            zone_type "patio"
            area_sqft 180.0
            materials ["Flagstone"]
          },
          {
            label "Front Bed"
            zone_type "bed"
            area_sqft 160.0
            materials ["Cedar Bark", "Corten Steel Edging", "Landscape Lighting"]
          }
        ]
      }
    ]
  }
}
