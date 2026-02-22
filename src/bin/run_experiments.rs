// Genesis Protocol — Flagship Experiment Runner
//
// Runs all three flagship experiments, exports results to experiments/ directory.
// Usage: cargo run --bin run_experiments

use genesis_experiment::{
    ExperimentRunner, ExperimentReport, FlagshipExperiments,
};
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║       GENESIS PROTOCOL — FLAGSHIP EXPERIMENTS          ║");
    println!("║           Phase 5: The Experimental Method             ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let output_dir = "experiments";
    std::fs::create_dir_all(output_dir).expect("Failed to create experiments/ directory");

    let experiments: Vec<(&str, genesis_experiment::ExperimentConfig)> = vec![
        ("entropy_sweep", FlagshipExperiments::entropy_sweep()),
        ("catastrophe_resilience", FlagshipExperiments::catastrophe_resilience()),
        ("inequality_threshold", FlagshipExperiments::inequality_threshold()),
        ("treasury_stability", FlagshipExperiments::treasury_stability()),
    ];

    let mut all_findings: Vec<(String, Vec<String>)> = Vec::new();
    let global_start = Instant::now();

    for (slug, config) in &experiments {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Experiment: {}", config.name);
        println!("  Hypothesis: {}", config.hypothesis);
        println!("  Worlds: {} | Epochs/world: {} | Total epochs: {}",
            config.total_worlds(),
            config.epochs_per_run,
            config.total_worlds() as u64 * config.epochs_per_run,
        );
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let start = Instant::now();
        let result = ExperimentRunner::run(config);
        let elapsed = start.elapsed();

        println!("  Completed in {:.2}s", elapsed.as_secs_f64());
        println!("  Total epochs run: {}", result.total_epochs_run);
        println!("  Result hash: {}", &result.result_hash);
        println!();

        // Derive findings from the data
        let findings = derive_findings(slug, &result);
        for (i, f) in findings.iter().enumerate() {
            println!("  Finding {}: {}", i + 1, f);
        }
        println!();

        // Generate report
        let report = ExperimentReport::generate(&result, findings.clone());

        // Save to experiments/<slug>/
        let dir = format!("{}/{}", output_dir, slug);
        report.save_to_dir_with_slug(&dir, Some(slug)).expect("Failed to save report");

        println!("  Saved: {}/", dir);
        println!("    - *_report.txt");
        println!("    - *_data.csv");
        println!("    - *_manifest.json");
        println!();

        all_findings.push((config.name.clone(), findings));
    }

    let total_elapsed = global_start.elapsed();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                   ALL EXPERIMENTS COMPLETE              ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("  Total time: {:.2}s", total_elapsed.as_secs_f64());
    println!("  Total worlds spawned: {}", experiments.iter().map(|(_, c)| c.total_worlds()).sum::<usize>());
    println!("  Output directory: {}/", output_dir);
    println!();

    // Print consolidated findings
    println!("── Consolidated Findings ──");
    println!();
    for (name, findings) in &all_findings {
        println!("  {}", name);
        for f in findings {
            println!("    • {}", f);
        }
        println!();
    }
}

/// Derive findings from experiment results by analyzing the data.
fn derive_findings(slug: &str, result: &genesis_experiment::ExperimentResult) -> Vec<String> {
    let mut findings = Vec::new();

    match slug {
        "entropy_sweep" => {
            // Find collapse transition point
            let mut first_collapse_step = None;
            let mut first_total_collapse_step = None;
            for step in &result.steps {
                if step.collapse_rate > 0.0 && first_collapse_step.is_none() {
                    first_collapse_step = Some(step.clone());
                }
                if step.collapse_rate >= 1.0 && first_total_collapse_step.is_none() {
                    first_total_collapse_step = Some(step.clone());
                }
            }

            // Collapse analysis
            if let Some(ref step) = first_collapse_step {
                findings.push(format!(
                    "First collapse observed at entropy_coeff = {:.6} (collapse rate: {:.0}%)",
                    step.parameter_value, step.collapse_rate * 100.0
                ));
            } else {
                findings.push("No collapses observed across entire entropy range".into());
            }

            if let Some(ref step) = first_total_collapse_step {
                findings.push(format!(
                    "Total collapse (100%) occurs at entropy_coeff = {:.6}",
                    step.parameter_value
                ));
            }

            // Gini trajectory
            let gini_first = result.steps.first()
                .and_then(|s| s.metric_summaries.get("gini_coefficient"))
                .map(|s| s.mean);
            let gini_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("gini_coefficient"))
                .map(|s| s.mean);
            if let (Some(first), Some(last)) = (gini_first, gini_last) {
                let change_pct = ((last - first) / first.max(0.001)) * 100.0;
                findings.push(format!(
                    "Gini coefficient moves from {:.4} to {:.4} ({:+.1}%) across entropy range",
                    first, last, change_pct
                ));
            }

            // Population impact
            let pop_first = result.steps.first()
                .and_then(|s| s.metric_summaries.get("mean_population"))
                .map(|s| s.mean);
            let pop_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("mean_population"))
                .map(|s| s.mean);
            if let (Some(first), Some(last)) = (pop_first, pop_last) {
                findings.push(format!(
                    "Mean population declines from {:.1} to {:.1} as entropy increases",
                    first, last
                ));
            }

            // Mean fitness impact
            let fit_first = result.steps.first()
                .and_then(|s| s.metric_summaries.get("mean_fitness"))
                .map(|s| s.mean);
            let fit_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("mean_fitness"))
                .map(|s| s.mean);
            if let (Some(first), Some(last)) = (fit_first, fit_last) {
                findings.push(format!(
                    "Mean fitness shifts from {:.4} to {:.4} under increasing entropy pressure",
                    first, last
                ));
            }
        }

        "catastrophe_resilience" => {
            // Find survival sweet spot
            let mut best_survival_step = result.steps.first().cloned();
            let mut best_survival = 0.0f64;
            for step in &result.steps {
                if step.mean_survival_epochs > best_survival {
                    best_survival = step.mean_survival_epochs;
                    best_survival_step = Some(step.clone());
                }
            }

            if let Some(ref step) = best_survival_step {
                findings.push(format!(
                    "Peak mean survival ({:.1} epochs) at catastrophe_base_prob = {:.4}",
                    step.mean_survival_epochs, step.parameter_value
                ));
            }

            // Collapse curve
            let peaceful = result.steps.first().map(|s| s.collapse_rate).unwrap_or(0.0);
            let max_cat = result.steps.last().map(|s| s.collapse_rate).unwrap_or(0.0);
            findings.push(format!(
                "Collapse rate: {:.0}% at peaceful (0.0) vs {:.0}% at extreme (0.03)",
                peaceful * 100.0, max_cat * 100.0
            ));

            // Resilience analysis: compare mid-range vs extremes
            let mid_idx = result.steps.len() / 2;
            if let Some(mid_step) = result.steps.get(mid_idx) {
                let mid_pop = mid_step.metric_summaries.get("mean_population")
                    .map(|s| s.mean).unwrap_or(0.0);
                let first_pop = result.steps.first()
                    .and_then(|s| s.metric_summaries.get("mean_population"))
                    .map(|s| s.mean).unwrap_or(0.0);
                findings.push(format!(
                    "Mid-range catastrophe ({:.4}) produces mean pop {:.1} vs {:.1} at peaceful",
                    mid_step.parameter_value, mid_pop, first_pop
                ));
            }

            // Catastrophe death toll
            let cat_deaths_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("total_catastrophe_deaths"))
                .map(|s| s.mean);
            if let Some(deaths) = cat_deaths_last {
                findings.push(format!(
                    "Mean catastrophe deaths at extreme rate: {:.1}",
                    deaths
                ));
            }
        }

        "inequality_threshold" => {
            // Find optimal threshold by population stability
            let mut best_pop_step = result.steps.first().cloned();
            let mut best_pop = 0.0f64;
            for step in &result.steps {
                let pop = step.metric_summaries.get("mean_population")
                    .map(|s| s.mean).unwrap_or(0.0);
                if pop > best_pop {
                    best_pop = pop;
                    best_pop_step = Some(step.clone());
                }
            }

            if let Some(ref step) = best_pop_step {
                findings.push(format!(
                    "Optimal population stability at gini_threshold = {:.2} (mean pop: {:.1})",
                    step.parameter_value, best_pop
                ));
            }

            // Gini coefficient vs threshold
            let gini_low = result.steps.first()
                .and_then(|s| s.metric_summaries.get("gini_coefficient"))
                .map(|s| s.mean);
            let gini_high = result.steps.last()
                .and_then(|s| s.metric_summaries.get("gini_coefficient"))
                .map(|s| s.mean);
            if let (Some(low), Some(high)) = (gini_low, gini_high) {
                findings.push(format!(
                    "Gini: {:.4} at aggressive threshold (0.20) vs {:.4} at laissez-faire (0.90)",
                    low, high
                ));
            }

            // Fitness vs threshold
            let fit_low = result.steps.first()
                .and_then(|s| s.metric_summaries.get("mean_fitness"))
                .map(|s| s.mean);
            let fit_high = result.steps.last()
                .and_then(|s| s.metric_summaries.get("mean_fitness"))
                .map(|s| s.mean);
            if let (Some(low), Some(high)) = (fit_low, fit_high) {
                findings.push(format!(
                    "Mean fitness: {:.4} at aggressive redistribution vs {:.4} at laissez-faire",
                    low, high
                ));
            }

            // Collapse comparison
            let collapse_low = result.steps.first().map(|s| s.collapse_rate).unwrap_or(0.0);
            let collapse_high = result.steps.last().map(|s| s.collapse_rate).unwrap_or(0.0);
            findings.push(format!(
                "Collapse rate: {:.0}% (aggressive redistribution) vs {:.0}% (laissez-faire)",
                collapse_low * 100.0, collapse_high * 100.0
            ));

            // Treasury health
            let treasury_first = result.steps.first()
                .and_then(|s| s.metric_summaries.get("treasury_ratio"))
                .map(|s| s.mean);
            let treasury_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("treasury_ratio"))
                .map(|s| s.mean);
            if let (Some(first), Some(last)) = (treasury_first, treasury_last) {
                findings.push(format!(
                    "Treasury ratio: {:.4} at threshold 0.20 vs {:.4} at threshold 0.90",
                    first, last
                ));
            }
        }

        "treasury_stability" => {
            // Treasury ratio trajectory across overflow thresholds
            let treasury_first = result.steps.first()
                .and_then(|s| s.metric_summaries.get("treasury_ratio"))
                .map(|s| s.mean);
            let treasury_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("treasury_ratio"))
                .map(|s| s.mean);
            if let (Some(first), Some(last)) = (treasury_first, treasury_last) {
                let change_pct = ((last - first) / first.max(0.001)) * 100.0;
                findings.push(format!(
                    "Treasury ratio: {:.4} at aggressive deployment (0.10) vs {:.4} at hoarding (0.90) ({:+.1}%)",
                    first, last, change_pct
                ));
            }

            // Find optimal threshold by population stability
            let mut best_pop_step = result.steps.first().cloned();
            let mut best_pop = 0.0f64;
            for step in &result.steps {
                let pop = step.metric_summaries.get("mean_population")
                    .map(|s| s.mean).unwrap_or(0.0);
                if pop > best_pop {
                    best_pop = pop;
                    best_pop_step = Some(step.clone());
                }
            }
            if let Some(ref step) = best_pop_step {
                findings.push(format!(
                    "Peak population stability at overflow_threshold = {:.2} (mean pop: {:.1})",
                    step.parameter_value, best_pop
                ));
            }

            // Gini trajectory
            let gini_first = result.steps.first()
                .and_then(|s| s.metric_summaries.get("gini_coefficient"))
                .map(|s| s.mean);
            let gini_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("gini_coefficient"))
                .map(|s| s.mean);
            if let (Some(first), Some(last)) = (gini_first, gini_last) {
                let change_pct = ((last - first) / first.max(0.001)) * 100.0;
                findings.push(format!(
                    "Gini coefficient: {:.4} at deployment (0.10) vs {:.4} at hoarding (0.90) ({:+.1}%)",
                    first, last, change_pct
                ));
            }

            // Collapse analysis
            let any_collapse = result.steps.iter().any(|s| s.collapse_rate > 0.0);
            if any_collapse {
                let max_collapse_step = result.steps.iter()
                    .max_by(|a, b| a.collapse_rate.partial_cmp(&b.collapse_rate).unwrap())
                    .unwrap();
                findings.push(format!(
                    "Highest collapse rate: {:.0}% at overflow_threshold = {:.2}",
                    max_collapse_step.collapse_rate * 100.0, max_collapse_step.parameter_value
                ));
            } else {
                findings.push("No collapses observed across entire threshold range".into());
            }

            // Fitness comparison
            let fit_first = result.steps.first()
                .and_then(|s| s.metric_summaries.get("mean_fitness"))
                .map(|s| s.mean);
            let fit_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("mean_fitness"))
                .map(|s| s.mean);
            if let (Some(first), Some(last)) = (fit_first, fit_last) {
                findings.push(format!(
                    "Mean fitness: {:.4} at aggressive deployment vs {:.4} at hoarding",
                    first, last
                ));
            }

            // Population volatility comparison
            let vol_first = result.steps.first()
                .and_then(|s| s.metric_summaries.get("population_volatility"))
                .map(|s| s.mean);
            let vol_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("population_volatility"))
                .map(|s| s.mean);
            if let (Some(first), Some(last)) = (vol_first, vol_last) {
                findings.push(format!(
                    "Population volatility: {:.2} at deployment vs {:.2} at hoarding",
                    first, last
                ));
            }

            // Birth/death ratio
            let bdr_first = result.steps.first()
                .and_then(|s| s.metric_summaries.get("birth_death_ratio"))
                .map(|s| s.mean);
            let bdr_last = result.steps.last()
                .and_then(|s| s.metric_summaries.get("birth_death_ratio"))
                .map(|s| s.mean);
            if let (Some(first), Some(last)) = (bdr_first, bdr_last) {
                findings.push(format!(
                    "Birth/death ratio: {:.4} at deployment vs {:.4} at hoarding",
                    first, last
                ));
            }
        }

        _ => {}
    }

    findings
}
