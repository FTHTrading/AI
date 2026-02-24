// stress.rs — Adversarial Stress-Testing Layer
//
// Provides a StressConfig that multiplies and biases every lever in
// the organism's epoch tick, letting us answer:
//
//   "Is Genesis stable because it is robust, or because it is tuned?"
//
// Usage:
//   World::with_stress(config)  — attach a profile
//   World::clear_stress()       — return to baseline
//
// All stressors are multiplicative against existing parameters so the
// baseline values remain the single source of truth.

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

// ─── Configuration ─────────────────────────────────────────────────────

/// All multipliers default to 1.0 (no effect).
/// All probabilities default to 0.0 (disabled).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressConfig {
    // ── Economic levers ──────────────────────────────────────────────
    /// Scale the treasury skim rate. 1.5x → 3x drains agents faster.
    pub skim_rate_multiplier: f64,

    /// Scale replication cost. 1.5x makes births expensive; 2x can stop
    /// reproduction entirely when circulating ATP is low.
    pub replication_cost_multiplier: f64,

    /// Scale basal metabolic cost. 1.5x → 3x increases extinction pressure.
    pub basal_cost_multiplier: f64,

    // ── Environmental levers ─────────────────────────────────────────
    /// Bias catastrophe clustering.  When a catastrophe is already active,
    /// the next-epoch catastrophe probability is multiplied by
    /// `(1 + catastrophe_cluster_bias)`.  0.0 = uniform; 2.0 = 3× higher.
    pub catastrophe_cluster_bias: f64,

    // ── Genetic levers ───────────────────────────────────────────────
    /// Scale mutation volatility. 2x = more radical trait shifts per epoch.
    pub mutation_volatility_multiplier: f64,

    // ── Treasury levers ──────────────────────────────────────────────
    /// Per-epoch probability of freezing all treasury releases.
    /// 0.10 = 10% chance each epoch the seasonal release is suppressed.
    pub treasury_lock_probability: f64,

    // ── Structural invariant toggles (Season 2) ─────────────────────
    /// Enable/disable treasury cycling (redistribution back to agents).
    /// When false, ATP flows into the treasury but never comes out:
    /// no stipends, no crisis spending, no overflow redistribution,
    /// no seasonal release.  This is a structural invariant violation,
    /// not a parameter sweep — it breaks the recycling loop.
    pub treasury_cycling_enabled: bool,

    /// Enable/disable the 2% per-epoch ATP decay (S2 invariant).
    /// When false, agent balances never decay — wealth becomes immortal.
    /// Predicted effect: wealth concentration → demographic stagnation
    /// → reproductive starvation cascade.
    pub atp_decay_enabled: bool,

    /// Enable/disable reproduction grants (S3 invariant).
    /// When false, children receive 0 ATP at birth instead of CHILD_GRANT (8.0).
    /// Predicted effect: children immediately enter stasis, dramatically
    /// reducing reproductive viability and causing demographic freeze.
    pub reproduction_grants_enabled: bool,

    /// Enable/disable the extinction floor safety net (S3 invariant).
    /// When false: stasis tolerance drops to 1 epoch (instant death on stasis),
    /// juvenile basal rebate is disabled, and MIN_POPULATION_SIZE check is
    /// bypassed — removing all mechanisms that buffer agents from death.
    /// Predicted effect: removes the population floor, enabling true extinction.
    pub extinction_floor_enabled: bool,
}

impl Default for StressConfig {
    fn default() -> Self {
        Self {
            skim_rate_multiplier: 1.0,
            replication_cost_multiplier: 1.0,
            basal_cost_multiplier: 1.0,
            catastrophe_cluster_bias: 0.0,
            mutation_volatility_multiplier: 1.0,
            treasury_lock_probability: 0.0,
            treasury_cycling_enabled: true,
            atp_decay_enabled: true,
            reproduction_grants_enabled: true,
            extinction_floor_enabled: true,
        }
    }
}

impl StressConfig {
    /// Baseline — no stress.
    pub fn baseline() -> Self {
        Self::default()
    }

    /// Mild — slightly elevated costs, 5% treasury lock chance.
    pub fn mild() -> Self {
        Self {
            skim_rate_multiplier: 1.5,
            replication_cost_multiplier: 1.2,
            basal_cost_multiplier: 1.2,
            catastrophe_cluster_bias: 0.5,
            mutation_volatility_multiplier: 1.2,
            treasury_lock_probability: 0.05,
            treasury_cycling_enabled: true,
            atp_decay_enabled: true,
            reproduction_grants_enabled: true,
            extinction_floor_enabled: true,
        }
    }

    /// Moderate — 2× on key costs, 10% lock probability.
    pub fn moderate() -> Self {
        Self {
            skim_rate_multiplier: 2.0,
            replication_cost_multiplier: 1.5,
            basal_cost_multiplier: 1.5,
            catastrophe_cluster_bias: 1.0,
            mutation_volatility_multiplier: 1.5,
            treasury_lock_probability: 0.10,
            treasury_cycling_enabled: true,
            atp_decay_enabled: true,
            reproduction_grants_enabled: true,
            extinction_floor_enabled: true,
        }
    }

    /// Brutal — 3× skim, 2× costs, 20% lock, max clustering.
    pub fn brutal() -> Self {
        Self {
            skim_rate_multiplier: 3.0,
            replication_cost_multiplier: 2.0,
            basal_cost_multiplier: 2.0,
            catastrophe_cluster_bias: 2.0,
            mutation_volatility_multiplier: 2.0,
            treasury_lock_probability: 0.20,
            treasury_cycling_enabled: true,
            atp_decay_enabled: true,
            reproduction_grants_enabled: true,
            extinction_floor_enabled: true,
        }
    }

    /// Hoarding — lock treasury constantly (95% chance each epoch).
    pub fn hoarding() -> Self {
        Self {
            skim_rate_multiplier: 2.0,
            replication_cost_multiplier: 1.0,
            basal_cost_multiplier: 1.0,
            catastrophe_cluster_bias: 0.0,
            mutation_volatility_multiplier: 1.0,
            treasury_lock_probability: 0.95,
            treasury_cycling_enabled: true,
            atp_decay_enabled: true,
            reproduction_grants_enabled: true,
            extinction_floor_enabled: true,
        }
    }

    /// Mutation runaway — 3× mutation, otherwise baseline.
    pub fn mutation_runaway() -> Self {
        Self {
            mutation_volatility_multiplier: 3.0,
            ..Self::default()
        }
    }

    /// Catastrophe cluster — maximum event clustering, otherwise baseline.
    pub fn catastrophe_cluster() -> Self {
        Self {
            catastrophe_cluster_bias: 2.0,
            ..Self::default()
        }
    }

    /// Parse a named profile string. Returns None if unknown.
    pub fn from_profile(name: &str) -> Option<Self> {
        match name {
            "baseline"            => Some(Self::baseline()),
            "mild"                => Some(Self::mild()),
            "moderate"            => Some(Self::moderate()),
            "brutal"              => Some(Self::brutal()),
            "hoarding"            => Some(Self::hoarding()),
            "mutation-runaway"    => Some(Self::mutation_runaway()),
            "catastrophe-cluster" => Some(Self::catastrophe_cluster()),
            _ => None,
        }
    }

    /// Roll treasury lock for this epoch using deterministic seed.
    pub fn treasury_locked(&self, seed: u64) -> bool {
        if self.treasury_lock_probability <= 0.0 { return false; }
        let roll = lcg_roll(seed);
        roll < self.treasury_lock_probability
    }
}

// ─── Phase-Transition Detector ─────────────────────────────────────────

/// Detects nonlinear population regime shifts:
/// |ΔP| > 3σ within a 50-epoch rolling window.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhaseTransitionDetector {
    /// Rolling window of population samples.
    window: VecDeque<usize>,
    /// Total detected phase transitions.
    pub total_transitions: u64,
    /// Epoch of the most recent phase transition (if any).
    pub last_transition_epoch: Option<u64>,
}

impl PhaseTransitionDetector {
    const WINDOW_SIZE: usize = 50;
    const SIGMA_THRESHOLD: f64 = 3.0;

    pub fn new() -> Self {
        Self::default()
    }

    /// Push latest population, return Some(transition_message) if threshold exceeded.
    pub fn push(&mut self, population: usize, epoch: u64) -> Option<String> {
        self.window.push_back(population);
        if self.window.len() > Self::WINDOW_SIZE {
            self.window.pop_front();
        }
        if self.window.len() < 10 { return None; } // need minimum history

        let n = self.window.len() as f64;
        let mean = self.window.iter().map(|&p| p as f64).sum::<f64>() / n;
        let variance = self.window.iter()
            .map(|&p| (p as f64 - mean).powi(2))
            .sum::<f64>() / n;
        let sigma = variance.sqrt();

        if sigma < 0.5 { return None; } // stable — don't fire on trivially small populations

        // Use current vs window mean as the delta
        let delta = (population as f64 - mean).abs();
        if delta > Self::SIGMA_THRESHOLD * sigma {
            self.total_transitions += 1;
            self.last_transition_epoch = Some(epoch);
            Some(format!(
                "PHASE TRANSITION DETECTED at epoch {} | Δpop={:.1} σ={:.2} ({:.1}σ)",
                epoch, delta, sigma, delta / sigma
            ))
        } else {
            None
        }
    }

    pub fn sigma_ratio(&self) -> f64 {
        let n = self.window.len() as f64;
        if n < 2.0 { return 0.0; }
        let mean = self.window.iter().map(|&p| p as f64).sum::<f64>() / n;
        let variance = self.window.iter()
            .map(|&p| (p as f64 - mean).powi(2))
            .sum::<f64>() / n;
        variance.sqrt()
    }
}

// ─── Stress Metrics Accumulator ────────────────────────────────────────

/// Per-epoch sample for stress run analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSample {
    pub epoch: u64,
    pub population: usize,
    pub mean_fitness: f64,
    pub hoarding_ratio: f64,
    pub birth_death_ratio: f64,
    pub role_entropy: f64,
    pub total_atp: f64,
    pub catastrophe_active: bool,
    pub eco_state: String,
}

/// Aggregate results from a full stress run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressRunResult {
    /// Profile used.
    pub profile: String,
    /// Epoch count of this run.
    pub epochs: u64,
    /// Per-epoch samples (every epoch if < 500, else sampled).
    pub samples: Vec<EpochSample>,
    /// Mean population across run.
    pub mean_population: f64,
    /// Population variance.
    pub population_variance: f64,
    /// Number of extinction events (pop = 0).
    pub extinction_events: u64,
    /// Number of phase transitions detected.
    pub phase_transitions: u64,
    /// Minimum B:D ratio sustained (rolling 100-epoch window).
    pub min_birth_death_ratio: f64,
    /// Maximum hoarding ratio observed.
    pub max_hoarding_ratio: f64,
    /// Epochs where B:D < 0.9 consecutively.
    pub max_consecutive_decline_epochs: u64,
    /// Role entropy at end of run.
    pub final_role_entropy: f64,
    /// Instability flags triggered.
    pub instability_flags: Vec<String>,
}

/// Accumulates metrics across an ongoing stress run.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StressMetrics {
    pub profile_name: String,
    samples: Vec<EpochSample>,
    consecutive_decline: u64,
    max_consecutive_decline: u64,
    pub transition_detector: PhaseTransitionDetector,
    extinction_events: u64,
    instability_flags: Vec<String>,
}

impl StressMetrics {
    pub fn new(profile_name: impl Into<String>) -> Self {
        Self {
            profile_name: profile_name.into(),
            ..Default::default()
        }
    }

    /// Record a sample. Returns any phase-transition log line.
    pub fn record(
        &mut self,
        epoch: u64,
        population: usize,
        mean_fitness: f64,
        hoarding_ratio: f64,
        birth_death_ratio: f64,
        role_entropy: f64,
        total_atp: f64,
        catastrophe_active: bool,
        eco_state: &str,
    ) -> Option<String> {
        if population == 0 {
            self.extinction_events += 1;
        }

        // Track consecutive decline
        if birth_death_ratio < 0.9 {
            self.consecutive_decline += 1;
            if self.consecutive_decline > self.max_consecutive_decline {
                self.max_consecutive_decline = self.consecutive_decline;
            }
        } else {
            self.consecutive_decline = 0;
        }

        // Instability flags
        let cap_estimate = 200.0_f64; // conservative
        if population as f64 / cap_estimate < 0.10 {
            self.instability_flags.push(format!(
                "Epoch {}: Population < 10% cap (pop={})", epoch, population
            ));
        }
        if hoarding_ratio > 0.80 {
            self.instability_flags.push(format!(
                "Epoch {}: Hoarding ratio {:.1}% > 80%", epoch, hoarding_ratio * 100.0
            ));
        }
        if self.consecutive_decline >= 500 {
            self.instability_flags.push(format!(
                "Epoch {}: B:D < 0.9 for 500+ consecutive epochs", epoch
            ));
        }

        // Sample (always record; caller may downsample on export)
        self.samples.push(EpochSample {
            epoch,
            population,
            mean_fitness,
            hoarding_ratio,
            birth_death_ratio,
            role_entropy,
            total_atp,
            catastrophe_active,
            eco_state: eco_state.to_string(),
        });

        self.transition_detector.push(population, epoch)
    }

    /// Finalize and produce a StressRunResult.
    pub fn finalize(self) -> StressRunResult {
        let n = self.samples.len() as f64;
        let pops: Vec<f64> = self.samples.iter().map(|s| s.population as f64).collect();
        let mean_population = pops.iter().sum::<f64>() / n.max(1.0);
        let population_variance = pops.iter()
            .map(|p| (p - mean_population).powi(2))
            .sum::<f64>() / n.max(1.0);
        let min_birth_death_ratio = self.samples.iter()
            .map(|s| s.birth_death_ratio)
            .fold(f64::INFINITY, f64::min);
        let max_hoarding_ratio = self.samples.iter()
            .map(|s| s.hoarding_ratio)
            .fold(0.0_f64, f64::max);
        let final_role_entropy = self.samples.last()
            .map(|s| s.role_entropy)
            .unwrap_or(0.0);
        let epochs = self.samples.last().map(|s| s.epoch).unwrap_or(0);

        // Downsample to ≤500 entries for export
        let samples = if self.samples.len() > 500 {
            let step = self.samples.len() / 500;
            self.samples.into_iter().step_by(step).collect()
        } else {
            self.samples
        };

        // Deduplicate instability flags
        let mut flags = self.instability_flags;
        flags.dedup();
        // Keep at most last 50 flag entries
        if flags.len() > 50 {
            let start = flags.len() - 50;
            flags = flags[start..].to_vec();
        }

        StressRunResult {
            profile: self.profile_name,
            epochs,
            samples,
            mean_population,
            population_variance,
            extinction_events: self.extinction_events,
            phase_transitions: self.transition_detector.total_transitions,
            min_birth_death_ratio,
            max_hoarding_ratio,
            max_consecutive_decline_epochs: self.max_consecutive_decline,
            final_role_entropy,
            instability_flags: flags,
        }
    }
}

// ─── Role entropy helper ───────────────────────────────────────────────

/// Shannon entropy of the role distribution.
/// H = -Σ p_i * log2(p_i). Returns 0.0 for empty population.
/// Max value for 5 equal roles = log2(5) ≈ 2.32.
pub fn role_entropy(role_counts: &std::collections::HashMap<genesis_dna::AgentRole, usize>) -> f64 {
    let total: usize = role_counts.values().sum();
    if total == 0 { return 0.0; }
    let total_f = total as f64;
    role_counts.values()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / total_f;
            -p * p.log2()
        })
        .sum()
}

// ─── LCG pseudo-random (deterministic, no external deps) ─────────────────

fn lcg_roll(seed: u64) -> f64 {
    // LCG: next = (a * seed + c) mod m
    let next = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    // Shift right 33 yields a 31-bit value; divide by 2^31 to map to [0, 1).
    (next >> 33) as f64 / (1u64 << 31) as f64
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_noop() {
        let cfg = StressConfig::default();
        assert_eq!(cfg.skim_rate_multiplier, 1.0);
        assert_eq!(cfg.treasury_lock_probability, 0.0);
    }

    #[test]
    fn test_profiles_parse() {
        assert!(StressConfig::from_profile("mild").is_some());
        assert!(StressConfig::from_profile("brutal").is_some());
        assert!(StressConfig::from_profile("unknown").is_none());
    }

    #[test]
    fn test_phase_detector_no_fire_on_stable() {
        let mut d = PhaseTransitionDetector::new();
        for i in 0..50usize {
            let result = d.push(50 + (i % 2), i as u64); // tiny oscillation
            assert!(result.is_none(), "Should not fire on trivial variation");
        }
    }

    #[test]
    fn test_phase_detector_fires_on_collapse() {
        let mut d = PhaseTransitionDetector::new();
        // Stable at 50 for 40 epochs
        for i in 0..40usize {
            d.push(50, i as u64);
        }
        // Sudden collapse to 2
        let result = d.push(2, 40);
        assert!(result.is_some(), "Should detect collapse");
        assert!(result.unwrap().contains("PHASE TRANSITION"));
    }

    #[test]
    fn test_role_entropy_uniform() {
        use genesis_dna::AgentRole;
        let mut map = std::collections::HashMap::new();
        for r in [AgentRole::Optimizer, AgentRole::Strategist, AgentRole::Communicator,
                  AgentRole::Archivist, AgentRole::Executor] {
            map.insert(r, 4);
        }
        let h = role_entropy(&map);
        // 5 equal roles → H = log2(5) ≈ 2.322
        assert!((h - 2.322).abs() < 0.01, "Expected H≈2.322, got {}", h);
    }

    #[test]
    fn test_role_entropy_monoculture() {
        use genesis_dna::AgentRole;
        let mut map = std::collections::HashMap::new();
        map.insert(AgentRole::Optimizer, 20);
        let h = role_entropy(&map);
        assert!((h - 0.0).abs() < 0.001, "Monoculture should have H=0");
    }

    #[test]
    fn test_stress_metrics_record_and_finalize() {
        let mut m = StressMetrics::new("test");
        use genesis_dna::AgentRole;
        let mut roles = std::collections::HashMap::new();
        roles.insert(AgentRole::Optimizer, 10);
        roles.insert(AgentRole::Strategist, 10);
        for i in 0..20u64 {
            m.record(i, 20, 0.5, 0.3, 1.05, 1.0, 200.0, false, "Autumn");
        }
        let result = m.finalize();
        assert!((result.mean_population - 20.0).abs() < 0.01);
        assert_eq!(result.extinction_events, 0);
    }

    #[test]
    fn test_treasury_lock_deterministic() {
        let cfg = StressConfig { treasury_lock_probability: 0.5, ..Default::default() };
        // With p=0.5 over 100 different seeds we expect ~40-60 locks
        let locked_count = (0u64..100).filter(|&s| cfg.treasury_locked(s)).count();
        assert!(locked_count > 30 && locked_count < 70,
            "Expected ~50 locks, got {}", locked_count);
    }
}
