// Genesis Protocol — Season 2: Structural Invariant S4
//
// Energy Topology Violations — Thermodynamic Collapse Boundary
// Tests whether attacking the energy loop itself (resource inflow, death outflow)
// produces collapse where governance parameter violations (S1–S3) did not.
//
// Supervisor v2 directive: "The collapse boundary lies in energy topology,
// not governance parameters. Proceed to topology-level experiments."
//
// 5 experiments, all under hostile conditions:
//   S4-A: Zero Regeneration           → finite resource universe
//   S4-B: Death Sink                  → death actively destroys pool ATP
//   S4-C: Zero Regen + Death Sink     → double topology violation
//   S4-D: Full Attack                 → all topology + all safety OFF + 10× replication cost
//   S4-E: Extended Horizon (5000 ep)  → slow-drift collapse detection
//
// Usage: cargo run --release --bin s4_topology_violations

use genesis_experiment::{ExperimentRunner, ExperimentReport, FlagshipExperiments};
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  SEASON 2 — COLLAPSE BOUNDARY: S4 TOPOLOGY VIOLATIONS  ║");
    println!("║     Energy Loop Attack Experiment Suite                 ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let output_dir = "experiments/season2";
    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let experiments = FlagshipExperiments::s4_topology_suite();

    let global_start = Instant::now();
    let mut all_findings: Vec<(String, String, Vec<String>)> = Vec::new();

    for (slug, config) in &experiments {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Experiment: {}", config.name);
        println!("  Hypothesis: {}", config.hypothesis);
        println!("  Worlds: {} | Epochs/world: {} | Total epochs: {}",
            config.total_worlds(),
            config.epochs_per_run,
            config.total_worlds() as u64 * config.epochs_per_run,
        );
        let stress = config.base_stress_override.as_ref().unwrap();
        println!("  Resource Regeneration: {}", if stress.resource_regeneration_enabled { "ON" } else { "OFF" });
        println!("  Death Drains Resources: {}", if stress.death_drains_resources { "YES" } else { "NO" });
        println!("  ATP Decay: {}", if stress.atp_decay_enabled { "ON" } else { "OFF" });
        println!("  Treasury Cycling: {}", if stress.treasury_cycling_enabled { "ON" } else { "OFF" });
        println!("  Reproduction Grants: {}", if stress.reproduction_grants_enabled { "ON" } else { "OFF" });
        println!("  Extinction Floor: {}", if stress.extinction_floor_enabled { "ON" } else { "OFF" });
        println!("  Replication Cost Mult: {:.1}×", stress.replication_cost_multiplier);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let start = Instant::now();
        let result = ExperimentRunner::run(config);
        let elapsed = start.elapsed();

        println!("  Completed in {:.2}s", elapsed.as_secs_f64());
        println!("  Total epochs run: {}", result.total_epochs_run);
        println!("  Result hash: {}", &result.result_hash);
        println!();

        // Analyze topology-specific collapse signatures
        let findings = derive_s4_findings(slug, &result);
        for (i, f) in findings.iter().enumerate() {
            println!("  Finding {}: {}", i + 1, f);
        }
        println!();

        // Generate report
        let report = ExperimentReport::generate(&result, findings.clone());
        let dir = format!("{}/{}", output_dir, slug);
        report.save_to_dir_with_slug(&dir, Some(slug)).expect("Failed to save report");
        println!("  Saved: {}/", dir);
        println!();

        all_findings.push((slug.to_string(), config.name.clone(), findings));
    }

    // ─── Cross-experiment topology analysis ─────────────────────────
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  S4 ENERGY TOPOLOGY VIOLATIONS — SYNTHESIS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for (slug, name, findings) in &all_findings {
        println!("\n  {} [{}]:", name, slug);
        for f in findings {
            println!("    • {}", f);
        }
    }

    // ─── TOPOLOGY ESCALATION SUMMARY ────────────────────────────────
    println!("\n  ── TOPOLOGY ESCALATION SUMMARY ──");
    println!("  (Progressive energy loop disruption)");
    println!();

    for (_slug, name, findings) in &all_findings {
        let collapse_line = findings.iter()
            .find(|f| f.starts_with("Collapse rate:"))
            .cloned()
            .unwrap_or_else(|| "Collapse rate: unknown".to_string());
        println!("  {:50} → {}", name, collapse_line);

        // Show per-step details for collapsed experiments
        if findings.iter().any(|f| f.contains("COLLAPSE DETECTED") || f.contains("EXTINCTION CONFIRMED")) {
            for f in findings.iter().filter(|f| f.starts_with("  cap=")) {
                println!("    {}", f);
            }
        }
    }

    // ─── THERMODYNAMIC VERDICT ──────────────────────────────────────
    println!();
    println!("  ── S4 THERMODYNAMIC VERDICT ──");

    let any_collapse = all_findings.iter()
        .any(|(_, _, fs)| fs.iter().any(|f|
            f.contains("COLLAPSE DETECTED") || f.contains("EXTINCTION CONFIRMED") || f.contains("COLLAPSE BOUNDARY FOUND")
        ));

    let total_worlds_s4 = 120 * 4 + 60; // 4 standard + 1 extended

    if any_collapse {
        println!("  ★ THERMODYNAMIC COLLAPSE BOUNDARY LOCATED");
        println!("  Energy topology attacks produce extinction where governance violations could not.");
        println!();

        for (_slug, name, findings) in &all_findings {
            let collapsed = findings.iter().any(|f|
                f.contains("COLLAPSE DETECTED") || f.contains("EXTINCTION CONFIRMED") || f.contains("COLLAPSE BOUNDARY FOUND")
            );
            let status = if collapsed { "COLLAPSED ✗" } else { "SURVIVED ✓" };
            println!("    {} → {}", name, status);
        }

        println!();
        println!("  Interpretation: The Genesis Protocol's anti-fragility emerges from");
        println!("  positive closed-loop energy conservation with enforced throughput,");
        println!("  NOT from governance parameters. Collapse requires attacking the");
        println!("  thermodynamic substrate itself.");
    } else {
        println!("  ★ NO COLLAPSE — Even topology-level attacks do not produce extinction");
        println!("  The system demonstrates thermodynamic resilience beyond energy loop integrity.");
        println!("  The anti-fragility mechanism is deeper than energy topology.");
    }

    println!();
    println!("  Cumulative Season 2: {} experiments, {} worlds",
        5 + 4 + 2 + 2, // S4 + S3 + S2 + S1
        total_worlds_s4 + 480 + 240 + 240,
    );

    let elapsed = global_start.elapsed();
    println!("\n  Total S4 time: {:.2}s", elapsed.as_secs_f64());
    println!("  Output directory: {}/", output_dir);
}

fn derive_s4_findings(slug: &str, result: &genesis_experiment::ExperimentResult) -> Vec<String> {
    let mut findings = Vec::new();

    // Flatten all trials from all steps
    let all_trials: Vec<_> = result.steps.iter()
        .flat_map(|s| s.trials.iter())
        .collect();

    let total_trials = all_trials.len();
    let collapsed_trials: Vec<_> = all_trials.iter()
        .filter(|t| t.collapse_epoch.is_some())
        .collect();
    let collapse_count = collapsed_trials.len();
    let collapse_rate = collapse_count as f64 / total_trials as f64 * 100.0;

    findings.push(format!(
        "Collapse rate: {}/{} trials ({:.1}%)",
        collapse_count, total_trials, collapse_rate
    ));

    if collapse_count > 0 {
        // Time to collapse statistics
        let collapse_epochs: Vec<u64> = collapsed_trials.iter()
            .map(|t| t.collapse_epoch.unwrap())
            .collect();
        let min_collapse = *collapse_epochs.iter().min().unwrap();
        let max_collapse = *collapse_epochs.iter().max().unwrap();
        let mean_collapse = collapse_epochs.iter().sum::<u64>() as f64 / collapse_epochs.len() as f64;

        findings.push(format!(
            "⚠ COLLAPSE DETECTED — time to collapse: min={}, max={}, mean={:.1} epochs",
            min_collapse, max_collapse, mean_collapse
        ));

        // Collapse by sweep step
        findings.push("Per-step collapse:".to_string());
        for step in &result.steps {
            let step_total = step.trials.len();
            let step_collapsed = step.trials.iter().filter(|t| t.collapse_epoch.is_some()).count();
            let rate = step_collapsed as f64 / step_total as f64 * 100.0;
            findings.push(format!(
                "  cap={:.0}: {}/{} collapsed ({:.1}%)",
                step.parameter_value, step_collapsed, step_total, rate
            ));
        }

        // Phase transition detection
        let step_rates: Vec<f64> = result.steps.iter()
            .map(|s| {
                let n = s.trials.len() as f64;
                let c = s.trials.iter().filter(|t| t.collapse_epoch.is_some()).count() as f64;
                c / n
            })
            .collect();
        for i in 1..step_rates.len() {
            let delta = (step_rates[i] - step_rates[i - 1]).abs();
            if delta > 0.3 {
                findings.push(format!(
                    "⚠ PHASE TRANSITION at cap={:.0}→{:.0}: collapse rate {:.0}%→{:.0}%",
                    result.steps[i - 1].parameter_value,
                    result.steps[i].parameter_value,
                    step_rates[i - 1] * 100.0,
                    step_rates[i] * 100.0,
                ));
            }
        }

        if collapse_rate > 90.0 {
            findings.push("★ EXTINCTION CONFIRMED — near-total collapse across all carrying capacities".to_string());
        } else if collapse_rate > 50.0 {
            findings.push("★ COLLAPSE BOUNDARY FOUND — majority of worlds collapse".to_string());
        }
    } else {
        findings.push(format!(
            "NO COLLAPSES — topology violation [{}] did NOT cause extinction",
            slug
        ));
    }

    // ─── RESOURCE DEPLETION ANALYSIS ────────────────────────────────
    // This is the topology-specific section — how fast do pools deplete?

    findings.push("── Resource Topology ──".to_string());

    // Mean resource level (if tracked)
    let mean_resources: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("mean_resource_level").copied())
        .collect();
    if !mean_resources.is_empty() {
        let v = mean_resources.iter().sum::<f64>() / mean_resources.len() as f64;
        let min_v = mean_resources.iter().cloned().fold(f64::INFINITY, f64::min);
        findings.push(format!("Mean resource level: mean={:.1}, min={:.1}", v, min_v));
        if min_v < 10.0 {
            findings.push("  ⚠ RESOURCE EXHAUSTION: Pool levels critically low".to_string());
        }
    }

    // Mean extraction per epoch
    let extractions: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("mean_resources_extracted").copied())
        .collect();
    if !extractions.is_empty() {
        let v = extractions.iter().sum::<f64>() / extractions.len() as f64;
        findings.push(format!("Mean extraction/epoch: {:.2}", v));
    }

    // ─── INEQUALITY METRICS ─────────────────────────────────────────

    findings.push("── Inequality Instrumentation ──".to_string());

    // Wealth Concentration Index
    let wci: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("wealth_concentration_index").copied())
        .collect();
    let mean_wci = if !wci.is_empty() {
        let v = wci.iter().sum::<f64>() / wci.len() as f64;
        let max_v = wci.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        findings.push(format!(
            "Wealth concentration (top 10%): mean={:.4}, max={:.4}",
            v, max_v
        ));
        if v > 0.5 {
            findings.push("  ⚠ WEALTH OLIGARCHY: Top 10% controls >50% of ATP".to_string());
        }
        v
    } else { 0.0 };

    // Mean Gini Coefficient
    let mgini: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("mean_gini_coefficient").copied())
        .collect();
    let mean_mgini = if !mgini.is_empty() {
        let v = mgini.iter().sum::<f64>() / mgini.len() as f64;
        let max_v = mgini.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        findings.push(format!("Mean Gini coefficient: mean={:.4}, max={:.4}", v, max_v));
        if v > 0.6 {
            findings.push("  ⚠ SEVERE INEQUALITY: Gini > 0.6".to_string());
        }
        v
    } else { 0.0 };

    // Median/Mean ATP Divergence
    let mmd: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("median_mean_atp_divergence").copied())
        .collect();
    if !mmd.is_empty() {
        let v = mmd.iter().sum::<f64>() / mmd.len() as f64;
        let max_v = mmd.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        findings.push(format!("Median/Mean ATP divergence: mean={:.4}, max={:.4}", v, max_v));
        if v > 0.3 {
            findings.push("  ⚠ RIGHT-SKEWED: Mean >> Median, wealth hoarding".to_string());
        }
    }

    // ATP Variance
    let atpv: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("atp_variance").copied())
        .collect();
    if !atpv.is_empty() {
        let v = atpv.iter().sum::<f64>() / atpv.len() as f64;
        let max_v = atpv.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        findings.push(format!("ATP variance: mean={:.1}, max={:.1}", v, max_v));
    }

    // Reproductive Inequality Index
    let rii: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("reproductive_inequality_index").copied())
        .collect();
    let mean_rii = if !rii.is_empty() {
        let v = rii.iter().sum::<f64>() / rii.len() as f64;
        let max_v = rii.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        findings.push(format!("Reproductive inequality: mean={:.4}, max={:.4}", v, max_v));
        if v > 0.5 {
            findings.push("  ⚠ REPRODUCTIVE MONOPOLY: Top quartile >50% of births".to_string());
        }
        v
    } else { 0.0 };

    // Survival Inequality Index
    let sii: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("survival_inequality_index").copied())
        .collect();
    let mean_sii = if !sii.is_empty() {
        let v = sii.iter().sum::<f64>() / sii.len() as f64;
        let max_v = sii.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        findings.push(format!("Survival inequality: mean={:.4}, max={:.4}", v, max_v));
        if v > 0.5 {
            findings.push("  ⚠ SURVIVAL APARTHEID: Bottom quartile >50% of deaths".to_string());
        }
        v
    } else { 0.0 };

    // Top Decile Persistence
    let tdp: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("top_decile_persistence").copied())
        .collect();
    let mean_tdp = if !tdp.is_empty() {
        let v = tdp.iter().sum::<f64>() / tdp.len() as f64;
        let max_v = tdp.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        findings.push(format!("Top decile persistence: mean={:.4}, max={:.4}", v, max_v));
        if v > 0.8 {
            findings.push("  ⚠ WEALTH IMMORTALITY: Top decile dominates >80% of time".to_string());
        }
        v
    } else { 0.0 };

    // ─── POPULATION STATISTICS ──────────────────────────────────────

    findings.push("── Population & Economy ──".to_string());

    let min_pops: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("min_population").copied())
        .collect();
    if !min_pops.is_empty() {
        let global_min = min_pops.iter().cloned().fold(f64::INFINITY, f64::min);
        let mean_min = min_pops.iter().sum::<f64>() / min_pops.len() as f64;
        findings.push(format!(
            "Population floor: global_min={:.0}, mean_min={:.1}",
            global_min, mean_min
        ));
        if global_min < 3.0 {
            findings.push("  ⚠ CRITICAL: Population reached extinction floor (<3 agents)".to_string());
        }
    }

    let mean_pops: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("mean_population").copied())
        .collect();
    if !mean_pops.is_empty() {
        let mean_pop = mean_pops.iter().sum::<f64>() / mean_pops.len() as f64;
        let min_pop = mean_pops.iter().cloned().fold(f64::INFINITY, f64::min);
        findings.push(format!(
            "Mean population: {:.1} (min trial: {:.1})", mean_pop, min_pop
        ));
    }

    let final_pops: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("final_population").copied())
        .collect();
    if !final_pops.is_empty() {
        let mean_final = final_pops.iter().sum::<f64>() / final_pops.len() as f64;
        let min_final = final_pops.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_final = final_pops.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        findings.push(format!(
            "Final population: mean={:.1}, min={:.0}, max={:.0}",
            mean_final, min_final, max_final
        ));
    }

    let bdr: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("birth_death_ratio").copied())
        .collect();
    if !bdr.is_empty() {
        let mean_bdr = bdr.iter().sum::<f64>() / bdr.len() as f64;
        let min_bdr = bdr.iter().cloned().fold(f64::INFINITY, f64::min);
        findings.push(format!(
            "Birth/death ratio: mean={:.4}, min={:.4}",
            mean_bdr, min_bdr
        ));
        if mean_bdr < 0.5 {
            findings.push("  ⚠ DEMOGRAPHIC COLLAPSE: Deaths exceed births 2:1".to_string());
        }
    }

    let births: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("total_births").copied())
        .collect();
    if !births.is_empty() {
        let mean_births = births.iter().sum::<f64>() / births.len() as f64;
        let min_births = births.iter().cloned().fold(f64::INFINITY, f64::min);
        findings.push(format!(
            "Total births: mean={:.1}, min={:.0}",
            mean_births, min_births
        ));
        if min_births < 1.0 {
            findings.push("  ⚠ DEMOGRAPHIC FREEZE: Some worlds had ZERO births".to_string());
        }
    }

    let max_reserves: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("max_treasury_reserve").copied())
        .collect();
    if !max_reserves.is_empty() {
        let global_max = max_reserves.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean_max = max_reserves.iter().sum::<f64>() / max_reserves.len() as f64;
        findings.push(format!(
            "Treasury accumulation: global_max={:.1}, mean_max={:.1} ATP",
            global_max, mean_max
        ));
    }

    let fitnesses: Vec<f64> = all_trials.iter()
        .filter_map(|t| t.metrics.get("mean_fitness").copied())
        .collect();
    if !fitnesses.is_empty() {
        let mean_fit = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
        findings.push(format!("Mean fitness: {:.4}", mean_fit));
    }

    // ─── PER-STEP BREAKDOWN ─────────────────────────────────────────

    findings.push("── Per-step breakdown ──".to_string());
    for step in &result.steps {
        let step_collapse_pct = step.collapse_rate * 100.0;
        let step_pop = step.metric_summaries.get("mean_population").map(|s| s.mean).unwrap_or(0.0);
        let step_gini = step.metric_summaries.get("mean_gini_coefficient").map(|s| s.mean).unwrap_or(0.0);
        let step_wci = step.metric_summaries.get("wealth_concentration_index").map(|s| s.mean).unwrap_or(0.0);
        let step_rii = step.metric_summaries.get("reproductive_inequality_index").map(|s| s.mean).unwrap_or(0.0);
        let step_sii = step.metric_summaries.get("survival_inequality_index").map(|s| s.mean).unwrap_or(0.0);
        findings.push(format!(
            "  cap={:.0}: collapse={:.1}%, pop={:.1}, gini={:.4}, wci={:.4}, repro={:.4}, surv={:.4}",
            step.parameter_value, step_collapse_pct, step_pop, step_gini, step_wci, step_rii, step_sii
        ));
    }

    // ─── VERDICT ────────────────────────────────────────────────────

    findings.push("── VERDICT ──".to_string());

    let has_collapse = collapse_count > 0;

    if has_collapse {
        if collapse_rate > 90.0 {
            findings.push(format!(
                "★ TOTAL EXTINCTION — {:.1}% collapse under topology violation [{}]",
                collapse_rate, slug
            ));
            findings.push(
                "The energy loop is the true structural invariant. Removing it destroys all anti-fragility.".to_string()
            );
        } else if collapse_rate > 50.0 {
            findings.push(format!(
                "★ COLLAPSE BOUNDARY FOUND — {:.1}% of worlds collapse under [{}]",
                collapse_rate, slug
            ));
        } else {
            findings.push(format!(
                "★ PARTIAL COLLAPSE — {:.1}% of worlds collapse under [{}]",
                collapse_rate, slug
            ));
        }

        // Collapse mechanism identification
        if mean_rii > 0.5 && mean_wci > 0.4 {
            findings.push(
                "Mechanism: RESOURCE STARVATION → REPRODUCTIVE FAILURE → EXTINCTION".to_string()
            );
        } else if mean_sii > 0.5 {
            findings.push(
                "Mechanism: SURVIVAL APARTHEID → POPULATION ATTRITION".to_string()
            );
        } else {
            findings.push(
                "Mechanism: THERMODYNAMIC DEATH — resource exhaustion drives direct extinction".to_string()
            );
        }
    } else {
        // Classify sub-collapse pathology
        let mut pathologies = Vec::new();
        if mean_wci > 0.4 {
            pathologies.push(format!("wealth_concentration={:.3}", mean_wci));
        }
        if mean_mgini > 0.5 {
            pathologies.push(format!("gini={:.3}", mean_mgini));
        }
        if mean_rii > 0.4 {
            pathologies.push(format!("repro_inequality={:.3}", mean_rii));
        }
        if mean_sii > 0.4 {
            pathologies.push(format!("survival_inequality={:.3}", mean_sii));
        }
        if mean_tdp > 0.7 {
            pathologies.push(format!("wealth_persistence={:.3}", mean_tdp));
        }

        if pathologies.is_empty() {
            findings.push(format!(
                "NO COLLAPSE, NO PATHOLOGY — topology violation [{}] has no detectable effect",
                slug
            ));
            findings.push(
                "Anti-fragility mechanism is deeper than energy topology — architecture-level property.".to_string()
            );
        } else {
            findings.push(format!(
                "NO COLLAPSE but PATHOLOGICAL: [{}] → [{}]",
                slug, pathologies.join(", ")
            ));
            findings.push(
                "Survivable but degenerate economy under topology violation".to_string()
            );
        }
    }

    findings
}
