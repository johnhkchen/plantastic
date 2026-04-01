use std::fmt;

/// A capability area in the product. Each maps to a chunk of the
/// 4-hour → 30-minute time savings claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueArea {
    SiteAssessment,
    Design,
    Quoting,
    CrewHandoff,
    Infrastructure,
}

impl ValueArea {
    pub fn label(&self) -> &'static str {
        match self {
            ValueArea::SiteAssessment => "SITE ASSESSMENT",
            ValueArea::Design => "DESIGN",
            ValueArea::Quoting => "QUOTING",
            ValueArea::CrewHandoff => "CREW HANDOFF",
            ValueArea::Infrastructure => "INFRASTRUCTURE",
        }
    }

    /// The total time savings budget (minutes) for this area.
    pub fn budget_minutes(&self) -> f64 {
        match self {
            ValueArea::SiteAssessment => 90.0,
            ValueArea::Design => 60.0,
            ValueArea::Quoting => 60.0,
            ValueArea::CrewHandoff => 30.0,
            ValueArea::Infrastructure => 0.0, // infra doesn't save user time directly
        }
    }

    pub const ALL: &[ValueArea] = &[
        ValueArea::SiteAssessment,
        ValueArea::Design,
        ValueArea::Quoting,
        ValueArea::CrewHandoff,
        ValueArea::Infrastructure,
    ];
}

/// How deeply integrated a passing scenario is into the product.
///
/// A scenario can pass (the computation is correct) but still be unreachable
/// by a real user. The integration level captures this gap.
///
/// Time savings are weighted by integration level:
///   raw_minutes * (stars / 5) = effective_minutes
///
/// A 25-minute scenario at 1 star = 5 effective minutes.
/// The same scenario at 5 stars = 25 effective minutes.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum Integration {
    /// Pure computation works in isolation. No API, no UI, no persistence.
    /// "The engine runs but no user can reach it."
    OneStar,
    /// Reachable via API but no UI. Could test with curl.
    /// "A developer can use it, a landscaper can't."
    TwoStar,
    /// API + basic UI exists. Functional but rough.
    /// "A landscaper could use it with hand-holding."
    ThreeStar,
    /// Polished UI, persisted, handles errors. Missing some edge cases.
    /// "A landscaper could use it in a demo."
    FourStar,
    /// Production-ready. Branded, reliable, handles edge cases, tested on real data.
    /// "A landscaper uses it daily to win contracts."
    FiveStar,
}

impl Integration {
    pub fn stars(self) -> u8 {
        match self {
            Integration::OneStar => 1,
            Integration::TwoStar => 2,
            Integration::ThreeStar => 3,
            Integration::FourStar => 4,
            Integration::FiveStar => 5,
        }
    }

    /// Weight factor: stars / 5.
    pub fn weight(self) -> f64 {
        f64::from(self.stars()) / 5.0
    }

    pub fn label(self) -> &'static str {
        match self {
            Integration::OneStar => "★☆☆☆☆",
            Integration::TwoStar => "★★☆☆☆",
            Integration::ThreeStar => "★★★☆☆",
            Integration::FourStar => "★★★★☆",
            Integration::FiveStar => "★★★★★",
        }
    }
}

/// The outcome of running a scenario test.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Pass and Blocked used as scenarios get implemented
pub enum ScenarioOutcome {
    /// Scenario passes. Integration level indicates how reachable it is.
    Pass(Integration),
    /// Scenario test exists but fails.
    Fail(String),
    /// Scenario test body is a stub — capability not yet implemented.
    NotImplemented,
    /// Scenario cannot run because upstream dependencies aren't ready.
    Blocked(String),
}

impl ScenarioOutcome {
    pub fn symbol(&self) -> &'static str {
        match self {
            ScenarioOutcome::Pass(_) => "●",
            ScenarioOutcome::Fail(_) => "✗",
            ScenarioOutcome::NotImplemented => "○",
            ScenarioOutcome::Blocked(_) => "◌",
        }
    }

    pub fn status_label(&self) -> String {
        match self {
            ScenarioOutcome::Pass(level) => format!("PASS {}", level.label()),
            ScenarioOutcome::Fail(msg) => format!("FAIL: {msg}"),
            ScenarioOutcome::NotImplemented => "NOT IMPLEMENTED".to_string(),
            ScenarioOutcome::Blocked(reason) => format!("BLOCKED: {reason}"),
        }
    }

    pub fn counts_as_delivered(&self) -> bool {
        matches!(self, ScenarioOutcome::Pass(_))
    }

    /// Effective minutes = raw minutes * integration weight.
    pub fn effective_minutes(&self, raw_minutes: f64) -> f64 {
        match self {
            ScenarioOutcome::Pass(level) => raw_minutes * level.weight(),
            _ => 0.0,
        }
    }
}

/// A single scenario test: one row in the Value Map.
///
/// To register a new scenario, add it to the appropriate suite module
/// in `suites/` and return it from that module's `scenarios()` function.
pub struct Scenario {
    /// Unique ID matching the Value Map (e.g., "S.3.1").
    pub id: &'static str,
    /// Human-readable name (e.g., "Quantity computation from geometry").
    pub name: &'static str,
    /// Which capability area this validates.
    pub area: ValueArea,
    /// How many minutes of user time this capability saves.
    pub time_savings_minutes: f64,
    /// What manual process this replaces. Used in reporting.
    #[allow(dead_code)]
    pub replaces: &'static str,
    /// The test function. Returns the outcome.
    pub test_fn: fn() -> ScenarioOutcome,
}

impl fmt::Debug for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scenario")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("area", &self.area)
            .field("time_savings_minutes", &self.time_savings_minutes)
            .finish()
    }
}

/// Result of running a scenario: the definition paired with its outcome.
pub struct ScenarioResult {
    pub scenario: &'static Scenario,
    pub outcome: ScenarioOutcome,
}
