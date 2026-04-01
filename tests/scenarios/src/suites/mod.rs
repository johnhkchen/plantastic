pub mod crew_handoff;
pub mod design;
pub mod infrastructure;
pub mod quoting;
pub mod site_assessment;

use crate::registry::Scenario;

/// Collect all registered scenarios from every suite.
/// To add a new suite: create a module, implement `fn scenarios()`,
/// and add it to this list.
pub fn all_scenarios() -> Vec<&'static Scenario> {
    let mut all = Vec::new();
    all.extend(site_assessment::scenarios());
    all.extend(design::scenarios());
    all.extend(quoting::scenarios());
    all.extend(crew_handoff::scenarios());
    all.extend(infrastructure::scenarios());
    all
}
