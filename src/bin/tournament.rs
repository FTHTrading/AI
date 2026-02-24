// Genesis Protocol -- Season 1 Week 4: THE TOURNAMENT
//
// Three rounds of metabolic warfare:
//   Round 1: The Oxygen Attack    (replication cost 1x-5x)
//   Round 2: The Starvation       (basal cost 1x-10x)
//   Round 3: The Final Escalation (both simultaneously)
//
// All under maximum hostility (every protection stripped).
// 580 worlds. 290,000 total epochs.
//
// Usage: cargo run --release --bin tournament

use genesis_experiment::{
    ExperimentRunner, ExperimentReport, FlagshipExperiments,
};
use std::time::Instant;

fn main() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  GENESIS PROTOCOL — SEASON 1 WEEK 4: THE TOURNAMENT            ║");
    println!("║  Can we kill it?                                                ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Season 1 arc:");
    println!("    Week 2: Killed mutation → survived");
    println!("    Week 3: Killed immune system → survived");
    println!("    Multi-axis: Everything hostile → survived (0/220 collapsed)");
    println!("    Week 4: Attack METABOLISM directly — the final test");
    println!();
    println!("  Previous attacks targeted weather (catastrophe, entropy).");
    println!("  The Tournament targets OXYGEN — the cost of reproducing and existing.");
    println!();

    let total_start = Instant::now();

    // ============ ROUND 1: THE OXYGEN ATTACK ============

    println!("══════════════════════════════════════════════════════════════════");
    println!("  ROUND 1: THE OXYGEN ATTACK");
    println!("  Sweep: replication cost 1x (25 ATP) → 5x (125 ATP)");
    println!("══════════════════════════════════════════════════════════════════");
    println!();

    let config_r1 = FlagshipExperiments::metabolic_inversion();
    println!("  Worlds: {}  |  Epochs per world: {}  |  Total epochs: {}",
        config_r1.total_worlds(), config_r1.epochs_per_run,
        config_r1.total_worlds() as u64 * config_r1.epochs_per_run);
    println!();

    let start = Instant::now();
    let result_r1 = ExperimentRunner::run(&config_r1);
    let elapsed_r1 = start.elapsed();

    println!("  Completed in {:.2}s", elapsed_r1.as_secs_f64());
    println!("  Result hash: {}", &result_r1.result_hash);
    println!();

    print_results_table(&result_r1, "repl_cost_x");
    let r1_collapses = count_collapses(&result_r1);

    // ============ ROUND 2: THE STARVATION ============

    println!();
    println!("══════════════════════════════════════════════════════════════════");
    println!("  ROUND 2: THE STARVATION");
    println!("  Sweep: basal cost 1x (0.15 ATP) → 10x (1.5 ATP)");
    println!("══════════════════════════════════════════════════════════════════");
    println!();

    let config_r2 = FlagshipExperiments::basal_inversion();
    println!("  Worlds: {}  |  Epochs per world: {}  |  Total epochs: {}",
        config_r2.total_worlds(), config_r2.epochs_per_run,
        config_r2.total_worlds() as u64 * config_r2.epochs_per_run);
    println!();

    let start = Instant::now();
    let result_r2 = ExperimentRunner::run(&config_r2);
    let elapsed_r2 = start.elapsed();

    println!("  Completed in {:.2}s", elapsed_r2.as_secs_f64());
    println!("  Result hash: {}", &result_r2.result_hash);
    println!();

    print_results_table(&result_r2, "basal_cost_x");
    let r2_collapses = count_collapses(&result_r2);

    // ============ ROUND 3: THE FINAL ESCALATION ============

    println!();
    println!("══════════════════════════════════════════════════════════════════");
    println!("  ROUND 3: THE FINAL ESCALATION");
    println!("  Fixed: replication cost 3x (75 ATP)");
    println!("  Sweep: basal cost 1x → 10x ON TOP of 3x replication");
    println!("══════════════════════════════════════════════════════════════════");
    println!();

    let config_r3 = FlagshipExperiments::dual_inversion();
    println!("  Worlds: {}  |  Epochs per world: {}  |  Total epochs: {}",
        config_r3.total_worlds(), config_r3.epochs_per_run,
        config_r3.total_worlds() as u64 * config_r3.epochs_per_run);
    println!();

    let start = Instant::now();
    let result_r3 = ExperimentRunner::run(&config_r3);
    let elapsed_r3 = start.elapsed();

    println!("  Completed in {:.2}s", elapsed_r3.as_secs_f64());
    println!("  Result hash: {}", &result_r3.result_hash);
    println!();

    print_results_table(&result_r3, "basal_x(+3x_repl)");
    let r3_collapses = count_collapses(&result_r3);

    let total_elapsed = total_start.elapsed();

    // ============ CROSS-ROUND SYNTHESIS ============

    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  THE TOURNAMENT — FINAL SCOREBOARD                             ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    let total_worlds = result_r1.total_worlds + result_r2.total_worlds + result_r3.total_worlds;
    let total_collapsed = r1_collapses.0 + r2_collapses.0 + r3_collapses.0;

    println!("  Round 1 — The Oxygen Attack:    {:>3} / {:>3} collapsed ({:.1}%)",
        r1_collapses.0, r1_collapses.1,
        (r1_collapses.0 as f64 / r1_collapses.1 as f64) * 100.0);
    println!("  Round 2 — The Starvation:       {:>3} / {:>3} collapsed ({:.1}%)",
        r2_collapses.0, r2_collapses.1,
        (r2_collapses.0 as f64 / r2_collapses.1 as f64) * 100.0);
    println!("  Round 3 — The Final Escalation: {:>3} / {:>3} collapsed ({:.1}%)",
        r3_collapses.0, r3_collapses.1,
        (r3_collapses.0 as f64 / r3_collapses.1 as f64) * 100.0);
    println!();
    println!("  TOTAL: {} / {} worlds collapsed", total_collapsed, total_worlds);
    println!("  Duration: {:.2}s ({} worlds, {} total epochs)",
        total_elapsed.as_secs_f64(), total_worlds,
        result_r1.total_epochs_run + result_r2.total_epochs_run + result_r3.total_epochs_run);
    println!();

    // --- Find the first collapse boundary ---
    let mut first_collapse_round = None;

    for (round_name, result) in [
        ("Round 1 (Oxygen Attack)", &result_r1),
        ("Round 2 (Starvation)", &result_r2),
        ("Round 3 (Final Escalation)", &result_r3),
    ] {
        for step in &result.steps {
            let collapsed: usize = step.trials.iter()
                .filter(|t| t.collapse_epoch.is_some())
                .count();
            if collapsed > 0 && first_collapse_round.is_none() {
                first_collapse_round = Some((
                    round_name,
                    step.parameter_value,
                    collapsed,
                    step.trials.len(),
                ));
            }
        }
    }

    if let Some((round, param, collapsed, total)) = first_collapse_round {
        println!("  *** FIRST COLLAPSE FOUND ***");
        println!("  {}: param={:.1}, {} of {} worlds died", round, param, collapsed, total);
        println!();

        // Print all collapse details
        println!("  COLLAPSE DETAILS:");
        for (round_name, result) in [
            ("Round 1", &result_r1),
            ("Round 2", &result_r2),
            ("Round 3", &result_r3),
        ] {
            for step in &result.steps {
                let collapsed_trials: Vec<_> = step.trials.iter()
                    .filter(|t| t.collapse_epoch.is_some())
                    .collect();
                if !collapsed_trials.is_empty() {
                    println!("    {} param={:.1}: {} of {} collapsed",
                        round_name, step.parameter_value,
                        collapsed_trials.len(), step.trials.len());
                    for t in &collapsed_trials {
                        println!("      Trial {} (seed {}): died at epoch {}, pop={}",
                            t.run_index, t.seed, t.collapse_epoch.unwrap(), t.final_population);
                    }
                }
            }
        }
    } else {
        println!("  *** THE ORGANISM IS UNKILLABLE ***");
        println!("  Zero collapses across {} worlds under metabolic warfare.", total_worlds);
        println!("  Architecture alone sustains the organism.");
        println!("  No combination of parameter stress in the current design space");
        println!("  can induce population collapse.");
    }

    println!();

    // --- Population trend analysis ---
    println!("══════════════════════════════════════════════════════════════════");
    println!("  POPULATION TRAJECTORIES");
    println!("══════════════════════════════════════════════════════════════════");
    println!();

    for (round_name, result) in [
        ("R1 Oxygen", &result_r1),
        ("R2 Starvation", &result_r2),
        ("R3 Dual", &result_r3),
    ] {
        print!("  {:12}  | ", round_name);
        for step in &result.steps {
            let mean_pop: f64 = step.trials.iter()
                .map(|t| *t.metrics.get("mean_population").unwrap_or(&0.0))
                .sum::<f64>() / step.trials.len() as f64;
            print!("{:>5.1} ", mean_pop);
        }
        println!();
    }
    println!();

    // --- Birth/Death ratio trend ---
    println!("══════════════════════════════════════════════════════════════════");
    println!("  BIRTH/DEATH RATIO TRAJECTORIES");
    println!("══════════════════════════════════════════════════════════════════");
    println!();

    for (round_name, result) in [
        ("R1 Oxygen", &result_r1),
        ("R2 Starvation", &result_r2),
        ("R3 Dual", &result_r3),
    ] {
        print!("  {:12}  | ", round_name);
        for step in &result.steps {
            let bdr: f64 = step.trials.iter()
                .map(|t| *t.metrics.get("birth_death_ratio").unwrap_or(&0.0))
                .sum::<f64>() / step.trials.len() as f64;
            print!("{:>5.2} ", bdr);
        }
        println!();
    }
    println!();

    // ---- Save reports for all three rounds ----
    save_round_report("experiments/metabolic_inversion", "metabolic_inversion",
        &result_r1, r1_collapses, "Oxygen Attack (replication cost 1x-5x)");
    save_round_report("experiments/basal_inversion", "basal_inversion",
        &result_r2, r2_collapses, "Starvation (basal cost 1x-10x)");
    save_round_report("experiments/dual_inversion", "dual_inversion",
        &result_r3, r3_collapses, "Final Escalation (3x repl + 1-10x basal)");

    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  THE TOURNAMENT — COMPLETE                                     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_results_table(result: &genesis_experiment::runner::ExperimentResult, label: &str) {
    println!("  {:<18} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
        label, "collapse%", "surv_ep", "mean_pop", "fit", "births", "deaths");
    println!("  {}", "-".repeat(78));

    for step in &result.steps {
        let n = step.trials.len() as f64;
        let collapsed: usize = step.trials.iter()
            .filter(|t| t.collapse_epoch.is_some()).count();
        let collapse_pct = (collapsed as f64 / n) * 100.0;

        let survival = step.trials.iter()
            .map(|t| *t.metrics.get("survival_epochs").unwrap_or(&500.0))
            .sum::<f64>() / n;
        let mean_pop = step.trials.iter()
            .map(|t| *t.metrics.get("mean_population").unwrap_or(&0.0))
            .sum::<f64>() / n;
        let fit = step.trials.iter()
            .map(|t| *t.metrics.get("mean_fitness").unwrap_or(&0.0))
            .sum::<f64>() / n;
        let births = step.trials.iter()
            .map(|t| *t.metrics.get("total_births").unwrap_or(&0.0))
            .sum::<f64>() / n;
        let deaths = step.trials.iter()
            .map(|t| *t.metrics.get("total_deaths").unwrap_or(&0.0))
            .sum::<f64>() / n;

        println!("  {:<18.1} {:>8.1}% {:>9.1} {:>9.1} {:>9.4} {:>9.0} {:>9.0}",
            step.parameter_value, collapse_pct, survival, mean_pop, fit, births, deaths);
    }
}

fn count_collapses(result: &genesis_experiment::runner::ExperimentResult) -> (usize, usize) {
    let mut collapsed = 0;
    let mut total = 0;
    for step in &result.steps {
        total += step.trials.len();
        collapsed += step.trials.iter().filter(|t| t.collapse_epoch.is_some()).count();
    }
    (collapsed, total)
}

fn save_round_report(
    output_dir: &str,
    slug: &str,
    result: &genesis_experiment::runner::ExperimentResult,
    collapses: (usize, usize),
    description: &str,
) {
    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let findings = if collapses.0 > 0 {
        vec![
            format!("COLLAPSE FOUND: {} of {} worlds collapsed", collapses.0, collapses.1),
            format!("Attack vector: {}", description),
            "The organism has a metabolic boundary condition.".into(),
        ]
    } else {
        vec![
            format!("NO COLLAPSE: 0 of {} worlds survived {}", collapses.1, description),
            "Structural architecture sustains the organism under metabolic stress.".into(),
            "The boundary, if it exists, lies beyond the tested parameter space.".into(),
        ]
    };

    let report = ExperimentReport::generate(result, findings);
    report.save_to_dir_with_slug(output_dir, Some(slug))
        .expect("Failed to save report");

    println!("  Saved: {}/", output_dir);
}
