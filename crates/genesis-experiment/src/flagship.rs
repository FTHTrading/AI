// Flagship Experiments — Pre-built research protocols
//
// Three experiments that demonstrate what Genesis can discover:
//
// 1. Entropy Sweep — "Does the cost of existing determine civilization fate?"
//    Varies entropy_coeff across an order of magnitude.
//    Measures collapse rate, inequality trajectory, survival time.
//
// 2. Catastrophe Resilience — "Can civilizations evolve to survive catastrophe?"
//    Varies catastrophe_base_prob from mild to extreme.
//    Measures survival rate, population recovery, immune response.
//
// 3. Inequality Threshold — "At what Gini threshold does wealth tax stabilize societies?"
//    Varies gini_wealth_tax_threshold from always-active to never-triggers.
//    Measures Gini coefficient, population stability, treasury health.
//
// Each is designed to produce publishable results with statistical rigor.

use crate::config::{ExperimentConfig, ParameterSweep, Metric, SweepVariable};
use genesis_multiverse::PhysicsPreset;
use gateway::world::PressureConfig;
use gateway::stress::StressConfig;

/// Factory for the three flagship experiments.
pub struct FlagshipExperiments;

impl FlagshipExperiments {
    /// Experiment 1: Entropy Sweep
    ///
    /// Hypothesis: Higher entropy coefficients cause earlier civilization collapse
    /// and higher inequality before collapse.
    ///
    /// Sweeps entropy_coeff from 0.00001 (gentle) to 0.0001 (harsh).
    /// 10 steps Ã-- 20 runs Ã-- 500 epochs = 100 worlds, 50,000 total epochs.
    pub fn entropy_sweep() -> ExperimentConfig {
        ExperimentConfig {
            name: "Entropy Sweep: Cost of Existing".into(),
            hypothesis: "Higher entropy coefficients cause earlier civilization collapse \
                         and higher terminal inequality".into(),
            sweep: ParameterSweep::new(
                SweepVariable::EntropyCoeff,
                0.00001,
                0.0001,
                0.00001,
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::GiniCoefficient,
                Metric::MeanFitness,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::TotalEntropyBurned,
                Metric::TotalPressureMutations,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: None,
            base_seed: 20260222, // Date seed: Feb 22, 2026
        }
    }

    /// Experiment 2: Catastrophe Resilience
    ///
    /// Hypothesis: Moderate catastrophe rates build resilience;
    /// extreme rates cause extinction; zero rates cause stagnation.
    ///
    /// Sweeps catastrophe_base_prob from 0.0 (peaceful) to 0.03 (apocalyptic).
    /// 7 steps Ã-- 20 runs Ã-- 500 epochs = 140 worlds, 70,000 total epochs.
    pub fn catastrophe_resilience() -> ExperimentConfig {
        ExperimentConfig {
            name: "Catastrophe Resilience: Survival Under Fire".into(),
            hypothesis: "Moderate catastrophe rates (0.005-0.01) produce more resilient \
                         civilizations than either peaceful (0.0) or apocalyptic (0.03) conditions".into(),
            sweep: ParameterSweep::new(
                SweepVariable::CatastropheBaseProb,
                0.0,
                0.03,
                0.005,
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::TotalCatastropheDeaths,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::TotalImmuneThreats,
                Metric::TotalPressureMutations,
                Metric::MeanFitness,
                Metric::BirthDeathRatio,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: None,
            base_seed: 20260222,
        }
    }

    /// Experiment 3: Inequality Threshold
    ///
    /// Hypothesis: There exists an optimal Gini threshold for wealth tax activation
    /// that maximizes both population stability and economic efficiency.
    ///
    /// Sweeps gini_wealth_tax_threshold from 0.20 (aggressive redistribution)
    /// to 0.90 (laissez-faire).
    /// 8 steps Ã-- 20 runs Ã-- 500 epochs = 160 worlds, 80,000 total epochs.
    pub fn inequality_threshold() -> ExperimentConfig {
        ExperimentConfig {
            name: "Inequality Threshold: When Does Redistribution Help?".into(),
            hypothesis: "A Gini wealth tax threshold between 0.35-0.50 optimizes both \
                         population stability and mean fitness, while extremes (too aggressive \
                         or too passive) reduce long-term outcomes".into(),
            sweep: ParameterSweep::new(
                SweepVariable::GiniWealthTaxThreshold,
                0.20,
                0.90,
                0.10,
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::GiniCoefficient,
                Metric::MeanFitness,
                Metric::TreasuryRatio,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::RoleEntropy,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: None,
            base_seed: 20260222,
        }
    }

    /// Experiment 4: Treasury Stability
    ///
    /// Hypothesis: There exists an optimal treasury overflow threshold that
    /// maximizes economic stability. Too-aggressive redistribution (low threshold)
    /// depletes reserves; too-passive hoarding (high threshold) concentrates wealth.
    ///
    /// Sweeps treasury_overflow_threshold from 0.10 (deploy early) to 0.90 (hoard).
    /// 9 steps Ã-- 20 runs Ã-- 500 epochs = 180 worlds, 90,000 total epochs.
    pub fn treasury_stability() -> ExperimentConfig {
        ExperimentConfig {
            name: "Treasury Stability: Reserve Deployment Policy".into(),
            hypothesis: "An intermediate treasury overflow threshold (0.30-0.50) maximizes \
                         population stability and minimizes inequality, while extremes — \
                         aggressive deployment or passive hoarding — degrade outcomes".into(),
            sweep: ParameterSweep::new(
                SweepVariable::TreasuryOverflowThreshold,
                0.10,
                0.90,
                0.10,
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::TreasuryRatio,
                Metric::GiniCoefficient,
                Metric::MeanFitness,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::BirthDeathRatio,
                Metric::TotalEntropyBurned,
                Metric::RoleEntropy,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: None,
            base_seed: 20260222,
        }
    }

    // â”€â”€â”€ Reserve Stress Suite â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    //
    // 4-tier shock experiment: How should reserve deployment policy
    // change when external shock frequency increases?
    //
    // Each tier sets a different baseline catastrophe_base_prob
    // (representing shock frequency) while sweeping
    // treasury_overflow_threshold (reserve deployment policy).
    //
    // Domain mapping:
    //   Reserve deployment    â†’ treasury overflow threshold
    //   Shock frequency       â†’ catastrophe_base_prob
    //   Economic stress       â†’ entropy taxation
    //   Wealth concentration  â†’ Gini threshold
    //
    // Answers: "Should reserve deployment policy change when
    // external conditions deteriorate?"

    /// Reserve Stress — Calm (shock = 0.001)
    ///
    /// Baseline: Low shock frequency, representing stable conditions.
    /// Sweeps treasury_overflow_threshold 0.10 â†’ 0.90.
    /// 9 steps Ã-- 15 runs Ã-- 500 epochs = 135 worlds, 67,500 epochs.
    pub fn reserve_calm() -> ExperimentConfig {
        Self::reserve_tier("Calm", 0.001,
            "Under calm conditions (shock=0.001), aggressive treasury \
             deployment outperforms hoarding with minimal downside risk")
    }

    /// Reserve Stress — Moderate (shock = 0.005)
    ///
    /// Baseline: Moderate shock frequency, representing normal volatility.
    /// Sweeps treasury_overflow_threshold 0.10 â†’ 0.90.
    /// 9 steps Ã-- 15 runs Ã-- 500 epochs = 135 worlds, 67,500 epochs.
    pub fn reserve_moderate() -> ExperimentConfig {
        Self::reserve_tier("Moderate", 0.005,
            "Under moderate shocks (shock=0.005), the optimal deployment \
             threshold shifts toward conservative reserves")
    }

    /// Reserve Stress — Stressed (shock = 0.015)
    ///
    /// Baseline: High shock frequency, representing environmental stress.
    /// Sweeps treasury_overflow_threshold 0.10 â†’ 0.90.
    /// 9 steps Ã-- 15 runs Ã-- 500 epochs = 135 worlds, 67,500 epochs.
    pub fn reserve_stressed() -> ExperimentConfig {
        Self::reserve_tier("Stressed", 0.015,
            "Under stressed conditions (shock=0.015), reserve hoarding begins \
             to outperform deployment as shock recovery demands liquidity buffers")
    }

    /// Reserve Stress — Crisis (shock = 0.030)
    ///
    /// Baseline: Extreme shock frequency, representing crisis conditions.
    /// Sweeps treasury_overflow_threshold 0.10 â†’ 0.90.
    /// 9 steps Ã-- 15 runs Ã-- 500 epochs = 135 worlds, 67,500 epochs.
    pub fn reserve_crisis() -> ExperimentConfig {
        Self::reserve_tier("Crisis", 0.030,
            "Under crisis conditions (shock=0.030), conservative reserve \
             management becomes critical for survival — deployment policy \
             must shift dramatically or civilizations collapse")
    }

    /// Returns all 4 reserve stress tier experiments.
    pub fn reserve_stress_suite() -> Vec<(&'static str, ExperimentConfig)> {
        vec![
            ("reserve_calm", Self::reserve_calm()),
            ("reserve_moderate", Self::reserve_moderate()),
            ("reserve_stressed", Self::reserve_stressed()),
            ("reserve_crisis", Self::reserve_crisis()),
        ]
    }

    /// Internal: build a single reserve stress tier experiment.
    fn reserve_tier(tier_label: &str, shock_prob: f64, hypothesis: &str) -> ExperimentConfig {
        let mut base_pressure = PressureConfig::default();
        base_pressure.catastrophe_base_prob = shock_prob;

        ExperimentConfig {
            name: format!("Reserve Stress — {} (shock={:.3})", tier_label, shock_prob),
            hypothesis: hypothesis.into(),
            sweep: ParameterSweep::new(
                SweepVariable::TreasuryOverflowThreshold,
                0.10,
                0.90,
                0.10,
            ),
            runs_per_step: 15,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::TreasuryRatio,
                Metric::GiniCoefficient,
                Metric::MeanFitness,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::BirthDeathRatio,
                Metric::TotalCatastropheDeaths,
                Metric::TotalEntropyBurned,
                Metric::RoleEntropy,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(base_pressure),
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: None,
            base_seed: 20260222,
        }
    }

    // â”€â”€â”€ Resource Depletion Crossover â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    //
    // 4-tier carrying capacity experiment: How does metabolic cost
    // sensitivity change when the population's carrying capacity is
    // compressed?
    //
    // Each tier sets a different baseline soft_cap (carrying capacity)
    // while sweeping entropy_coeff (metabolic cost of existence).
    //
    // Domain mapping:
    //   Carrying capacity  â†’ soft_cap
    //   Metabolic cost     â†’ entropy_coeff
    //   Resource scarcity  â†’ lower soft_cap = fewer sustainable agents
    //
    // Answers: "Does the relationship between metabolic cost and
    // population health change under resource scarcity?"

    /// Resource Depletion — Abundant (soft_cap = 200)
    ///
    /// Baseline: High carrying capacity, representing resource abundance.
    /// Sweeps entropy_coeff 0.00001 â†’ 0.00010, step 0.00001.
    /// 10 steps Ã-- 15 runs Ã-- 500 epochs = 150 worlds, 75,000 epochs.
    pub fn resource_depletion_abundant() -> ExperimentConfig {
        Self::resource_depletion_tier("Abundant", 200, 
            "Under abundant resources (soft_cap=200), increasing metabolic \
             cost has minimal impact on population stability")
    }

    /// Resource Depletion — Normal (soft_cap = 120)
    ///
    /// Baseline: Normal carrying capacity, representing typical conditions.
    /// Sweeps entropy_coeff 0.00001 â†’ 0.00010, step 0.00001.
    /// 10 steps Ã-- 15 runs Ã-- 500 epochs = 150 worlds, 75,000 epochs.
    pub fn resource_depletion_normal() -> ExperimentConfig {
        Self::resource_depletion_tier("Normal", 120,
            "Under normal capacity (soft_cap=120), metabolic cost increases \
             produce measurable but manageable population stress")
    }

    /// Resource Depletion — Constrained (soft_cap = 60)
    ///
    /// Baseline: Low carrying capacity, representing resource constraints.
    /// Sweeps entropy_coeff 0.00001 â†’ 0.00010, step 0.00001.
    /// 10 steps Ã-- 15 runs Ã-- 500 epochs = 150 worlds, 75,000 epochs.
    pub fn resource_depletion_constrained() -> ExperimentConfig {
        Self::resource_depletion_tier("Constrained", 60,
            "Under constrained resources (soft_cap=60), metabolic cost \
             becomes a critical survival factor — small increases may \
             trigger population collapse")
    }

    /// Resource Depletion — Scarce (soft_cap = 30)
    ///
    /// Baseline: Minimal carrying capacity, representing extreme scarcity.
    /// Sweeps entropy_coeff 0.00001 â†’ 0.00010, step 0.00001.
    /// 10 steps Ã-- 15 runs Ã-- 500 epochs = 150 worlds, 75,000 epochs.
    pub fn resource_depletion_scarce() -> ExperimentConfig {
        Self::resource_depletion_tier("Scarce", 30,
            "Under extreme scarcity (soft_cap=30), even moderate metabolic \
             cost increases become existential — population collapse \
             thresholds shift dramatically")
    }

    /// Returns all 4 resource depletion tier experiments.
    pub fn resource_depletion_suite() -> Vec<(&'static str, ExperimentConfig)> {
        vec![
            ("resource_depletion_abundant", Self::resource_depletion_abundant()),
            ("resource_depletion_normal", Self::resource_depletion_normal()),
            ("resource_depletion_constrained", Self::resource_depletion_constrained()),
            ("resource_depletion_scarce", Self::resource_depletion_scarce()),
        ]
    }

    /// Internal: build a single resource depletion tier experiment.
    fn resource_depletion_tier(tier_label: &str, soft_cap: usize, hypothesis: &str) -> ExperimentConfig {
        let mut base_pressure = PressureConfig::default();
        base_pressure.soft_cap = soft_cap;

        ExperimentConfig {
            name: format!("Resource Depletion — {} (cap={})", tier_label, soft_cap),
            hypothesis: hypothesis.into(),
            sweep: ParameterSweep::new(
                SweepVariable::EntropyCoeff,
                0.00001,
                0.00010,
                0.00001,
            ),
            runs_per_step: 15,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::TreasuryRatio,
                Metric::GiniCoefficient,
                Metric::MeanFitness,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::BirthDeathRatio,
                Metric::TotalCatastropheDeaths,
                Metric::TotalEntropyBurned,
                Metric::RoleEntropy,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(base_pressure),
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: None,
            base_seed: 20260222,
        }
    }

    // â”€â”€â”€ Evolution Forbidden â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    //
    // Control experiment: What happens when mutation is completely disabled?
    //
    // Identical to catastrophe_resilience EXCEPT mutation_base_rate is
    // forced to 0.0. Agents cannot adapt their traits. If the organism
    // survives catastrophe only because mutation lets low-fitness agents
    // explore new trait configurations, then forbidding evolution should
    // reveal the dependency.
    //
    // The question: Is Genesis Protocol's resilience due to adaptation,
    // or is the economic structure alone sufficient?
    //
    // Domain mapping:
    //   Catastrophe frequency â†’ catastrophe_base_prob (swept)
    //   Adaptation            â†’ mutation_base_rate (locked at 0.0)
    //
    // Answers: "Can a civilization survive without evolution?"

    /// Evolution Forbidden: Catastrophe Resilience Without Adaptation
    ///
    /// Identical catastrophe sweep as `catastrophe_resilience()` but with
    /// mutation_base_rate locked at 0.0. No trait mutation occurs.
    /// 7 steps Ã-- 20 runs Ã-- 500 epochs = 140 worlds, 70,000 total epochs.
    pub fn evolution_forbidden() -> ExperimentConfig {
        ExperimentConfig {
            name: "Evolution Forbidden: Survival Without Adaptation".into(),
            hypothesis: "Without mutation (base_rate=0.0), civilizations lose resilience \
                         under catastrophe pressure — collapse rates increase relative to \
                         the standard catastrophe resilience experiment, proving that \
                         adaptation is a necessary survival mechanism".into(),
            sweep: ParameterSweep::new(
                SweepVariable::CatastropheBaseProb,
                0.0,
                0.03,
                0.005,
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::TotalCatastropheDeaths,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::TotalImmuneThreats,
                Metric::TotalPressureMutations,
                Metric::TotalTraitMutations,
                Metric::MeanFitness,
                Metric::BirthDeathRatio,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: Some(0.0), // THE KEY: no mutation allowed
            cortex_enabled_override: None,
            base_stress_override: None,
            base_seed: 20260222,
        }
    }

    // â”€â”€â”€ Resilience Matrix â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    //
    // Week 3: 4-quadrant grid testing all combinations of two adaptation layers:
    //   Q1: Agent ON,  Cortex ON  (baseline — full adaptation)
    //   Q2: Agent OFF, Cortex ON  (immune only — cortex compensates)
    //   Q3: Agent ON,  Cortex OFF (genetic only — traits compensate)
    //   Q4: Agent OFF, Cortex OFF (true kill test — fully static)
    //
    // Thesis: Resilience emerges from layered adaptive redundancy.
    // Goal: Find the first collapse.

    /// Q1: Both adaptation layers active (baseline control).
    pub fn resilience_q1_both() -> ExperimentConfig {
        Self::resilience_quadrant(
            "Resilience Matrix Q1: Full Adaptation",
            "With both agent-level mutation and cortex immune adaptation active, \
             civilizations maintain maximum resilience across all catastrophe levels — \
             this is the baseline against which partial and total disablement are measured",
            None,        // mutation_rate: default (ON)
            None,        // cortex: default (ON)
        )
    }

    /// Q2: Agent-level mutation OFF, cortex immune ON.
    /// (This is evolution_forbidden but with wider sweep to 0.05.)
    pub fn resilience_q2_immune_only() -> ExperimentConfig {
        Self::resilience_quadrant(
            "Resilience Matrix Q2: Immune Only",
            "With agent-level trait mutation frozen but cortex immune adaptation active, \
             civilizations rely solely on environmental pressure tuning — the cortex \
             compensates for frozen genetics, but may fail at extreme catastrophe rates",
            Some(0.0),   // mutation_rate: OFF
            None,        // cortex: default (ON)
        )
    }

    /// Q3: Agent-level mutation ON, cortex immune OFF.
    pub fn resilience_q3_genetic_only() -> ExperimentConfig {
        Self::resilience_quadrant(
            "Resilience Matrix Q3: Genetic Only",
            "With cortex immune adaptation disabled but agent-level trait mutation active, \
             civilizations rely solely on genetic adaptation — without environmental \
             pressure tuning, raw selection pressure determines survival",
            None,        // mutation_rate: default (ON)
            Some(false), // cortex: OFF
        )
    }

    /// Q4: Both adaptation layers OFF — the true kill test.
    pub fn resilience_q4_static() -> ExperimentConfig {
        Self::resilience_quadrant(
            "Resilience Matrix Q4: Fully Static",
            "With BOTH adaptation layers disabled — no trait mutation, no cortex immune \
             adaptation — civilizations are completely static. This is the true kill test: \
             if collapse occurs here but not in Q1-Q3, it proves layered adaptive redundancy \
             is the stabilizing mechanism",
            Some(0.0),   // mutation_rate: OFF
            Some(false), // cortex: OFF
        )
    }

    /// Internal builder for resilience quadrant configs.
    fn resilience_quadrant(
        name: &str,
        hypothesis: &str,
        mutation_rate: Option<f64>,
        cortex_enabled: Option<bool>,
    ) -> ExperimentConfig {
        ExperimentConfig {
            name: name.into(),
            hypothesis: hypothesis.into(),
            sweep: ParameterSweep::new(
                SweepVariable::CatastropheBaseProb,
                0.0,   // peaceful
                0.05,  // wider than Week 2's 0.03 — push to breaking
                0.005, // 11 steps
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::TotalCatastropheDeaths,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::TotalImmuneThreats,
                Metric::TotalPressureMutations,
                Metric::TotalTraitMutations,
                Metric::MeanFitness,
                Metric::BirthDeathRatio,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::TotalEntropyBurned,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: mutation_rate,
            cortex_enabled_override: cortex_enabled,
            base_stress_override: None,
            base_seed: 20260223, // Week 3 seed
        }
    }

    /// Returns all 4 quadrants as a suite.
    pub fn resilience_matrix_suite() -> Vec<(&'static str, ExperimentConfig)> {
        vec![
            ("resilience_q1_both", Self::resilience_q1_both()),
            ("resilience_q2_immune_only", Self::resilience_q2_immune_only()),
            ("resilience_q3_genetic_only", Self::resilience_q3_genetic_only()),
            ("resilience_q4_static", Self::resilience_q4_static()),
        ]
    }

    /// Quick versions for testing.
    pub fn resilience_q1_both_quick() -> ExperimentConfig {
        let mut config = Self::resilience_q1_both();
        config.name = "Resilience Q1 Both (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn resilience_q4_static_quick() -> ExperimentConfig {
        let mut config = Self::resilience_q4_static();
        config.name = "Resilience Q4 Static (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    /// Quick versions of flagships for testing (fewer runs, fewer epochs).
    pub fn entropy_sweep_quick() -> ExperimentConfig {
        let mut config = Self::entropy_sweep();
        config.name = "Entropy Sweep (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn catastrophe_resilience_quick() -> ExperimentConfig {
        let mut config = Self::catastrophe_resilience();
        config.name = "Catastrophe Resilience (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn inequality_threshold_quick() -> ExperimentConfig {
        let mut config = Self::inequality_threshold();
        config.name = "Inequality Threshold (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn treasury_stability_quick() -> ExperimentConfig {
        let mut config = Self::treasury_stability();
        config.name = "Treasury Stability (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn reserve_calm_quick() -> ExperimentConfig {
        let mut config = Self::reserve_calm();
        config.name = "Reserve Calm (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn reserve_moderate_quick() -> ExperimentConfig {
        let mut config = Self::reserve_moderate();
        config.name = "Reserve Moderate (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn reserve_stressed_quick() -> ExperimentConfig {
        let mut config = Self::reserve_stressed();
        config.name = "Reserve Stressed (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn reserve_crisis_quick() -> ExperimentConfig {
        let mut config = Self::reserve_crisis();
        config.name = "Reserve Crisis (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn resource_depletion_abundant_quick() -> ExperimentConfig {
        let mut config = Self::resource_depletion_abundant();
        config.name = "Resource Depletion Abundant (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn resource_depletion_normal_quick() -> ExperimentConfig {
        let mut config = Self::resource_depletion_normal();
        config.name = "Resource Depletion Normal (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn resource_depletion_constrained_quick() -> ExperimentConfig {
        let mut config = Self::resource_depletion_constrained();
        config.name = "Resource Depletion Constrained (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn resource_depletion_scarce_quick() -> ExperimentConfig {
        let mut config = Self::resource_depletion_scarce();
        config.name = "Resource Depletion Scarce (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    pub fn evolution_forbidden_quick() -> ExperimentConfig {
        let mut config = Self::evolution_forbidden();
        config.name = "Evolution Forbidden (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }


    // --- Multi-Axis Collapse Vector -------------------------------------
    //
    // The boundary search experiment. Every survival mechanism stripped
    // simultaneously while sweeping carrying capacity downward.
    //
    // Axes locked hostile:
    //   Mutation:       mutation_rate_override = 0.0 (no adaptation)
    //   Immune system:  cortex_enabled_override = false (no self-regulation)
    //   Redistribution: gini_wealth_tax_threshold = 1.0 (never triggers)
    //                   gini_wealth_tax_rate = 0.0 (zero tax)
    //   Treasury:       treasury_overflow_threshold = 1.0 (never deploys)
    //   Catastrophe:    catastrophe_base_prob = 0.03 (maximum tested)
    //   Entropy:        entropy_coeff = 0.0001 (10x default, harshest)
    //
    // Sweep: soft_cap from 30 to 180 (step 15) -- 11 capacity levels.
    //
    // This is not a parameter study. It is a search for the attractor
    // boundary. If the system collapses at any capacity level, we have
    // found the first boundary condition in Genesis Protocol. If it
    // survives at all levels, structural immunity is proven under
    // correlated multi-axis stress.
    //
    // Domain mapping:
    //   Carrying capacity  ? soft_cap (swept)
    //   All protections    ? disabled or maximally hostile
    //
    // Answers: "Does the attractor have a boundary?"

    /// Multi-Axis Collapse Vector: Attractor Boundary Search
    ///
    /// Every survival mechanism disabled. Maximum catastrophe + entropy.
    /// Sweeps soft_cap from 30 (extreme scarcity) to 180 (normal).
    /// 11 steps x 20 runs x 500 epochs = 220 worlds, 110,000 total epochs.
    pub fn multi_axis_collapse() -> ExperimentConfig {
        let mut hostile_pressure = PressureConfig::default();
        // Maximum catastrophe -- one shock every ~33 epochs
        hostile_pressure.catastrophe_base_prob = 0.03;
        // Harshest entropy -- 10x default
        hostile_pressure.entropy_coeff = 0.0001;
        // Disable wealth redistribution completely
        hostile_pressure.gini_wealth_tax_threshold = 1.0; // Gini can never reach 1.0
        hostile_pressure.gini_wealth_tax_rate = 0.0;       // Zero tax even if triggered
        // Treasury never deploys reserves
        hostile_pressure.treasury_overflow_threshold = 1.0;

        ExperimentConfig {
            name: "Multi-Axis Collapse Vector: Attractor Boundary Search".into(),
            hypothesis: "Under simultaneous maximal stress -- no mutation, no immune system, \
                         no redistribution, no treasury deployment, maximum catastrophe, \
                         maximum entropy -- carrying capacity compression will reveal \
                         the attractor boundary. If collapse occurs at any soft_cap level, \
                         the first boundary condition in Genesis Protocol is established. \
                         If the system survives at all levels, structural immunity is proven \
                         under correlated multi-axis stress.".into(),
            sweep: ParameterSweep::new(
                SweepVariable::SoftCap,
                30.0,   // extreme scarcity
                180.0,  // normal capacity
                15.0,   // 11 steps
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::MeanFitness,
                Metric::MaxFitness,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::BirthDeathRatio,
                Metric::TotalCatastropheDeaths,
                Metric::TotalEntropyBurned,
                Metric::TreasuryRatio,
                Metric::RoleEntropy,
                Metric::TotalTraitMutations,
                Metric::TotalPressureMutations,
                Metric::TotalImmuneThreats,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(hostile_pressure),
            mutation_rate_override: Some(0.0),        // No adaptation
            cortex_enabled_override: Some(false),     // No immune self-regulation
            base_stress_override: None,
            base_seed: 20260222,
        }
    }

    pub fn multi_axis_collapse_quick() -> ExperimentConfig {
        let mut config = Self::multi_axis_collapse();
        config.name = "Multi-Axis Collapse Vector (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    // =====================================================================
    // METABOLIC INVERSION -- The Oxygen Attack
    //
    // Multi-axis collapse attacked weather (catastrophe frequency, entropy,
    // redistribution suppression). Population converged to ~46 regardless.
    //
    // The real discovery was metabolic equilibrium: birth = death at a
    // fixed attractor point. Environmental hostility cannot break this
    // because death rate never exceeds birth rate sustainably.
    //
    // Metabolic inversion attacks oxygen -- the cost of reproduction itself.
    // If effective_replication_cost exceeds what agents can accumulate,
    // demographic replacement fails and the population collapses.
    //
    // Sweeps replication_cost_multiplier from 1.0x (25 ATP) to 5.0x (125 ATP)
    // under the same hostile conditions as multi-axis collapse.
    // =====================================================================

    /// Metabolic Inversion: break demographic replacement by increasing
    /// the cost of reproduction under full hostile conditions.
    ///
    /// Uses StressConfig.replication_cost_multiplier to scale the ATP cost
    /// of replication from 1.0x (normal, 25 ATP) to 5.0x (125 ATP).
    ///
    /// All hostile axes from multi-axis collapse retained:
    /// - No mutation, no immune cortex, no redistribution, no treasury
    /// - Maximum catastrophe (0.03), maximum entropy (0.0001, 10x default)
    pub fn metabolic_inversion() -> ExperimentConfig {
        let mut hostile_pressure = PressureConfig::default();
        // Retain ALL hostile axes from multi-axis collapse
        hostile_pressure.catastrophe_base_prob = 0.03;
        hostile_pressure.entropy_coeff = 0.0001;
        hostile_pressure.gini_wealth_tax_threshold = 1.0;
        hostile_pressure.gini_wealth_tax_rate = 0.0;
        hostile_pressure.treasury_overflow_threshold = 1.0;

        ExperimentConfig {
            name: "Metabolic Inversion: The Oxygen Attack".into(),
            hypothesis: "Environmental hostility failed to induce collapse because it attacks \
                         weather, not oxygen. The emergent population constant (~46) reflects \
                         metabolic equilibrium where birth = death. Metabolic inversion breaks \
                         this by making reproduction prohibitively expensive. At some \
                         replication_cost_multiplier, agents cannot accumulate enough ATP to \
                         reproduce, demographic replacement fails, and the population collapses. \
                         This experiment maps the metabolic boundary of the attractor.".into(),
            sweep: ParameterSweep::new(
                SweepVariable::ReplicationCostMultiplier,
                1.0,    // normal: 25 ATP per replication
                5.0,    // extreme: 125 ATP per replication
                0.5,    // 9 steps: 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::MeanFitness,
                Metric::MaxFitness,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::BirthDeathRatio,
                Metric::TotalCatastropheDeaths,
                Metric::TotalEntropyBurned,
                Metric::TreasuryRatio,
                Metric::RoleEntropy,
                Metric::TotalTraitMutations,
                Metric::TotalPressureMutations,
                Metric::TotalImmuneThreats,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(hostile_pressure),
            mutation_rate_override: Some(0.0),        // No adaptation
            cortex_enabled_override: Some(false),     // No immune self-regulation
            base_stress_override: None,               // StressConfig built from sweep variable
            base_seed: 20260223,                      // New seed for this experiment
        }
    }

    pub fn metabolic_inversion_quick() -> ExperimentConfig {
        let mut config = Self::metabolic_inversion();
        config.name = "Metabolic Inversion (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    // =====================================================================
    // BASAL INVERSION -- The Starvation
    //
    // Metabolic inversion attacks reproduction (replication cost).
    // Basal inversion attacks existence (cost of living each epoch).
    //
    // Sweeps basal_cost_multiplier from 1.0x (0.15 ATP) to 10.0x (1.5 ATP)
    // under the same hostile conditions as multi-axis collapse.
    //
    // At high multipliers, basal burn exceeds per-epoch income, causing
    // chronic ATP depletion even in survivors.
    // =====================================================================

    /// Basal Inversion: break survival by increasing the cost of existing
    /// under full hostile conditions.
    pub fn basal_inversion() -> ExperimentConfig {
        let mut hostile_pressure = PressureConfig::default();
        hostile_pressure.catastrophe_base_prob = 0.03;
        hostile_pressure.entropy_coeff = 0.0001;
        hostile_pressure.gini_wealth_tax_threshold = 1.0;
        hostile_pressure.gini_wealth_tax_rate = 0.0;
        hostile_pressure.treasury_overflow_threshold = 1.0;

        ExperimentConfig {
            name: "Basal Inversion: The Starvation".into(),
            hypothesis: "The Oxygen Attack targets reproduction. The Starvation targets \
                         existence itself. By scaling basal metabolic cost from 1x (0.15 ATP) \
                         to 10x (1.5 ATP), agents burn through ATP faster than they can earn. \
                         At some multiplier, chronic energy deficit causes population decay \
                         and eventual collapse.".into(),
            sweep: ParameterSweep::new(
                SweepVariable::BasalCostMultiplier,
                1.0,    // normal: 0.15 ATP per epoch
                10.0,   // extreme: 1.5 ATP per epoch
                1.0,    // 10 steps: 1,2,3,...,10
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::MeanFitness,
                Metric::MaxFitness,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::BirthDeathRatio,
                Metric::TotalCatastropheDeaths,
                Metric::TotalEntropyBurned,
                Metric::TreasuryRatio,
                Metric::RoleEntropy,
                Metric::TotalTraitMutations,
                Metric::TotalPressureMutations,
                Metric::TotalImmuneThreats,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(hostile_pressure),
            mutation_rate_override: Some(0.0),
            cortex_enabled_override: Some(false),
            base_stress_override: None,
            base_seed: 20260224, // Week 4 seed
        }
    }

    pub fn basal_inversion_quick() -> ExperimentConfig {
        let mut config = Self::basal_inversion();
        config.name = "Basal Inversion (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    // =====================================================================
    // DUAL INVERSION -- The Final Escalation
    //
    // Attack BOTH metabolic axes simultaneously.
    // Replication cost fixed at 3x (75 ATP) while sweeping basal cost 1-10x.
    //
    // If either single-axis inversion finds collapse, the dual should
    // find it sooner. If neither finds collapse, the dual is the last
    // chance to break the organism.
    // =====================================================================

    /// Dual Inversion: attack both reproduction and existence simultaneously.
    pub fn dual_inversion() -> ExperimentConfig {
        let mut hostile_pressure = PressureConfig::default();
        hostile_pressure.catastrophe_base_prob = 0.03;
        hostile_pressure.entropy_coeff = 0.0001;
        hostile_pressure.gini_wealth_tax_threshold = 1.0;
        hostile_pressure.gini_wealth_tax_rate = 0.0;
        hostile_pressure.treasury_overflow_threshold = 1.0;

        // Fix replication cost at 3x while sweeping basal cost
        let base_stress = StressConfig {
            replication_cost_multiplier: 3.0, // 75 ATP per birth (fixed)
            ..StressConfig::default()
        };

        ExperimentConfig {
            name: "Dual Inversion: The Final Escalation".into(),
            hypothesis: "Single-axis metabolic attacks may fail because the other metabolic \
                         pathway compensates. By fixing replication cost at 3x (75 ATP) \
                         while sweeping basal cost from 1x to 10x, both metabolic pathways \
                         are simultaneously stressed. If the organism survives this, nothing \
                         in the current parameter space can kill it.".into(),
            sweep: ParameterSweep::new(
                SweepVariable::BasalCostMultiplier,
                1.0,
                10.0,
                1.0,    // 10 steps
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::MeanPopulation,
                Metric::PopulationVolatility,
                Metric::MeanFitness,
                Metric::MaxFitness,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::BirthDeathRatio,
                Metric::TotalCatastropheDeaths,
                Metric::TotalEntropyBurned,
                Metric::TreasuryRatio,
                Metric::RoleEntropy,
                Metric::TotalTraitMutations,
                Metric::TotalPressureMutations,
                Metric::TotalImmuneThreats,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(hostile_pressure),
            mutation_rate_override: Some(0.0),
            cortex_enabled_override: Some(false),
            base_stress_override: Some(base_stress), // 3x replication cost baked in
            base_seed: 20260224,
        }
    }

    pub fn dual_inversion_quick() -> ExperimentConfig {
        let mut config = Self::dual_inversion();
        config.name = "Dual Inversion (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 50;
        config
    }

    /// Week 4 Tournament Suite: all three metabolic attacks.
    pub fn tournament_suite() -> Vec<(&'static str, ExperimentConfig)> {
        vec![
            ("metabolic_inversion", Self::metabolic_inversion()),
            ("basal_inversion", Self::basal_inversion()),
            ("dual_inversion", Self::dual_inversion()),
        ]
    }

    // =====================================================================
    // SEASON 2 — STRUCTURAL INVARIANT VIOLATIONS
    //
    // Season 1 swept environmental and metabolic parameters.
    // Season 2 breaks structural invariants — the architectural guarantees
    // that define the system's physics. These are binary violations, not
    // parameter sweeps within the design space.
    //
    // S1: Treasury Cycling Disabled
    //     ATP flows into the treasury (skim, wealth tax, gini tax) but
    //     never comes back out. No stipends, no crisis spending, no
    //     overflow redistribution, no seasonal release.
    //     This replicates the v1.0 failure mode under controlled conditions.
    // =====================================================================

    /// S1 Baseline: Treasury cycling disabled under normal conditions.
    ///
    /// Sweep carrying capacity (soft_cap) from 30 to 180 to test whether
    /// the system can survive without redistribution at any scale.
    /// All adaptation layers remain active — only the recycling loop is broken.
    pub fn s1_treasury_disabled_baseline() -> ExperimentConfig {
        let treasury_disabled = StressConfig {
            treasury_cycling_enabled: false,
            ..StressConfig::default()
        };

        ExperimentConfig {
            name: "S1 Treasury Disabled: Baseline".into(),
            hypothesis: "Treasury cycling (redistribution of collected ATP back to agents) \
                         is a structural invariant of the Genesis Protocol. Without it, \
                         ATP accumulates in the treasury indefinitely — a one-way drain \
                         that starves the circulating economy. Under baseline conditions \
                         with all adaptation layers active, this should produce population \
                         decline and eventual collapse, replicating the v1.0 failure mode \
                         under controlled experimental conditions.".into(),
            sweep: ParameterSweep::new(
                SweepVariable::SoftCap,
                30.0,   // extreme scarcity
                180.0,  // normal capacity
                30.0,   // 6 steps
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::MeanPopulation,
                Metric::MinPopulation,
                Metric::PopulationVolatility,
                Metric::MeanFitness,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::BirthDeathRatio,
                Metric::TreasuryRatio,
                Metric::MaxTreasuryReserve,
                Metric::TotalEntropyBurned,
                Metric::RoleEntropy,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: Some(treasury_disabled),
            base_seed: 20260223,
        }
    }

    pub fn s1_treasury_disabled_baseline_quick() -> ExperimentConfig {
        let mut config = Self::s1_treasury_disabled_baseline();
        config.name = "S1 Treasury Disabled: Baseline (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 100;
        config
    }

    /// S1 Hostile: Treasury cycling disabled under full hostile conditions.
    ///
    /// Same hostile pressure as multi-axis collapse (max catastrophe,
    /// max entropy, no Gini tax, no treasury overflow deployment),
    /// plus structural treasury cycling disabled.
    /// No mutation, no immune cortex.
    ///
    /// If treasury cycling is truly *necessary*, this should collapse
    /// faster than the baseline variant.
    pub fn s1_treasury_disabled_hostile() -> ExperimentConfig {
        let mut hostile_pressure = PressureConfig::default();
        hostile_pressure.catastrophe_base_prob = 0.03;
        hostile_pressure.entropy_coeff = 0.0001;
        hostile_pressure.gini_wealth_tax_threshold = 1.0;
        hostile_pressure.gini_wealth_tax_rate = 0.0;
        hostile_pressure.treasury_overflow_threshold = 1.0;

        let treasury_disabled = StressConfig {
            treasury_cycling_enabled: false,
            ..StressConfig::default()
        };

        ExperimentConfig {
            name: "S1 Treasury Disabled: Hostile".into(),
            hypothesis: "Under full hostile conditions (max catastrophe, max entropy, \
                         no redistribution, no immune cortex, no mutation) AND with \
                         treasury cycling structurally disabled, the system cannot recycle \
                         ATP from the treasury sink. This is the harshest possible S1 test. \
                         If collapse occurs, the time-to-extinction and minimum population \
                         trajectory characterize the phase transition. If survival persists, \
                         treasury cycling may not be independently necessary — the catastrophe \
                         and entropy taxes may be sufficient to prevent total drain.".into(),
            sweep: ParameterSweep::new(
                SweepVariable::SoftCap,
                30.0,
                180.0,
                30.0,   // 6 steps
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::MeanPopulation,
                Metric::MinPopulation,
                Metric::PopulationVolatility,
                Metric::MeanFitness,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::BirthDeathRatio,
                Metric::TreasuryRatio,
                Metric::MaxTreasuryReserve,
                Metric::TotalEntropyBurned,
                Metric::TotalCatastropheDeaths,
                Metric::RoleEntropy,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(hostile_pressure),
            mutation_rate_override: Some(0.0),
            cortex_enabled_override: Some(false),
            base_stress_override: Some(treasury_disabled),
            base_seed: 20260223,
        }
    }

    pub fn s1_treasury_disabled_hostile_quick() -> ExperimentConfig {
        let mut config = Self::s1_treasury_disabled_hostile();
        config.name = "S1 Treasury Disabled: Hostile (Quick)".into();
        config.runs_per_step = 3;
        config.epochs_per_run = 100;
        config
    }

    // =====================================================================
    // Season 2, S2: ATP Decay Disabled
    // =====================================================================

    /// S2 Baseline: ATP decay disabled under normal conditions.
    ///
    /// The 2% per-epoch ATP decay prevents infinite wealth accumulation.
    /// Without it, agent balances become immortal — wealth never erodes.
    /// Predicted cascade: wealth concentration → reproductive monopoly
    /// → demographic stagnation → eventual population collapse.
    pub fn s2_atp_decay_disabled_baseline() -> ExperimentConfig {
        let decay_disabled = StressConfig {
            atp_decay_enabled: false,
            ..StressConfig::default()
        };

        ExperimentConfig {
            name: "S2 ATP Decay Disabled: Baseline".into(),
            hypothesis: "ATP decay (2% per epoch) is a structural invariant that prevents \
                         infinite wealth accumulation. Without it, wealthy agents retain \
                         their balances indefinitely — creating permanent dynasties that \
                         monopolize reproduction (25 ATP cost) while poorer agents cannot \
                         afford to replicate. Under baseline conditions with all other \
                         adaptation layers active, this could produce demographic stagnation \
                         (falling births as wealth locks in) or runaway inequality \
                         (Gini → 1.0). If the wealth tax (1% on >100 ATP) and Gini tax \
                         compensate, the system may survive — but inequality metrics \
                         should diverge from baseline.".into(),
            sweep: ParameterSweep::new(
                SweepVariable::SoftCap,
                30.0,   // extreme scarcity
                180.0,  // normal capacity
                30.0,   // 6 steps
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::MeanPopulation,
                Metric::MinPopulation,
                Metric::MeanFitness,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::TreasuryRatio,
                Metric::MaxTreasuryReserve,
                Metric::BirthDeathRatio,
                // S2 inequality instrumentation
                Metric::AtpVariance,
                Metric::WealthConcentrationIndex,
                Metric::MedianMeanAtpDivergence,
                Metric::MeanGiniCoefficient,
                Metric::MaxGiniCoefficient,
                Metric::ReproductiveInequalityIndex,
                Metric::SurvivalInequalityIndex,
                Metric::TopDecilePersistence,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: Some(decay_disabled),
            base_seed: 42,
        }
    }

    /// S2 Baseline quick variant (2 steps × 5 runs × 100 epochs).
    pub fn s2_atp_decay_disabled_baseline_quick() -> ExperimentConfig {
        let mut config = Self::s2_atp_decay_disabled_baseline();
        config.name = "S2 ATP Decay Disabled: Baseline [QUICK]".into();
        config.sweep = ParameterSweep::new(SweepVariable::SoftCap, 60.0, 120.0, 60.0);
        config.runs_per_step = 5;
        config.epochs_per_run = 100;
        config
    }

    /// S2 Hostile: ATP decay disabled under full adversarial conditions.
    ///
    /// All protective mechanisms stripped: high catastrophe, high entropy,
    /// no wealth tax, no immune cortex, no mutation. With ATP decay also
    /// disabled, there is no erosion force on accumulated wealth.
    pub fn s2_atp_decay_disabled_hostile() -> ExperimentConfig {
        let mut hostile_pressure = PressureConfig::default();
        hostile_pressure.catastrophe_base_prob = 0.03;
        hostile_pressure.entropy_coeff = 0.0001;
        hostile_pressure.gini_wealth_tax_threshold = 1.0;  // effectively disabled
        hostile_pressure.gini_wealth_tax_rate = 0.0;
        hostile_pressure.treasury_overflow_threshold = 1.0;

        let decay_disabled = StressConfig {
            atp_decay_enabled: false,
            ..StressConfig::default()
        };

        ExperimentConfig {
            name: "S2 ATP Decay Disabled: Hostile".into(),
            hypothesis: "Under full hostile conditions (max catastrophe, max entropy, \
                         no Gini tax, no immune cortex, no mutation) AND with ATP decay \
                         structurally disabled, wealth becomes immortal in an already \
                         hostile environment. Without any erosion mechanisms, the oldest \
                         surviving agents accumulate unchecked wealth while new agents \
                         cannot afford to replicate. This is the harshest S2 test. \
                         Expected: rapid wealth concentration → reproductive collapse \
                         → population extinction.".into(),
            sweep: ParameterSweep::new(
                SweepVariable::SoftCap,
                30.0,
                180.0,
                30.0,   // 6 steps
            ),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: vec![
                Metric::FinalPopulation,
                Metric::Collapsed,
                Metric::SurvivalEpochs,
                Metric::MeanPopulation,
                Metric::MinPopulation,
                Metric::MeanFitness,
                Metric::GiniCoefficient,
                Metric::TotalBirths,
                Metric::TotalDeaths,
                Metric::TreasuryRatio,
                Metric::MaxTreasuryReserve,
                Metric::BirthDeathRatio,
                // S2 inequality instrumentation
                Metric::AtpVariance,
                Metric::WealthConcentrationIndex,
                Metric::MedianMeanAtpDivergence,
                Metric::MeanGiniCoefficient,
                Metric::MaxGiniCoefficient,
                Metric::ReproductiveInequalityIndex,
                Metric::SurvivalInequalityIndex,
                Metric::TopDecilePersistence,
            ],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(hostile_pressure),
            mutation_rate_override: Some(0.0),
            cortex_enabled_override: Some(false),
            base_stress_override: Some(decay_disabled),
            base_seed: 42,
        }
    }

    /// S2 Hostile quick variant (2 steps × 5 runs × 100 epochs).
    pub fn s2_atp_decay_disabled_hostile_quick() -> ExperimentConfig {
        let mut config = Self::s2_atp_decay_disabled_hostile();
        config.name = "S2 ATP Decay Disabled: Hostile [QUICK]".into();
        config.sweep = ParameterSweep::new(SweepVariable::SoftCap, 60.0, 120.0, 60.0);
        config.runs_per_step = 5;
        config.epochs_per_run = 100;
        config
    }

    /// Season 2 Invariant Violation Suite: all S1 + S2 experiments.
    pub fn s2_invariant_suite() -> Vec<(&'static str, ExperimentConfig)> {
        vec![
            ("s1_treasury_disabled_baseline", Self::s1_treasury_disabled_baseline()),
            ("s1_treasury_disabled_hostile", Self::s1_treasury_disabled_hostile()),
            ("s2_atp_decay_disabled_baseline", Self::s2_atp_decay_disabled_baseline()),
            ("s2_atp_decay_disabled_hostile", Self::s2_atp_decay_disabled_hostile()),
        ]
    }

    /// List all flagship experiment names.
    pub fn list() -> Vec<&'static str> {
        vec![
            "Entropy Sweep: Cost of Existing",
            "Catastrophe Resilience: Survival Under Fire",
            "Inequality Threshold: When Does Redistribution Help?",
            "Treasury Stability: Reserve Deployment Policy",
            "Reserve Stress — Calm",
            "Reserve Stress — Moderate",
            "Reserve Stress — Stressed",
            "Reserve Stress — Crisis",
            "Resource Depletion — Abundant",
            "Resource Depletion — Normal",
            "Resource Depletion — Constrained",
            "Resource Depletion — Scarce",
            "Evolution Forbidden: Survival Without Adaptation",
            "Resilience Matrix Q1: Full Adaptation",
            "Resilience Matrix Q2: Immune Only",
            "Resilience Matrix Q3: Genetic Only",
            "Resilience Matrix Q4: Fully Static",
            "Multi-Axis Collapse Vector: Attractor Boundary Search",
            "Metabolic Inversion: The Oxygen Attack",
            "Basal Inversion: The Starvation",
            "Dual Inversion: The Final Escalation",
            // Season 2
            "S1 Treasury Disabled: Baseline",
            "S1 Treasury Disabled: Hostile",
            "S2 ATP Decay Disabled: Baseline",
            "S2 ATP Decay Disabled: Hostile",
            // Season 3: Coupled Invariant Violations
            "S3 Coupled: Decay OFF + Treasury OFF",
            "S3 Coupled: Decay OFF + Grants OFF",
            "S3 Coupled: Decay OFF + Floor OFF",
            "S3 Coupled: All Safety OFF",
        ]
    }

    // ── Season 3: Coupled Invariant Violations ──────────────────────────
    //
    // Supervisor v2 directive: "Move from single-invariant tests to coupled
    // invariant violations." All S3 experiments run under hostile conditions
    // (max catastrophe, max entropy, no Gini tax, no cortex, no mutation)
    // to match S2 Hostile baseline for direct comparison.

    /// Hostile pressure config shared by all S3 experiments.
    fn s3_hostile_pressure() -> PressureConfig {
        let mut p = PressureConfig::default();
        p.catastrophe_base_prob = 0.03;
        p.entropy_coeff = 0.0001;
        p.gini_wealth_tax_threshold = 1.0;
        p.gini_wealth_tax_rate = 0.0;
        p.treasury_overflow_threshold = 1.0;
        p
    }

    /// Metrics collected by all S3 experiments (20 metrics including S2 inequality suite).
    fn s3_metrics() -> Vec<Metric> {
        vec![
            Metric::FinalPopulation,
            Metric::Collapsed,
            Metric::SurvivalEpochs,
            Metric::MeanPopulation,
            Metric::MinPopulation,
            Metric::MeanFitness,
            Metric::GiniCoefficient,
            Metric::TotalBirths,
            Metric::TotalDeaths,
            Metric::TreasuryRatio,
            Metric::MaxTreasuryReserve,
            Metric::BirthDeathRatio,
            Metric::AtpVariance,
            Metric::WealthConcentrationIndex,
            Metric::MedianMeanAtpDivergence,
            Metric::MeanGiniCoefficient,
            Metric::MaxGiniCoefficient,
            Metric::ReproductiveInequalityIndex,
            Metric::SurvivalInequalityIndex,
            Metric::TopDecilePersistence,
        ]
    }

    /// S3-A: ATP decay OFF + treasury cycling OFF (hostile).
    ///
    /// Couples two economic invariant violations: wealth never erodes AND
    /// never redistributes. Supervisor predicts: "Stratification amplification —
    /// wealth locks in AND no redistribution."
    pub fn s3_decay_treasury_off() -> ExperimentConfig {
        ExperimentConfig {
            name: "S3 Coupled: Decay OFF + Treasury OFF".into(),
            hypothesis: "With ATP decay disabled AND treasury cycling disabled under \
                         hostile conditions, wealth never erodes and never redistributes. \
                         This couples two economic invariant violations. The Gini tax is \
                         also disabled. Expected: permanent wealth stratification where \
                         founding agents monopolize reproduction indefinitely. The \
                         extinction floor should still prevent true collapse, producing \
                         maximal inequality without extinction.".into(),
            sweep: ParameterSweep::new(SweepVariable::SoftCap, 30.0, 180.0, 30.0),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: Self::s3_metrics(),
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(Self::s3_hostile_pressure()),
            mutation_rate_override: Some(0.0),
            cortex_enabled_override: Some(false),
            base_stress_override: Some(StressConfig {
                atp_decay_enabled: false,
                treasury_cycling_enabled: false,
                ..StressConfig::default()
            }),
            base_seed: 42,
        }
    }

    /// S3-A quick variant (2 steps × 5 runs × 100 epochs).
    pub fn s3_decay_treasury_off_quick() -> ExperimentConfig {
        let mut config = Self::s3_decay_treasury_off();
        config.name = "S3 Coupled: Decay OFF + Treasury OFF [QUICK]".into();
        config.sweep = ParameterSweep::new(SweepVariable::SoftCap, 60.0, 120.0, 60.0);
        config.runs_per_step = 5;
        config.epochs_per_run = 100;
        config
    }

    /// S3-B: ATP decay OFF + reproduction grants OFF (hostile).
    ///
    /// Children receive 0 ATP at birth (no CHILD_GRANT). Combined with
    /// immortal wealth (no decay), this creates demographic freeze where
    /// new agents cannot survive their first basal metabolic cost.
    /// Supervisor predicts: "Demographic freeze."
    pub fn s3_decay_grants_off() -> ExperimentConfig {
        ExperimentConfig {
            name: "S3 Coupled: Decay OFF + Grants OFF".into(),
            hypothesis: "With ATP decay disabled AND reproduction grants disabled, \
                         children are born with 0 ATP while existing wealthy agents \
                         retain their balances forever. New agents cannot survive their \
                         first basal metabolic cost and die immediately, creating \
                         demographic freeze: a static population of immortal wealthy \
                         agents with zero successful reproduction. This is a coupled \
                         economic + demographic invariant violation.".into(),
            sweep: ParameterSweep::new(SweepVariable::SoftCap, 30.0, 180.0, 30.0),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: Self::s3_metrics(),
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(Self::s3_hostile_pressure()),
            mutation_rate_override: Some(0.0),
            cortex_enabled_override: Some(false),
            base_stress_override: Some(StressConfig {
                atp_decay_enabled: false,
                reproduction_grants_enabled: false,
                ..StressConfig::default()
            }),
            base_seed: 42,
        }
    }

    /// S3-B quick variant (2 steps × 5 runs × 100 epochs).
    pub fn s3_decay_grants_off_quick() -> ExperimentConfig {
        let mut config = Self::s3_decay_grants_off();
        config.name = "S3 Coupled: Decay OFF + Grants OFF [QUICK]".into();
        config.sweep = ParameterSweep::new(SweepVariable::SoftCap, 60.0, 120.0, 60.0);
        config.runs_per_step = 5;
        config.epochs_per_run = 100;
        config
    }

    /// S3-C: ATP decay OFF + extinction floor OFF (hostile).
    ///
    /// Removes the population safety net. Without extinction floor: juvenile
    /// protection is disabled, stasis tolerance drops to 1 epoch, and
    /// populations can crash below MIN_POPULATION_SIZE to true zero.
    /// Combined with immortal wealth (no decay), decline becomes irreversible.
    /// Supervisor predicts: "True extinction boundary."
    pub fn s3_decay_floor_off() -> ExperimentConfig {
        ExperimentConfig {
            name: "S3 Coupled: Decay OFF + Floor OFF".into(),
            hypothesis: "With ATP decay disabled AND extinction floor disabled, the \
                         population safety net is removed. Juvenile protection is \
                         disabled (no 25% basal rebate for young agents), stasis \
                         tolerance drops from 8 to 1 epoch (instant death on stasis), \
                         and populations can crash to true zero. Combined with immortal \
                         wealth (no decay), wealthy agents persist but population \
                         decline becomes irreversible. This tests whether the extinction \
                         floor is the true structural stabilizer. Expected: true \
                         extinction events, especially at low carrying capacities.".into(),
            sweep: ParameterSweep::new(SweepVariable::SoftCap, 30.0, 180.0, 30.0),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: Self::s3_metrics(),
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(Self::s3_hostile_pressure()),
            mutation_rate_override: Some(0.0),
            cortex_enabled_override: Some(false),
            base_stress_override: Some(StressConfig {
                atp_decay_enabled: false,
                extinction_floor_enabled: false,
                ..StressConfig::default()
            }),
            base_seed: 42,
        }
    }

    /// S3-C quick variant (2 steps × 5 runs × 100 epochs).
    pub fn s3_decay_floor_off_quick() -> ExperimentConfig {
        let mut config = Self::s3_decay_floor_off();
        config.name = "S3 Coupled: Decay OFF + Floor OFF [QUICK]".into();
        config.sweep = ParameterSweep::new(SweepVariable::SoftCap, 60.0, 120.0, 60.0);
        config.runs_per_step = 5;
        config.epochs_per_run = 100;
        config
    }

    /// S3-D: All metabolic safety OFF (hostile).
    ///
    /// ATP decay OFF + treasury cycling OFF + reproduction grants OFF +
    /// extinction floor OFF. Every economic and metabolic safety mechanism
    /// removed simultaneously. Supervisor predicts: "High probability collapse."
    pub fn s3_all_off() -> ExperimentConfig {
        ExperimentConfig {
            name: "S3 Coupled: All Safety OFF".into(),
            hypothesis: "All metabolic and economic safety mechanisms disabled: ATP \
                         decay never erodes wealth, treasury never redistributes, \
                         children receive 0 ATP at birth, and the extinction floor \
                         is removed (no juvenile protection, instant stasis death, \
                         populations can reach zero). Under hostile conditions with \
                         all adaptation also disabled, this is the maximal coupled \
                         invariant violation. Expected: rapid collapse as the \
                         population has no recovery mechanism from any perturbation.".into(),
            sweep: ParameterSweep::new(SweepVariable::SoftCap, 30.0, 180.0, 30.0),
            runs_per_step: 20,
            epochs_per_run: 500,
            metrics: Self::s3_metrics(),
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: Some(Self::s3_hostile_pressure()),
            mutation_rate_override: Some(0.0),
            cortex_enabled_override: Some(false),
            base_stress_override: Some(StressConfig {
                atp_decay_enabled: false,
                treasury_cycling_enabled: false,
                reproduction_grants_enabled: false,
                extinction_floor_enabled: false,
                ..StressConfig::default()
            }),
            base_seed: 42,
        }
    }

    /// S3-D quick variant (2 steps × 5 runs × 100 epochs).
    pub fn s3_all_off_quick() -> ExperimentConfig {
        let mut config = Self::s3_all_off();
        config.name = "S3 Coupled: All Safety OFF [QUICK]".into();
        config.sweep = ParameterSweep::new(SweepVariable::SoftCap, 60.0, 120.0, 60.0);
        config.runs_per_step = 5;
        config.epochs_per_run = 100;
        config
    }

    /// Season 3 Coupled Invariant Violation Suite: all S3 experiments.
    pub fn s3_coupled_suite() -> Vec<(&'static str, ExperimentConfig)> {
        vec![
            ("s3_decay_treasury_off", Self::s3_decay_treasury_off()),
            ("s3_decay_grants_off", Self::s3_decay_grants_off()),
            ("s3_decay_floor_off", Self::s3_decay_floor_off()),
            ("s3_all_off", Self::s3_all_off()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::ExperimentRunner;
    use crate::report::ExperimentReport;

    #[test]
    fn entropy_sweep_config_valid() {
        let config = FlagshipExperiments::entropy_sweep();
        assert_eq!(config.sweep.step_count(), 10);
        assert_eq!(config.total_worlds(), 200);
        assert!(config.metrics.len() >= 10);
    }

    #[test]
    fn catastrophe_resilience_config_valid() {
        let config = FlagshipExperiments::catastrophe_resilience();
        assert_eq!(config.sweep.step_count(), 7);
        assert_eq!(config.total_worlds(), 140);
        assert!(config.metrics.len() >= 9);
    }

    #[test]
    fn inequality_threshold_config_valid() {
        let config = FlagshipExperiments::inequality_threshold();
        assert_eq!(config.sweep.step_count(), 8);
        assert_eq!(config.total_worlds(), 160);
        assert!(config.metrics.len() >= 10);
    }

    #[test]
    fn quick_entropy_sweep_runs() {
        let config = FlagshipExperiments::entropy_sweep_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 10);
        assert!(result.total_worlds > 0);
        // Should complete without panic
    }

    #[test]
    fn quick_catastrophe_resilience_runs() {
        let config = FlagshipExperiments::catastrophe_resilience_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 7);
    }

    #[test]
    fn quick_inequality_threshold_runs() {
        let config = FlagshipExperiments::inequality_threshold_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 8);
    }

    #[test]
    fn quick_experiment_produces_report() {
        let config = FlagshipExperiments::entropy_sweep_quick();
        let result = ExperimentRunner::run(&config);
        let report = ExperimentReport::generate(
            &result,
            vec!["Quick test finding".into()],
        );
        assert!(report.text_report.contains("Entropy Sweep"));
        assert!(report.csv_data.lines().count() > 1);
        assert!(report.manifest.verify());
    }

    #[test]
    fn treasury_stability_config_valid() {
        let config = FlagshipExperiments::treasury_stability();
        assert_eq!(config.sweep.step_count(), 9);
        assert_eq!(config.total_worlds(), 180);
        assert!(config.metrics.len() >= 12);
    }

    #[test]
    fn quick_treasury_stability_runs() {
        let config = FlagshipExperiments::treasury_stability_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 9);
    }

    #[test]
    fn flagship_list() {
        let list = FlagshipExperiments::list();
        assert_eq!(list.len(), 25);
        assert!(list[0].contains("Entropy"));
        assert!(list[1].contains("Catastrophe"));
        assert!(list[2].contains("Inequality"));
        assert!(list[3].contains("Treasury"));
        assert!(list[4].contains("Calm"));
        assert!(list[5].contains("Moderate"));
        assert!(list[6].contains("Stressed"));
        assert!(list[7].contains("Crisis"));
        assert!(list[8].contains("Abundant"));
        assert!(list[9].contains("Normal"));
        assert!(list[10].contains("Constrained"));
        assert!(list[11].contains("Scarce"));
        assert!(list[12].contains("Evolution Forbidden"));
        assert!(list[13].contains("Q1"));
        assert!(list[14].contains("Q2"));
        assert!(list[15].contains("Q3"));
        assert!(list[16].contains("Q4"));
        assert!(list[17].contains("Multi-Axis"));
        assert!(list[18].contains("Metabolic Inversion"));
        assert!(list[19].contains("Basal Inversion"));
        assert!(list[20].contains("Dual Inversion"));
        assert!(list[21].contains("S1"));
        assert!(list[22].contains("S1"));
    }

    #[test]
    fn reserve_calm_config_valid() {
        let config = FlagshipExperiments::reserve_calm();
        assert_eq!(config.sweep.step_count(), 9);
        assert_eq!(config.total_worlds(), 135);
        assert!(config.metrics.len() >= 13);
        assert!(config.base_pressure_override.is_some());
        let p = config.base_pressure_override.unwrap();
        assert!((p.catastrophe_base_prob - 0.001).abs() < 1e-10);
    }

    #[test]
    fn reserve_crisis_config_valid() {
        let config = FlagshipExperiments::reserve_crisis();
        assert_eq!(config.sweep.step_count(), 9);
        assert_eq!(config.total_worlds(), 135);
        let p = config.base_pressure_override.unwrap();
        assert!((p.catastrophe_base_prob - 0.030).abs() < 1e-10);
    }

    #[test]
    fn reserve_stress_suite_valid() {
        let suite = FlagshipExperiments::reserve_stress_suite();
        assert_eq!(suite.len(), 4);
        let total_worlds: usize = suite.iter().map(|(_, c)| c.total_worlds()).sum();
        assert_eq!(total_worlds, 540);
    }

    #[test]
    fn quick_reserve_calm_runs() {
        let config = FlagshipExperiments::reserve_calm_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 9);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn resource_depletion_abundant_config_valid() {
        let config = FlagshipExperiments::resource_depletion_abundant();
        assert_eq!(config.sweep.step_count(), 10);
        assert_eq!(config.total_worlds(), 150);
        assert!(config.metrics.len() >= 13);
        assert!(config.base_pressure_override.is_some());
        let p = config.base_pressure_override.unwrap();
        assert_eq!(p.soft_cap, 200);
    }

    #[test]
    fn resource_depletion_scarce_config_valid() {
        let config = FlagshipExperiments::resource_depletion_scarce();
        assert_eq!(config.sweep.step_count(), 10);
        assert_eq!(config.total_worlds(), 150);
        let p = config.base_pressure_override.unwrap();
        assert_eq!(p.soft_cap, 30);
    }

    #[test]
    fn resource_depletion_suite_valid() {
        let suite = FlagshipExperiments::resource_depletion_suite();
        assert_eq!(suite.len(), 4);
        let total_worlds: usize = suite.iter().map(|(_, c)| c.total_worlds()).sum();
        assert_eq!(total_worlds, 600);
    }

    #[test]
    fn quick_resource_depletion_abundant_runs() {
        let config = FlagshipExperiments::resource_depletion_abundant_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 10);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn evolution_forbidden_config_valid() {
        let config = FlagshipExperiments::evolution_forbidden();
        assert_eq!(config.sweep.step_count(), 7);
        assert_eq!(config.total_worlds(), 140);
        assert!(config.metrics.len() >= 10);
        assert_eq!(config.mutation_rate_override, Some(0.0));
        assert!(config.base_pressure_override.is_none());
    }

    #[test]
    fn quick_evolution_forbidden_runs() {
        let config = FlagshipExperiments::evolution_forbidden_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 7);
        assert!(result.total_worlds > 0);
        // Verify mutation was actually suppressed: pressure_mutations should be 0
        for step in &result.steps {
            for trial in &step.trials {
                if let Some(&mutations) = trial.metrics.get("total_trait_mutations") {
                    assert_eq!(mutations, 0.0,
                        "Expected zero trait mutations with mutation_rate_override=0.0");
                }
            }
        }
    }

    // â”€â”€â”€ Resilience Matrix Tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn resilience_q1_config_valid() {
        let config = FlagshipExperiments::resilience_q1_both();
        assert_eq!(config.sweep.step_count(), 11);
        assert_eq!(config.total_worlds(), 220);
        assert!(config.metrics.len() >= 14);
        assert!(config.mutation_rate_override.is_none());
        assert!(config.cortex_enabled_override.is_none());
    }

    #[test]
    fn resilience_q4_config_valid() {
        let config = FlagshipExperiments::resilience_q4_static();
        assert_eq!(config.sweep.step_count(), 11);
        assert_eq!(config.total_worlds(), 220);
        assert_eq!(config.mutation_rate_override, Some(0.0));
        assert_eq!(config.cortex_enabled_override, Some(false));
    }

    #[test]
    fn resilience_matrix_suite_valid() {
        let suite = FlagshipExperiments::resilience_matrix_suite();
        assert_eq!(suite.len(), 4);
        let total_worlds: usize = suite.iter().map(|(_, c)| c.total_worlds()).sum();
        assert_eq!(total_worlds, 880); // 4 Ã-- 220
    }

    #[test]
    fn quick_resilience_q1_runs() {
        let config = FlagshipExperiments::resilience_q1_both_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 11);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn quick_resilience_q4_static_runs() {
        let config = FlagshipExperiments::resilience_q4_static_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 11);
        assert!(result.total_worlds > 0);
        // THE TRUE KILL TEST: both mutation layers must be zero
        for step in &result.steps {
            for trial in &step.trials {
                if let Some(&mutations) = trial.metrics.get("total_trait_mutations") {
                    assert_eq!(mutations, 0.0,
                        "Q4: expected zero trait mutations");
                }
                if let Some(&pressure) = trial.metrics.get("total_pressure_mutations") {
                    assert_eq!(pressure, 0.0,
                        "Q4: expected zero pressure mutations (cortex disabled)");
                }
            }
        }
    }
    // --- Multi-Axis Collapse Tests --------------------------------------

    #[test]
    fn multi_axis_collapse_config_valid() {
        let config = FlagshipExperiments::multi_axis_collapse();
        assert_eq!(config.sweep.step_count(), 11);
        assert_eq!(config.total_worlds(), 220);
        assert!(config.metrics.len() >= 17); // all 18 metrics
        // Verify all hostile axes are set
        assert_eq!(config.mutation_rate_override, Some(0.0));
        assert_eq!(config.cortex_enabled_override, Some(false));
        let p = config.base_pressure_override.as_ref().unwrap();
        assert!((p.catastrophe_base_prob - 0.03).abs() < 1e-10);
        assert!((p.entropy_coeff - 0.0001).abs() < 1e-10);
        assert!((p.gini_wealth_tax_threshold - 1.0).abs() < 1e-10);
        assert!((p.gini_wealth_tax_rate - 0.0).abs() < 1e-10);
        assert!((p.treasury_overflow_threshold - 1.0).abs() < 1e-10);
    }

    #[test]
    fn quick_multi_axis_collapse_runs() {
        let config = FlagshipExperiments::multi_axis_collapse_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 11);
        assert!(result.total_worlds > 0);
        // Verify mutation was suppressed across all trials
        for step in &result.steps {
            for trial in &step.trials {
                if let Some(&mutations) = trial.metrics.get("total_trait_mutations") {
                    assert_eq!(mutations, 0.0,
                        "Expected zero trait mutations in multi-axis collapse");
                }
                // Verify cortex did not fire (no pressure mutations)
                if let Some(&pm) = trial.metrics.get("total_pressure_mutations") {
                    assert_eq!(pm, 0.0,
                        "Expected zero pressure mutations with cortex disabled");
                }
            }
        }
    }

    // --- Tournament Tests (Week 4) --------------------------------------

    #[test]
    fn basal_inversion_config_valid() {
        let config = FlagshipExperiments::basal_inversion();
        assert_eq!(config.sweep.step_count(), 10);
        assert_eq!(config.total_worlds(), 200);
        assert!(config.metrics.len() >= 17);
        assert_eq!(config.mutation_rate_override, Some(0.0));
        assert_eq!(config.cortex_enabled_override, Some(false));
        let p = config.base_pressure_override.as_ref().unwrap();
        assert!((p.catastrophe_base_prob - 0.03).abs() < 1e-10);
    }

    #[test]
    fn dual_inversion_config_valid() {
        let config = FlagshipExperiments::dual_inversion();
        assert_eq!(config.sweep.step_count(), 10);
        assert_eq!(config.total_worlds(), 200);
        assert!(config.base_stress_override.is_some());
        let stress = config.base_stress_override.as_ref().unwrap();
        assert!((stress.replication_cost_multiplier - 3.0).abs() < 1e-10);
    }

    #[test]
    fn tournament_suite_valid() {
        let suite = FlagshipExperiments::tournament_suite();
        assert_eq!(suite.len(), 3);
        let total_worlds: usize = suite.iter().map(|(_, c)| c.total_worlds()).sum();
        assert_eq!(total_worlds, 580); // 180 + 200 + 200
    }

    #[test]
    fn quick_basal_inversion_runs() {
        let config = FlagshipExperiments::basal_inversion_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 10);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn quick_dual_inversion_runs() {
        let config = FlagshipExperiments::dual_inversion_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 10);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn metabolic_inversion_config_valid() {
        let config = FlagshipExperiments::metabolic_inversion();
        assert_eq!(config.sweep.step_count(), 9);
        assert_eq!(config.total_worlds(), 180);
        assert!(config.metrics.len() >= 17);
        assert_eq!(config.mutation_rate_override, Some(0.0));
        assert_eq!(config.cortex_enabled_override, Some(false));
        let p = config.base_pressure_override.as_ref().unwrap();
        assert!((p.catastrophe_base_prob - 0.03).abs() < 1e-10);
        assert!((p.entropy_coeff - 0.0001).abs() < 1e-10);
        assert!((p.gini_wealth_tax_threshold - 1.0).abs() < 1e-10);
        assert!((p.gini_wealth_tax_rate - 0.0).abs() < 1e-10);
        assert!((p.treasury_overflow_threshold - 1.0).abs() < 1e-10);
        assert_eq!(config.sweep.variable, SweepVariable::ReplicationCostMultiplier);
    }

    #[test]
    fn quick_metabolic_inversion_runs() {
        let config = FlagshipExperiments::metabolic_inversion_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 9);
        assert!(result.total_worlds > 0);
        for step in &result.steps {
            for trial in &step.trials {
                if let Some(&mutations) = trial.metrics.get("total_trait_mutations") {
                    assert_eq!(mutations, 0.0,
                        "Expected zero trait mutations in metabolic inversion");
                }
                if let Some(&pm) = trial.metrics.get("total_pressure_mutations") {
                    assert_eq!(pm, 0.0,
                        "Expected zero pressure mutations with cortex disabled");
                }
            }
        }
    }

    // ─── Season 2: Structural Invariant Violation Tests ──────────────

    #[test]
    fn s1_treasury_disabled_baseline_config_valid() {
        let config = FlagshipExperiments::s1_treasury_disabled_baseline();
        assert_eq!(config.sweep.step_count(), 6);
        assert_eq!(config.total_worlds(), 120);
        assert!(config.metrics.len() >= 14);
        assert!(config.base_pressure_override.is_none());
        let stress = config.base_stress_override.as_ref().unwrap();
        assert!(!stress.treasury_cycling_enabled);
    }

    #[test]
    fn s1_treasury_disabled_hostile_config_valid() {
        let config = FlagshipExperiments::s1_treasury_disabled_hostile();
        assert_eq!(config.sweep.step_count(), 6);
        assert_eq!(config.total_worlds(), 120);
        assert_eq!(config.mutation_rate_override, Some(0.0));
        assert_eq!(config.cortex_enabled_override, Some(false));
        let stress = config.base_stress_override.as_ref().unwrap();
        assert!(!stress.treasury_cycling_enabled);
        let p = config.base_pressure_override.as_ref().unwrap();
        assert!((p.catastrophe_base_prob - 0.03).abs() < 1e-10);
    }

    #[test]
    fn quick_s1_treasury_disabled_baseline_runs() {
        let config = FlagshipExperiments::s1_treasury_disabled_baseline_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 6);
        assert!(result.total_worlds > 0);
        // Verify min_population metric is collected
        for step in &result.steps {
            for trial in &step.trials {
                assert!(trial.metrics.contains_key("min_population"),
                    "Expected min_population metric in Season 2 experiment");
                assert!(trial.metrics.contains_key("max_treasury_reserve"),
                    "Expected max_treasury_reserve metric in Season 2 experiment");
            }
        }
    }

    #[test]
    fn quick_s1_treasury_disabled_hostile_runs() {
        let config = FlagshipExperiments::s1_treasury_disabled_hostile_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 6);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn s2_invariant_suite_valid() {
        let suite = FlagshipExperiments::s2_invariant_suite();
        assert_eq!(suite.len(), 4);
        for (name, config) in &suite {
            assert!(!name.is_empty());
            assert!(config.total_worlds() > 0);
        }
    }

    #[test]
    fn s2_atp_decay_disabled_baseline_config_valid() {
        let config = FlagshipExperiments::s2_atp_decay_disabled_baseline();
        assert_eq!(config.sweep.step_count(), 6);
        assert_eq!(config.total_worlds(), 120);
        assert!(config.metrics.len() >= 18);
        assert!(config.base_pressure_override.is_none());
        let stress = config.base_stress_override.as_ref().unwrap();
        assert!(!stress.atp_decay_enabled);
        assert!(stress.treasury_cycling_enabled); // treasury still active
    }

    #[test]
    fn s2_atp_decay_disabled_hostile_config_valid() {
        let config = FlagshipExperiments::s2_atp_decay_disabled_hostile();
        assert_eq!(config.sweep.step_count(), 6);
        assert_eq!(config.total_worlds(), 120);
        assert_eq!(config.mutation_rate_override, Some(0.0));
        assert_eq!(config.cortex_enabled_override, Some(false));
        let stress = config.base_stress_override.as_ref().unwrap();
        assert!(!stress.atp_decay_enabled);
        let p = config.base_pressure_override.as_ref().unwrap();
        assert!((p.catastrophe_base_prob - 0.03).abs() < 1e-10);
    }

    #[test]
    fn quick_s2_atp_decay_disabled_baseline_runs() {
        let config = FlagshipExperiments::s2_atp_decay_disabled_baseline_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 2);
        assert!(result.total_worlds > 0);
        // Verify inequality metrics are collected
        for step in &result.steps {
            for trial in &step.trials {
                assert!(trial.metrics.contains_key("wealth_concentration_index"),
                    "Expected wealth_concentration_index metric in S2 experiment");
                assert!(trial.metrics.contains_key("median_mean_atp_divergence"),
                    "Expected median_mean_atp_divergence metric in S2 experiment");
                assert!(trial.metrics.contains_key("mean_gini_coefficient"),
                    "Expected mean_gini_coefficient metric in S2 experiment");
                assert!(trial.metrics.contains_key("reproductive_inequality_index"),
                    "Expected reproductive_inequality_index metric in S2 experiment");
            }
        }
    }

    #[test]
    fn quick_s2_atp_decay_disabled_hostile_runs() {
        let config = FlagshipExperiments::s2_atp_decay_disabled_hostile_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 2);
        assert!(result.total_worlds > 0);
    }

    // ── Season 3: Coupled Invariant Violation Tests ─────────────────────

    #[test]
    fn s3_coupled_suite_valid() {
        let suite = FlagshipExperiments::s3_coupled_suite();
        assert_eq!(suite.len(), 4);
        for (name, config) in &suite {
            assert!(!name.is_empty());
            assert_eq!(config.total_worlds(), 120);
        }
    }

    #[test]
    fn s3_decay_treasury_off_config_valid() {
        let config = FlagshipExperiments::s3_decay_treasury_off();
        assert_eq!(config.sweep.step_count(), 6);
        assert_eq!(config.total_worlds(), 120);
        assert!(config.metrics.len() >= 18);
        let stress = config.base_stress_override.as_ref().unwrap();
        assert!(!stress.atp_decay_enabled);
        assert!(!stress.treasury_cycling_enabled);
        assert!(stress.reproduction_grants_enabled); // grants still active
        assert!(stress.extinction_floor_enabled); // floor still active
    }

    #[test]
    fn s3_decay_grants_off_config_valid() {
        let config = FlagshipExperiments::s3_decay_grants_off();
        assert_eq!(config.sweep.step_count(), 6);
        assert_eq!(config.total_worlds(), 120);
        let stress = config.base_stress_override.as_ref().unwrap();
        assert!(!stress.atp_decay_enabled);
        assert!(!stress.reproduction_grants_enabled);
        assert!(stress.treasury_cycling_enabled); // treasury still active
        assert!(stress.extinction_floor_enabled); // floor still active
    }

    #[test]
    fn s3_decay_floor_off_config_valid() {
        let config = FlagshipExperiments::s3_decay_floor_off();
        assert_eq!(config.sweep.step_count(), 6);
        assert_eq!(config.total_worlds(), 120);
        let stress = config.base_stress_override.as_ref().unwrap();
        assert!(!stress.atp_decay_enabled);
        assert!(!stress.extinction_floor_enabled);
        assert!(stress.treasury_cycling_enabled); // treasury still active
        assert!(stress.reproduction_grants_enabled); // grants still active
    }

    #[test]
    fn s3_all_off_config_valid() {
        let config = FlagshipExperiments::s3_all_off();
        assert_eq!(config.sweep.step_count(), 6);
        assert_eq!(config.total_worlds(), 120);
        let stress = config.base_stress_override.as_ref().unwrap();
        assert!(!stress.atp_decay_enabled);
        assert!(!stress.treasury_cycling_enabled);
        assert!(!stress.reproduction_grants_enabled);
        assert!(!stress.extinction_floor_enabled);
    }

    #[test]
    fn quick_s3_decay_treasury_off_runs() {
        let config = FlagshipExperiments::s3_decay_treasury_off_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 2);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn quick_s3_decay_grants_off_runs() {
        let config = FlagshipExperiments::s3_decay_grants_off_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 2);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn quick_s3_decay_floor_off_runs() {
        let config = FlagshipExperiments::s3_decay_floor_off_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 2);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn quick_s3_all_off_runs() {
        let config = FlagshipExperiments::s3_all_off_quick();
        let result = ExperimentRunner::run(&config);
        assert_eq!(result.steps.len(), 2);
        assert!(result.total_worlds > 0);
    }

    #[test]
    fn experiment_list_includes_s3() {
        let list = FlagshipExperiments::list();
        assert_eq!(list.len(), 29);
        assert!(list.contains(&"S3 Coupled: Decay OFF + Treasury OFF"));
        assert!(list.contains(&"S3 Coupled: Decay OFF + Grants OFF"));
        assert!(list.contains(&"S3 Coupled: Decay OFF + Floor OFF"));
        assert!(list.contains(&"S3 Coupled: All Safety OFF"));
    }

}
