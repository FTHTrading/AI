// Genesis Protocol -- Multi-Axis Collapse Vector
//
// The boundary search experiment.
// Every survival mechanism stripped simultaneously.
// Sweeps carrying capacity from 30 to 180.
// 220 worlds, 110,000 total epochs.
//
// Usage: cargo run --release --bin multi_axis_collapse

use genesis_experiment::{
    ExperimentRunner, ExperimentReport, FlagshipExperiments,
};
use std::time::Instant;

fn main() {
    println!("======================================================================");
    println!("  GENESIS PROTOCOL -- MULTI-AXIS COLLAPSE VECTOR");
    println!("  Attractor Boundary Search");
    println!("======================================================================");
    println!();

    let config = FlagshipExperiments::multi_axis_collapse();

    println!("  Hypothesis: {}", config.hypothesis);
    println!();
    println!("  Protocol:");
    println!("    Sweep variable:  SoftCap (carrying capacity)");
    println!("    Range:           30 -> 180 (step 15, 11 levels)");
    println!("    Runs per step:   {}", config.runs_per_step);
    println!("    Epochs per run:  {}", config.epochs_per_run);
    println!("    Total worlds:    {}", config.total_worlds());
    println!("    Total epochs:    {}", config.total_worlds() as u64 * config.epochs_per_run);
    println!("    Base seed:       {}", config.base_seed);
    println!();
    println!("  Hostile axes (ALL locked):");
    println!("    Mutation:        DISABLED (rate = 0.0)");
    println!("    Cortex/Immune:   DISABLED");
    println!("    Redistribution:  DISABLED (threshold = 1.0, rate = 0.0)");
    println!("    Treasury deploy: DISABLED (threshold = 1.0)");
    println!("    Catastrophe:     MAXIMUM (0.03)");
    println!("    Entropy:         MAXIMUM (0.0001, 10x default)");
    println!();
    println!("  Running experiment...");
    println!();

    let start = Instant::now();
    let result = ExperimentRunner::run(&config);
    let elapsed = start.elapsed();

    println!("  Completed in {:.2}s", elapsed.as_secs_f64());
    println!("  Total epochs run: {}", result.total_epochs_run);
    println!("  Result hash: {}", &result.result_hash);
    println!();

    // ---- Per-Step Analysis ----

    println!("======================================================================");
    println!("  RESULTS BY CARRYING CAPACITY");
    println!("======================================================================");
    println!();
    println!("{:<12} {:>10} {:>10} {:>12} {:>12} {:>12} {:>10}",
        "soft_cap", "collapse%", "surv_ep", "mean_pop", "mean_fit", "gini", "deaths");
    println!("{}", "-".repeat(80));

    let mut any_collapse = false;
    let mut total_collapsed = 0usize;
    let mut total_trials = 0usize;

    for step in &result.steps {
        let soft_cap = step.parameter_value;
        let n_trials = step.trials.len();
        total_trials += n_trials;

        // Count collapses
        let collapsed: usize = step.trials.iter()
            .filter(|t| t.collapse_epoch.is_some())
            .count();
        total_collapsed += collapsed;
        let collapse_pct = (collapsed as f64 / n_trials as f64) * 100.0;

        if collapsed > 0 {
            any_collapse = true;
        }

        // Extract metric means
        let survival_epochs: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("survival_epochs").unwrap_or(&500.0))
            .sum::<f64>() / n_trials as f64;
        let mean_pop: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("mean_population").unwrap_or(&0.0))
            .sum::<f64>() / n_trials as f64;
        let mean_fit: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("mean_fitness").unwrap_or(&0.0))
            .sum::<f64>() / n_trials as f64;
        let gini: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("gini_coefficient").unwrap_or(&0.0))
            .sum::<f64>() / n_trials as f64;
        let deaths: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("total_deaths").unwrap_or(&0.0))
            .sum::<f64>() / n_trials as f64;

        println!("{:<12.0} {:>9.1}% {:>10.1} {:>12.1} {:>12.4} {:>12.4} {:>10.0}",
            soft_cap, collapse_pct, survival_epochs, mean_pop, mean_fit, gini, deaths);
    }

    println!();
    println!("======================================================================");
    println!("  SUMMARY");
    println!("======================================================================");
    println!();
    println!("  Total worlds:          {}", total_trials);
    println!("  Total collapsed:       {}", total_collapsed);
    println!("  Overall collapse rate: {:.1}%", (total_collapsed as f64 / total_trials as f64) * 100.0);
    println!("  Duration:              {:.2}s", elapsed.as_secs_f64());
    println!();

    if any_collapse {
        println!("  *** BOUNDARY FOUND ***");
        println!("  The attractor has a boundary condition.");
        println!("  Collapse occurs when all survival mechanisms are stripped");
        println!("  and carrying capacity is compressed.");
        println!();

        // Identify the boundary
        let mut boundary_cap = None;
        for step in result.steps.iter().rev() {
            let collapsed: usize = step.trials.iter()
                .filter(|t| t.collapse_epoch.is_some())
                .count();
            let collapse_pct = (collapsed as f64 / step.trials.len() as f64) * 100.0;
            if collapse_pct >= 50.0 {
                boundary_cap = Some(step.parameter_value);
                break;
            }
        }

        if let Some(cap) = boundary_cap {
            println!("  Boundary (>=50% collapse): soft_cap = {:.0}", cap);
        }
    } else {
        println!("  *** NO COLLAPSE FOUND ***");
        println!("  Structural immunity is proven under correlated multi-axis stress.");
        println!("  The attractor has no boundary within the tested parameter space.");
        println!("  Even with ALL protections stripped and maximum hostility,");
        println!("  the system survives at every capacity level tested.");
    }
    println!();

    // ---- Detailed per-trial collapse analysis ----
    println!("======================================================================");
    println!("  COLLAPSE DETAIL (if any)");
    println!("======================================================================");
    println!();
    for step in &result.steps {
        let collapsed_trials: Vec<_> = step.trials.iter()
            .filter(|t| t.collapse_epoch.is_some())
            .collect();
        if !collapsed_trials.is_empty() {
            println!("  soft_cap={:.0}: {} of {} collapsed", step.parameter_value,
                collapsed_trials.len(), step.trials.len());
            for t in &collapsed_trials {
                println!("    Trial {} (seed {}): collapsed at epoch {}, final_pop={}",
                    t.run_index, t.seed,
                    t.collapse_epoch.unwrap(),
                    t.final_population);
            }
            println!();
        }
    }

    // ---- Save full report ----
    let output_dir = "experiments/multi_axis_collapse";
    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let findings = if any_collapse {
        vec![
            format!("BOUNDARY FOUND: {} of {} worlds collapsed ({:.1}%)",
                total_collapsed, total_trials,
                (total_collapsed as f64 / total_trials as f64) * 100.0),
            "Collapse occurs under correlated multi-axis stress with reduced carrying capacity".into(),
            "The attractor is NOT unconditionally stable -- it has a boundary condition".into(),
        ]
    } else {
        vec![
            format!("NO COLLAPSE: 0 of {} worlds collapsed under maximum hostility", total_trials),
            "Structural immunity holds even with ALL survival mechanisms stripped simultaneously".into(),
            "The attractor boundary, if it exists, lies beyond the tested parameter space".into(),
        ]
    };

    let report = ExperimentReport::generate(&result, findings);
    report.save_to_dir_with_slug(output_dir, Some("multi_axis_collapse"))
        .expect("Failed to save report");

    println!("  Full report saved to {}/", output_dir);
    println!("    - multi_axis_collapse_report.txt");
    println!("    - multi_axis_collapse_data.csv");
    println!("    - multi_axis_collapse_manifest.json");
    println!();
    println!("======================================================================");
    println!("  EXPERIMENT COMPLETE");
    println!("======================================================================");
}
