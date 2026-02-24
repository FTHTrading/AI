// Genesis Protocol -- Metabolic Inversion
//
// "You attacked weather. You didn't attack oxygen."
//
// The multi-axis collapse experiment proved environmental hostility
// irrelevant. Population converges to ~46 regardless of soft_cap,
// entropy, catastrophe, or redistribution policy.
//
// This experiment attacks the metabolic substrate itself.
// Sweeps replication_cost_multiplier from 1.0x to 5.0x (step 0.5).
// At 5.0x, effective replication cost = 125 ATP (exceeds PRIMORDIAL_GRANT of 50).
//
// Question: At what cost does demographic replacement fail?
//
// 9 steps × 20 runs × 500 epochs = 180 worlds, 90,000 total epochs.
//
// Usage: cargo run --release --bin metabolic_inversion

use genesis_experiment::{
    ExperimentRunner, ExperimentReport, FlagshipExperiments,
};
use std::time::Instant;

fn main() {
    println!("======================================================================");
    println!("  GENESIS PROTOCOL -- METABOLIC INVERSION");
    println!("  The Oxygen Attack");
    println!("======================================================================");
    println!();

    let config = FlagshipExperiments::metabolic_inversion();

    println!("  Hypothesis: {}", config.hypothesis);
    println!();
    println!("  Protocol:");
    println!("    Sweep variable:  ReplicationCostMultiplier");
    println!("    Range:           1.0x -> 5.0x (step 0.5, 9 levels)");
    println!("    Effective cost:  25 ATP -> 125 ATP");
    println!("    Runs per step:   {}", config.runs_per_step);
    println!("    Epochs per run:  {}", config.epochs_per_run);
    println!("    Total worlds:    {}", config.total_worlds());
    println!("    Total epochs:    {}", config.total_worlds() as u64 * config.epochs_per_run);
    println!("    Base seed:       {}", config.base_seed);
    println!();
    println!("  Hostile axes (ALL locked from multi-axis):");
    println!("    Mutation:        DISABLED (rate = 0.0)");
    println!("    Cortex/Immune:   DISABLED");
    println!("    Redistribution:  DISABLED (threshold = 1.0, rate = 0.0)");
    println!("    Treasury deploy: DISABLED (threshold = 1.0)");
    println!("    Catastrophe:     MAXIMUM (0.03)");
    println!("    Entropy:         MAXIMUM (0.0001, 10x default)");
    println!();
    println!("  Metabolic attack:");
    println!("    REPLICATION_COST base = 25.0 ATP");
    println!("    Multiplier sweep = 1.0 .. 5.0");
    println!("    At 5.0x: cost = 125 ATP (exceeds PRIMORDIAL_GRANT of 50)");
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
    println!("  RESULTS BY REPLICATION COST MULTIPLIER");
    println!("======================================================================");
    println!();
    println!("{:<10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "repl_mul", "eff_cost", "collapse%", "surv_ep", "mean_pop",
        "mean_fit", "births", "deaths", "b/d_ratio");
    println!("{}", "-".repeat(100));

    let mut any_collapse = false;
    let mut total_collapsed = 0usize;
    let mut total_trials = 0usize;
    let mut first_collapse_mult: Option<f64> = None;
    let mut boundary_mult: Option<f64> = None;

    for step in &result.steps {
        let mult = step.parameter_value;
        let effective_cost = 25.0 * mult;
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
            if first_collapse_mult.is_none() {
                first_collapse_mult = Some(mult);
            }
        }
        if collapse_pct >= 50.0 && boundary_mult.is_none() {
            boundary_mult = Some(mult);
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
        let births: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("total_births").unwrap_or(&0.0))
            .sum::<f64>() / n_trials as f64;
        let deaths: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("total_deaths").unwrap_or(&0.0))
            .sum::<f64>() / n_trials as f64;
        let bd_ratio = if deaths > 0.0 { births / deaths } else { f64::INFINITY };

        println!("{:<10.1} {:>10.0} {:>9.1}% {:>10.1} {:>10.1} {:>10.4} {:>10.0} {:>10.0} {:>10.4}",
            mult, effective_cost, collapse_pct, survival_epochs, mean_pop,
            mean_fit, births, deaths, bd_ratio);
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
        println!("  *** METABOLIC BOUNDARY FOUND ***");
        println!("  Demographic replacement fails when replication cost is sufficiently high.");
        println!("  The attractor IS a function of metabolic economics.");
        println!();
        if let Some(first) = first_collapse_mult {
            println!("  First collapse at:     {:.1}x (effective cost: {:.0} ATP)",
                first, 25.0 * first);
        }
        if let Some(boundary) = boundary_mult {
            println!("  Boundary (>=50%):      {:.1}x (effective cost: {:.0} ATP)",
                boundary, 25.0 * boundary);
        }
        println!();
        println!("  The equilibrium at ~46 is metabolically determined.");
        println!("  Environmental immunity holds, but metabolic inversion breaks the attractor.");
    } else {
        println!("  *** NO COLLAPSE FOUND ***");
        println!("  Even at 5.0x replication cost (125 ATP), the system survives.");
        println!("  The metabolic boundary, if it exists, lies beyond the tested range.");
        println!("  Consider extending the sweep range or attacking basal cost instead.");
    }
    println!();

    // ---- Demographic transition analysis ----
    println!("======================================================================");
    println!("  DEMOGRAPHIC TRANSITION ANALYSIS");
    println!("======================================================================");
    println!();
    println!("  Population dynamics by multiplier:");
    println!();

    for step in &result.steps {
        let mult = step.parameter_value;
        let n = step.trials.len();
        let mean_pop: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("mean_population").unwrap_or(&0.0))
            .sum::<f64>() / n as f64;
        let births: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("total_births").unwrap_or(&0.0))
            .sum::<f64>() / n as f64;
        let deaths: f64 = step.trials.iter()
            .map(|t| *t.metrics.get("total_deaths").unwrap_or(&0.0))
            .sum::<f64>() / n as f64;
        let collapsed: usize = step.trials.iter()
            .filter(|t| t.collapse_epoch.is_some())
            .count();

        let bar_len = (mean_pop / 50.0 * 30.0).min(30.0) as usize;
        let bar: String = "#".repeat(bar_len);
        let collapse_marker = if collapsed > 0 {
            format!(" [{}% DEAD]", (collapsed * 100) / n)
        } else {
            String::new()
        };

        println!("  {:.1}x | {:>5.1} pop | b={:>5.0} d={:>5.0} | {}{}", 
            mult, mean_pop, births, deaths, bar, collapse_marker);
    }
    println!();

    // ---- Collapse detail ----
    println!("======================================================================");
    println!("  COLLAPSE DETAIL");
    println!("======================================================================");
    println!();
    let mut any_detail = false;
    for step in &result.steps {
        let collapsed_trials: Vec<_> = step.trials.iter()
            .filter(|t| t.collapse_epoch.is_some())
            .collect();
        if !collapsed_trials.is_empty() {
            any_detail = true;
            println!("  repl_mult={:.1}x (cost={:.0} ATP): {} of {} collapsed",
                step.parameter_value,
                25.0 * step.parameter_value,
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
    if !any_detail {
        println!("  No collapses recorded at any multiplier level.");
        println!();
    }

    // ---- Save full report ----
    let output_dir = "experiments/metabolic_inversion";
    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let findings = if any_collapse {
        let mut f = vec![
            format!("METABOLIC BOUNDARY FOUND: {} of {} worlds collapsed ({:.1}%)",
                total_collapsed, total_trials,
                (total_collapsed as f64 / total_trials as f64) * 100.0),
            "Environmental immunity holds, but metabolic inversion breaks the attractor".into(),
            "The attractor equilibrium (~46) is metabolically determined".into(),
        ];
        if let Some(first) = first_collapse_mult {
            f.push(format!("First collapse at {:.1}x (effective cost {:.0} ATP)", first, 25.0 * first));
        }
        if let Some(boundary) = boundary_mult {
            f.push(format!("Boundary (>=50% collapse) at {:.1}x (effective cost {:.0} ATP)", boundary, 25.0 * boundary));
        }
        f
    } else {
        vec![
            format!("NO COLLAPSE: 0 of {} worlds survived at all multiplier levels", total_trials),
            "Metabolic inversion up to 5.0x (125 ATP) insufficient to break attractor".into(),
            "The metabolic boundary, if it exists, lies beyond 5.0x replication cost".into(),
        ]
    };

    let report = ExperimentReport::generate(&result, findings);
    report.save_to_dir_with_slug(output_dir, Some("metabolic_inversion"))
        .expect("Failed to save report");

    println!("  Full report saved to {}/", output_dir);
    println!("    - metabolic_inversion_report.txt");
    println!("    - metabolic_inversion_data.csv");
    println!("    - metabolic_inversion_manifest.json");
    println!();
    println!("======================================================================");
    println!("  EXPERIMENT COMPLETE");
    println!("======================================================================");
}
