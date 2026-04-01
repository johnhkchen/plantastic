use crate::progress::{Milestone, MILESTONES};
use crate::registry::{ScenarioOutcome, ScenarioResult, ValueArea};
use std::collections::HashMap;

/// Render the value delivery dashboard to stdout.
pub fn print_dashboard(results: &[ScenarioResult]) {
    let total_budget: f64 = ValueArea::ALL.iter().map(ValueArea::budget_minutes).sum();

    let total_effective: f64 = results
        .iter()
        .map(|r| r.outcome.effective_minutes(r.scenario.time_savings_minutes))
        .sum::<f64>()
        .max(0.0);

    let total_raw: f64 = results
        .iter()
        .filter(|r| r.outcome.counts_as_delivered())
        .map(|r| r.scenario.time_savings_minutes)
        .sum::<f64>()
        .max(0.0);

    let pct = if total_budget > 0.0 {
        (total_effective / total_budget) * 100.0
    } else {
        0.0
    };

    let (pass, fail, not_impl, blocked) = count_by_status(results);
    let (ms_done, ms_total) = milestone_counts();

    println!();
    println!("══════════════════════════════════════════════════════════════════");
    println!("  PLANTASTIC — VALUE DELIVERY DASHBOARD");
    println!("══════════════════════════════════════════════════════════════════");
    println!();
    println!(
        "  Effective savings: {:.1} min / {:.1} min ({:.1}%)",
        total_effective, total_budget, pct
    );
    if total_raw > total_effective {
        println!(
            "  Raw passing:       {:.1} min (before integration + polish weighting)",
            total_raw
        );
    }
    let polish_debt: f64 = results
        .iter()
        .filter_map(|r| match &r.outcome {
            ScenarioOutcome::Pass(_int_level, pol_level) => {
                let raw = r.scenario.time_savings_minutes;
                let gap = (5 - pol_level.stars()) as f64;
                Some(raw * gap / 10.0)
            }
            _ => None,
        })
        .sum();
    if polish_debt > 0.0 {
        println!(
            "  Polish debt:       {:.1} min recoverable by polish alone",
            polish_debt
        );
    }
    println!(
        "  Scenarios:         {pass} pass, {fail} fail, {not_impl} not implemented, {blocked} blocked",
    );
    println!("  Milestones:        {ms_done} / {ms_total} delivered");
    println!("  Formula:           effective = raw × (int★ + pol★) / 10");
    println!("  Ratings:           integration★ / polish★ (each 1–5, weighted equally)");
    println!();

    // Group results by area
    let mut by_area: HashMap<ValueArea, Vec<&ScenarioResult>> = HashMap::new();
    for r in results {
        by_area.entry(r.scenario.area).or_default().push(r);
    }

    for area in ValueArea::ALL {
        let area_results = by_area
            .get(area)
            .map_or(&[] as &[&ScenarioResult], Vec::as_slice);
        print_area_section(*area, area_results);
    }

    // Engineering progress section
    print_milestone_section();

    println!("══════════════════════════════════════════════════════════════════");
    println!();
}

fn print_area_section(area: ValueArea, results: &[&ScenarioResult]) {
    let budget = area.budget_minutes();
    let delivered: f64 = results
        .iter()
        .map(|r| r.outcome.effective_minutes(r.scenario.time_savings_minutes))
        .sum::<f64>()
        .max(0.0);

    let pct = if budget > 0.0 {
        (delivered / budget) * 100.0
    } else if results.iter().all(|r| r.outcome.counts_as_delivered()) && !results.is_empty() {
        100.0
    } else {
        0.0
    };

    let bar = progress_bar(pct, 40);

    if budget > 0.0 {
        println!(
            "  {}                    {:.1} / {:.1} min",
            area.label(),
            delivered,
            budget
        );
    } else {
        let passed = results
            .iter()
            .filter(|r| r.outcome.counts_as_delivered())
            .count();
        println!(
            "  {}                    {passed} / {} passing",
            area.label(),
            results.len()
        );
    }

    println!("  {bar} {pct:.0}%");

    for r in results {
        let sym = r.outcome.symbol();
        let status = r.outcome.status_label();
        let prereq = prerequisite_summary(r.scenario.id);
        println!(
            "    {sym} {:<6} {:<40} {status}",
            r.scenario.id, r.scenario.name
        );
        if !prereq.is_empty() {
            println!("             {prereq}");
        }
    }

    println!();
}

fn print_milestone_section() {
    let delivered: Vec<&Milestone> = MILESTONES
        .iter()
        .filter(|m| m.delivered_by.is_some())
        .collect();

    let pending: Vec<&Milestone> = MILESTONES
        .iter()
        .filter(|m| m.delivered_by.is_none())
        .collect();

    let (done, total) = milestone_counts();
    #[allow(clippy::cast_precision_loss)]
    let pct = if total > 0 {
        (done as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let bar = progress_bar(pct, 40);

    println!("  ENGINEERING PROGRESS              {done} / {total} milestones");
    println!("  {bar} {pct:.0}%");

    for m in &delivered {
        let ticket = m.delivered_by.unwrap_or("?");
        let unlocks_str = m.unlocks.join(", ");
        println!("    + {:<50} {ticket}", m.label);
        println!("      unlocks: {unlocks_str}");
        if !m.note.is_empty() {
            for line in wrap_text(m.note, 58) {
                println!("      {line}");
            }
        }
    }

    if !pending.is_empty() {
        let next_count = 3.min(pending.len());
        println!();
        println!("    Next up:");
        for m in pending.iter().take(next_count) {
            let unlocks_str = m.unlocks.join(", ");
            println!("    ○ {:<50} unlocks: {unlocks_str}", m.label);
        }
        let remaining = pending.len() - next_count;
        if remaining > 0 {
            println!("      ... and {remaining} more");
        }
    }

    println!();
}

/// Build a short prereq summary for a scenario (e.g., "prereqs: 2/4 met").
fn prerequisite_summary(scenario_id: &str) -> String {
    let total = MILESTONES
        .iter()
        .filter(|m| m.unlocks.contains(&scenario_id))
        .count();

    if total == 0 {
        return String::new();
    }

    let met = MILESTONES
        .iter()
        .filter(|m| m.unlocks.contains(&scenario_id) && m.delivered_by.is_some())
        .count();

    if met == total {
        format!("prereqs: {met}/{total} met -- ready to implement")
    } else {
        format!("prereqs: {met}/{total} met")
    }
}

fn milestone_counts() -> (usize, usize) {
    let total = MILESTONES.len();
    let done = MILESTONES
        .iter()
        .filter(|m| m.delivered_by.is_some())
        .count();
    (done, total)
}

fn progress_bar(pct: f64, width: usize) -> String {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn count_by_status(results: &[ScenarioResult]) -> (usize, usize, usize, usize) {
    let mut pass = 0;
    let mut fail = 0;
    let mut not_impl = 0;
    let mut blocked = 0;
    for r in results {
        match &r.outcome {
            ScenarioOutcome::Pass(..) => pass += 1,
            ScenarioOutcome::Fail(_) => fail += 1,
            ScenarioOutcome::NotImplemented => not_impl += 1,
            ScenarioOutcome::Blocked(_) => blocked += 1,
        }
    }
    (pass, fail, not_impl, blocked)
}

/// Simple word-wrap for milestone notes.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > width && !current.is_empty() {
            lines.push(current.clone());
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Returns exit code: 0 if no failures, 1 if any scenario fails.
/// `NotImplemented` and Blocked are not failures — they're honest status.
pub fn exit_code(results: &[ScenarioResult]) -> i32 {
    i32::from(
        results
            .iter()
            .any(|r| matches!(r.outcome, ScenarioOutcome::Fail(_))),
    )
}
