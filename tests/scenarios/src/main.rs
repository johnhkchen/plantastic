mod api_helpers;
mod progress;
mod registry;
mod report;
mod suites;

use registry::ScenarioResult;
use std::panic::{self, AssertUnwindSafe};

fn main() {
    let scenarios = suites::all_scenarios();

    // Validate no duplicate IDs
    let mut seen = std::collections::HashSet::new();
    for s in &scenarios {
        if !seen.insert(s.id) {
            eprintln!("FATAL: duplicate scenario ID: {}", s.id);
            std::process::exit(2);
        }
    }

    // Run each scenario, catching panics as failures
    let results: Vec<ScenarioResult> = scenarios
        .into_iter()
        .map(|scenario| {
            let outcome =
                panic::catch_unwind(AssertUnwindSafe(scenario.test_fn)).unwrap_or_else(|e| {
                    let msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "panic (no message)".to_string()
                    };
                    registry::ScenarioOutcome::Fail(msg)
                });
            ScenarioResult { scenario, outcome }
        })
        .collect();

    report::print_dashboard(&results);

    std::process::exit(report::exit_code(&results));
}
