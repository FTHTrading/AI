// Adaptive Cortex — The Organism's Self-Regulating Brain
//
// The immune system (immune.rs) is the sensory layer: it detects threats.
// The cortex is the motor layer: it prescribes and applies mutations
// to PressureConfig in response to threats.
//
// Philosophy: the organism's laws of nature are not fixed constants —
// they are adaptive parameters that drift in response to internal health.
// A healthy organism relaxes pressure. A sick organism tightens it.
// But all mutations are bounded, gradual, and reversible.
//
// This is not a control loop. It's homeostasis.
// The organism heals itself — or it doesn't.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::immune::{ImmuneReport, ImmuneEvent, ThreatKind, ThreatLevel};

// ─── Pressure Mutations ─────────────────────────────────────────────────

/// A single proposed mutation to a PressureConfig field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureMutation {
    /// Which field is being mutated.
    pub field: PressureField,
    /// Current value before mutation.
    pub old_value: f64,
    /// Proposed new value after mutation.
    pub new_value: f64,
    /// The threat that triggered this mutation.
    pub trigger: ThreatKind,
    /// Severity of the triggering threat.
    pub severity: ThreatLevel,
    /// Human-readable rationale.
    pub rationale: String,
}

/// Identifiers for PressureConfig fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PressureField {
    SoftCap,
    EntropyCoeff,
    CatastropheBaseProb,
    CatastrophePopScale,
    GiniWealthTaxThreshold,
    GiniWealthTaxRate,
    TreasuryOverflowThreshold,
}

impl PressureField {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SoftCap => "soft_cap",
            Self::EntropyCoeff => "entropy_coeff",
            Self::CatastropheBaseProb => "catastrophe_base_prob",
            Self::CatastrophePopScale => "catastrophe_pop_scale",
            Self::GiniWealthTaxThreshold => "gini_wealth_tax_threshold",
            Self::GiniWealthTaxRate => "gini_wealth_tax_rate",
            Self::TreasuryOverflowThreshold => "treasury_overflow_threshold",
        }
    }
}

/// Complete prescription: the set of mutations the cortex prescribes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureResponse {
    /// Epoch when this response was computed.
    pub epoch: u64,
    /// The immune report that triggered this response.
    pub report_health: ThreatLevel,
    /// Number of active threats in the report.
    pub threat_count: usize,
    /// Individual mutations to apply.
    pub mutations: Vec<PressureMutation>,
    /// Whether any mutations were actually prescribed.
    pub has_mutations: bool,
    /// Timestamp of computation.
    pub computed_at: DateTime<Utc>,
}

impl PressureResponse {
    pub fn empty(epoch: u64) -> Self {
        Self {
            epoch,
            report_health: ThreatLevel::Normal,
            threat_count: 0,
            mutations: Vec::new(),
            has_mutations: false,
            computed_at: Utc::now(),
        }
    }
}

// ─── Adaptive Bounds ────────────────────────────────────────────────────
//
// Every parameter has absolute min/max bounds and a maximum step size
// per adaptation cycle. The organism can drift its laws, but cannot
// make them arbitrarily extreme. This prevents runaway feedback loops.

/// Bounds and step limits for adaptive pressure parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveBounds {
    pub soft_cap: (f64, f64, f64),              // (min, max, max_step)
    pub entropy_coeff: (f64, f64, f64),
    pub catastrophe_base_prob: (f64, f64, f64),
    pub catastrophe_pop_scale: (f64, f64, f64),
    pub gini_threshold: (f64, f64, f64),
    pub gini_rate: (f64, f64, f64),
    pub treasury_overflow: (f64, f64, f64),
}

impl Default for AdaptiveBounds {
    fn default() -> Self {
        Self {
            // Soft cap: 50–500, drift ±10 per cycle
            soft_cap: (50.0, 500.0, 10.0),
            // Entropy coeff: 0.000001–0.001, drift ±20% per cycle
            entropy_coeff: (0.000_001, 0.001, 0.000_005),
            // Catastrophe base prob: 0.0–0.05, drift ±0.001
            catastrophe_base_prob: (0.0, 0.05, 0.001),
            // Catastrophe pop scale: 0.0–0.0001, drift ±0.000005
            catastrophe_pop_scale: (0.0, 0.0001, 0.000_005),
            // Gini threshold: 0.20–0.80, drift ±0.05
            gini_threshold: (0.20, 0.80, 0.05),
            // Gini tax rate: 0.005–0.10, drift ±0.005
            gini_rate: (0.005, 0.10, 0.005),
            // Treasury overflow: 0.20–0.80, drift ±0.05
            treasury_overflow: (0.20, 0.80, 0.05),
        }
    }
}

impl AdaptiveBounds {
    fn bounds_for(&self, field: PressureField) -> (f64, f64, f64) {
        match field {
            PressureField::SoftCap => self.soft_cap,
            PressureField::EntropyCoeff => self.entropy_coeff,
            PressureField::CatastropheBaseProb => self.catastrophe_base_prob,
            PressureField::CatastrophePopScale => self.catastrophe_pop_scale,
            PressureField::GiniWealthTaxThreshold => self.gini_threshold,
            PressureField::GiniWealthTaxRate => self.gini_rate,
            PressureField::TreasuryOverflowThreshold => self.treasury_overflow,
        }
    }

    /// Apply a bounded mutation: clamp delta to max_step, clamp result to [min, max].
    pub fn apply(&self, field: PressureField, current: f64, delta: f64) -> f64 {
        let (min, max, max_step) = self.bounds_for(field);
        let clamped_delta = delta.clamp(-max_step, max_step);
        (current + clamped_delta).clamp(min, max)
    }
}

// ─── Cortex Engine ──────────────────────────────────────────────────────

/// Adaptation interval: how often (in epochs) the cortex runs diagnosis
/// and potentially adjusts pressure. Not every epoch — homeostasis
/// operates on slow timescales.
const DEFAULT_ADAPTATION_INTERVAL: u64 = 25;

/// Cooldown: minimum epochs between mutations to the same field.
/// Prevents oscillation.
const DEFAULT_FIELD_COOLDOWN: u64 = 50;

/// The adaptive cortex — transforms immune reports into pressure mutations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveCortex {
    /// How often the cortex runs (epochs).
    pub interval: u64,
    /// Minimum epochs between mutations to the same field.
    pub field_cooldown: u64,
    /// Bounds for all adaptive parameters.
    pub bounds: AdaptiveBounds,
    /// Last epoch each field was mutated (cooldown tracking).
    pub last_mutation_epoch: HashMap<PressureField, u64>,
    /// History of all pressure responses (last N).
    pub response_history: Vec<PressureResponse>,
    /// Maximum response history to keep.
    pub max_history: usize,
    /// Peak treasury reserve observed (for depletion detection).
    pub peak_treasury: f64,
    /// Whether the cortex is enabled.
    pub enabled: bool,
}

impl Default for AdaptiveCortex {
    fn default() -> Self {
        Self {
            interval: DEFAULT_ADAPTATION_INTERVAL,
            field_cooldown: DEFAULT_FIELD_COOLDOWN,
            bounds: AdaptiveBounds::default(),
            last_mutation_epoch: HashMap::new(),
            response_history: Vec::new(),
            max_history: 100,
            peak_treasury: 0.0,
            enabled: true,
        }
    }
}

impl AdaptiveCortex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Should the cortex run this epoch?
    pub fn should_adapt(&self, epoch: u64) -> bool {
        self.enabled && epoch > 0 && epoch % self.interval == 0
    }

    /// Can a field be mutated this epoch (cooldown check)?
    fn can_mutate(&self, field: PressureField, epoch: u64) -> bool {
        match self.last_mutation_epoch.get(&field) {
            Some(&last) => epoch.saturating_sub(last) >= self.field_cooldown,
            None => true,
        }
    }

    /// Core: analyze an immune report and prescribe pressure mutations.
    ///
    /// Each threat type maps to specific PressureConfig adjustments:
    ///
    /// | Threat                  | Response                                    |
    /// |------------------------|---------------------------------------------|
    /// | PopulationCollapse     | ↑ soft_cap, ↓ entropy_coeff, ↓ catastrophe  |
    /// | MonocultureDominance   | ↑ catastrophe (niche shock), ↑ entropy      |
    /// | AtpOligarchy           | ↓ gini_threshold, ↑ gini_rate               |
    /// | WealthConcentration    | ↓ gini_threshold, ↑ gini_rate               |
    /// | MutationRunaway        | ↓ catastrophe (less culling)                |
    /// | RoleExtinction         | ↑ soft_cap (allow recovery births)          |
    /// | TreasuryDepletion      | ↓ treasury_overflow (accumulate more)       |
    /// | EconomicStagnation     | ↓ entropy_coeff, ↑ treasury_overflow        |
    pub fn prescribe(
        &mut self,
        report: &ImmuneReport,
        current_soft_cap: usize,
        current_entropy_coeff: f64,
        current_catastrophe_base: f64,
        current_catastrophe_scale: f64,
        current_gini_threshold: f64,
        current_gini_rate: f64,
        current_treasury_overflow: f64,
    ) -> PressureResponse {
        let epoch = report.epoch;
        let mut mutations = Vec::new();

        // Only respond to Watch-level or higher threats
        for event in &report.events {
            if event.level == ThreatLevel::Normal {
                continue;
            }

            let severity_scale = match event.level {
                ThreatLevel::Watch => 0.5,
                ThreatLevel::Warning => 1.0,
                ThreatLevel::Critical => 2.0,
                ThreatLevel::Normal => 0.0,
            };

            match event.kind {
                ThreatKind::PopulationCollapse => {
                    // Population dying → ease pressure
                    self.try_mutate(
                        &mut mutations, PressureField::SoftCap,
                        current_soft_cap as f64, 5.0 * severity_scale,
                        event, epoch,
                        "Population collapsing — raising soft cap to allow recovery",
                    );
                    self.try_mutate(
                        &mut mutations, PressureField::EntropyCoeff,
                        current_entropy_coeff, -0.000_002 * severity_scale,
                        event, epoch,
                        "Population collapsing — reducing entropy tax",
                    );
                    self.try_mutate(
                        &mut mutations, PressureField::CatastropheBaseProb,
                        current_catastrophe_base, -0.0005 * severity_scale,
                        event, epoch,
                        "Population collapsing — reducing catastrophe frequency",
                    );
                }

                ThreatKind::MonocultureDominance => {
                    // One role dominates → increase diversity pressure
                    self.try_mutate(
                        &mut mutations, PressureField::CatastropheBaseProb,
                        current_catastrophe_base, 0.0005 * severity_scale,
                        event, epoch,
                        "Monoculture detected — increasing catastrophe frequency for niche shocks",
                    );
                    self.try_mutate(
                        &mut mutations, PressureField::EntropyCoeff,
                        current_entropy_coeff, 0.000_002 * severity_scale,
                        event, epoch,
                        "Monoculture detected — increasing entropy tax to weaken dominant role",
                    );
                }

                ThreatKind::AtpOligarchy | ThreatKind::WealthConcentration => {
                    // Wealth too concentrated → tighten redistribution
                    self.try_mutate(
                        &mut mutations, PressureField::GiniWealthTaxThreshold,
                        current_gini_threshold, -0.02 * severity_scale,
                        event, epoch,
                        "Wealth concentration — lowering Gini threshold to trigger tax sooner",
                    );
                    self.try_mutate(
                        &mut mutations, PressureField::GiniWealthTaxRate,
                        current_gini_rate, 0.002 * severity_scale,
                        event, epoch,
                        "Wealth concentration — increasing Gini tax rate",
                    );
                }

                ThreatKind::MutationRunaway => {
                    // Too many mutations → reduce stochastic culling
                    self.try_mutate(
                        &mut mutations, PressureField::CatastropheBaseProb,
                        current_catastrophe_base, -0.0003 * severity_scale,
                        event, epoch,
                        "Mutation runaway — reducing catastrophe culling",
                    );
                }

                ThreatKind::RoleExtinction => {
                    // A role died out → allow more births for recovery
                    self.try_mutate(
                        &mut mutations, PressureField::SoftCap,
                        current_soft_cap as f64, 8.0 * severity_scale,
                        event, epoch,
                        "Role extinction — raising soft cap for recovery births",
                    );
                }

                ThreatKind::TreasuryDepletion => {
                    // Treasury empty → reduce overflow threshold to accumulate more
                    self.try_mutate(
                        &mut mutations, PressureField::TreasuryOverflowThreshold,
                        current_treasury_overflow, -0.03 * severity_scale,
                        event, epoch,
                        "Treasury depleting — lowering overflow threshold to retain reserves",
                    );
                }

                ThreatKind::EconomicStagnation => {
                    // Economy frozen → reduce entropy tax, release treasury
                    self.try_mutate(
                        &mut mutations, PressureField::EntropyCoeff,
                        current_entropy_coeff, -0.000_003 * severity_scale,
                        event, epoch,
                        "Economy stagnating — reducing entropy tax to increase circulation",
                    );
                    self.try_mutate(
                        &mut mutations, PressureField::TreasuryOverflowThreshold,
                        current_treasury_overflow, 0.03 * severity_scale,
                        event, epoch,
                        "Economy stagnating — raising overflow threshold to release treasury",
                    );
                }
            }
        }

        // If organism is fully healthy, gently drift toward defaults
        // (regression to baseline). 10% of the way back per adaptation cycle.
        if report.is_healthy() && report.threat_count() == 0 {
            self.drift_toward_default(
                &mut mutations, PressureField::SoftCap,
                current_soft_cap as f64, 180.0, epoch,
            );
            self.drift_toward_default(
                &mut mutations, PressureField::EntropyCoeff,
                current_entropy_coeff, 0.00002, epoch,
            );
            self.drift_toward_default(
                &mut mutations, PressureField::CatastropheBaseProb,
                current_catastrophe_base, 0.002, epoch,
            );
            self.drift_toward_default(
                &mut mutations, PressureField::CatastrophePopScale,
                current_catastrophe_scale, 0.00001, epoch,
            );
            self.drift_toward_default(
                &mut mutations, PressureField::GiniWealthTaxThreshold,
                current_gini_threshold, 0.40, epoch,
            );
            self.drift_toward_default(
                &mut mutations, PressureField::GiniWealthTaxRate,
                current_gini_rate, 0.02, epoch,
            );
            self.drift_toward_default(
                &mut mutations, PressureField::TreasuryOverflowThreshold,
                current_treasury_overflow, 0.50, epoch,
            );
        }

        let has_mutations = !mutations.is_empty();
        let response = PressureResponse {
            epoch,
            report_health: report.overall_health,
            threat_count: report.threat_count(),
            mutations,
            has_mutations,
            computed_at: Utc::now(),
        };

        // Record in history
        self.response_history.push(response.clone());
        if self.response_history.len() > self.max_history {
            self.response_history.remove(0);
        }

        response
    }

    /// Attempt to create a bounded mutation. Respects cooldown and bounds.
    fn try_mutate(
        &self,
        mutations: &mut Vec<PressureMutation>,
        field: PressureField,
        current: f64,
        delta: f64,
        event: &ImmuneEvent,
        epoch: u64,
        rationale: &str,
    ) {
        if !self.can_mutate(field, epoch) {
            return;
        }
        // Check if we already proposed a mutation for this field this cycle
        if mutations.iter().any(|m| m.field == field) {
            return;
        }

        let new_value = self.bounds.apply(field, current, delta);

        // Only emit mutation if the value actually changed meaningfully
        let change = (new_value - current).abs();
        let threshold = current.abs() * 0.001; // 0.1% minimum change
        if change < threshold.max(1e-10) {
            return;
        }

        mutations.push(PressureMutation {
            field,
            old_value: current,
            new_value,
            trigger: event.kind,
            severity: event.level,
            rationale: rationale.to_string(),
        });
    }

    /// Gently drift a parameter toward its default value when healthy.
    /// Moves 10% of the distance per cycle, bounded by step limits.
    fn drift_toward_default(
        &self,
        mutations: &mut Vec<PressureMutation>,
        field: PressureField,
        current: f64,
        default: f64,
        epoch: u64,
    ) {
        if !self.can_mutate(field, epoch) {
            return;
        }
        if mutations.iter().any(|m| m.field == field) {
            return;
        }

        let distance = default - current;
        if distance.abs() < current.abs() * 0.005 {
            return; // close enough to default, no drift needed
        }

        let drift = distance * 0.10; // 10% regression toward default
        let new_value = self.bounds.apply(field, current, drift);

        if (new_value - current).abs() < 1e-10 {
            return;
        }

        mutations.push(PressureMutation {
            field,
            old_value: current,
            new_value,
            trigger: ThreatKind::EconomicStagnation, // neutral trigger for drift
            severity: ThreatLevel::Normal,
            rationale: format!(
                "Healthy organism — drifting {} toward default ({:.6} → {:.6})",
                field.name(), current, new_value,
            ),
        });
    }

    /// Record that mutations were applied this epoch (update cooldowns).
    pub fn record_mutations(&mut self, response: &PressureResponse) {
        for mutation in &response.mutations {
            self.last_mutation_epoch.insert(mutation.field, response.epoch);
        }
    }

    /// Update peak treasury tracking.
    pub fn update_peak_treasury(&mut self, reserve: f64) {
        if reserve > self.peak_treasury {
            self.peak_treasury = reserve;
        }
    }

    /// Get the most recent pressure response, if any.
    pub fn last_response(&self) -> Option<&PressureResponse> {
        self.response_history.last()
    }

    /// Total number of mutations applied across all history.
    pub fn total_mutations_applied(&self) -> usize {
        self.response_history.iter()
            .map(|r| r.mutations.len())
            .sum()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::immune;
    use std::collections::HashMap;

    fn healthy_report(epoch: u64) -> ImmuneReport {
        let roles: HashMap<String, usize> = [
            ("Optimizer", 10), ("Strategist", 10), ("Communicator", 10),
            ("Archivist", 10), ("Executor", 10),
        ].iter().map(|(k, v)| (k.to_string(), *v)).collect();
        let balances = vec![50.0; 50];
        let history = vec![50, 50, 50, 50, 50];
        immune::diagnose(
            epoch, &roles, &balances, 5, 50, &history, 5,
            &["Optimizer", "Strategist", "Communicator", "Archivist", "Executor"],
            500.0, 500.0, 500.0, 2500.0,
        )
    }

    fn collapsing_report(epoch: u64) -> ImmuneReport {
        // Balanced roles so we don't also trigger MonocultureDominance
        let roles: HashMap<String, usize> = [
            ("Optimizer", 2), ("Strategist", 2), ("Communicator", 2),
            ("Archivist", 1), ("Executor", 1),
        ].iter().map(|(k, v)| (k.to_string(), *v)).collect();
        let balances = vec![5.0; 8];
        let history = vec![100, 80, 50, 20, 8];
        immune::diagnose(
            epoch, &roles, &balances, 2, 8, &history, 5,
            &["Optimizer", "Strategist", "Communicator", "Archivist", "Executor"],
            5.0, 200.0, 10.0, 40.0,
        )
    }

    fn oligarchy_report(epoch: u64) -> ImmuneReport {
        let roles: HashMap<String, usize> = [
            ("Optimizer", 40), ("Strategist", 40), ("Communicator", 40),
            ("Archivist", 40), ("Executor", 40),
        ].iter().map(|(k, v)| (k.to_string(), *v)).collect();
        let mut balances = vec![1.0; 200];
        balances[0] = 50000.0; // mega-whale
        balances[1] = 30000.0;
        let history = vec![200, 200, 200, 200, 200];
        immune::diagnose(
            epoch, &roles, &balances, 10, 200, &history, 5,
            &["Optimizer", "Strategist", "Communicator", "Archivist", "Executor"],
            500.0, 500.0, 5000.0, 80200.0,
        )
    }

    #[test]
    fn test_healthy_organism_no_mutations() {
        let mut cortex = AdaptiveCortex::new();
        let report = healthy_report(100);
        let response = cortex.prescribe(
            &report, 180, 0.00002, 0.002, 0.00001, 0.40, 0.02, 0.50,
        );
        // Healthy organism at default values → no mutations needed
        assert!(!response.has_mutations);
        assert_eq!(response.mutations.len(), 0);
    }

    #[test]
    fn test_collapse_eases_pressure() {
        let mut cortex = AdaptiveCortex::new();
        let report = collapsing_report(100);
        let response = cortex.prescribe(
            &report, 180, 0.00002, 0.002, 0.00001, 0.40, 0.02, 0.50,
        );
        assert!(response.has_mutations, "Expected mutations for collapsing organism");

        // Should raise soft cap
        let cap_mutation = response.mutations.iter()
            .find(|m| m.field == PressureField::SoftCap);
        if let Some(m) = cap_mutation {
            assert!(m.new_value > 180.0, "Soft cap should increase during collapse");
        }

        // Should reduce entropy coefficient
        let entropy_mutation = response.mutations.iter()
            .find(|m| m.field == PressureField::EntropyCoeff);
        if let Some(m) = entropy_mutation {
            assert!(m.new_value < 0.00002, "Entropy coeff should decrease during collapse");
        }
    }

    #[test]
    fn test_oligarchy_tightens_gini() {
        let mut cortex = AdaptiveCortex::new();
        let report = oligarchy_report(100);
        let response = cortex.prescribe(
            &report, 180, 0.00002, 0.002, 0.00001, 0.40, 0.02, 0.50,
        );
        assert!(response.has_mutations, "Expected mutations for oligarchy");

        // Should lower Gini threshold or raise Gini rate
        let gini_mutations: Vec<_> = response.mutations.iter()
            .filter(|m| matches!(m.field,
                PressureField::GiniWealthTaxThreshold | PressureField::GiniWealthTaxRate
            ))
            .collect();
        assert!(!gini_mutations.is_empty(), "Expected Gini-related mutations");
    }

    #[test]
    fn test_cooldown_prevents_rapid_mutation() {
        let mut cortex = AdaptiveCortex::new();
        cortex.field_cooldown = 50;

        let report = collapsing_report(100);
        let response1 = cortex.prescribe(
            &report, 180, 0.00002, 0.002, 0.00001, 0.40, 0.02, 0.50,
        );
        cortex.record_mutations(&response1);

        // Same report 10 epochs later → cooldown prevents mutations
        let report2 = collapsing_report(110);
        let response2 = cortex.prescribe(
            &report2, 190, 0.000018, 0.0015, 0.00001, 0.40, 0.02, 0.50,
        );
        // Fields that were mutated at epoch 100 should be blocked at epoch 110
        for m in &response2.mutations {
            let was_mutated = response1.mutations.iter().any(|m1| m1.field == m.field);
            assert!(!was_mutated,
                "Field {:?} was mutated at 100, should be on cooldown at 110", m.field);
        }
    }

    #[test]
    fn test_bounds_clamp_extreme_values() {
        let bounds = AdaptiveBounds::default();

        // Trying to set soft_cap below minimum
        let result = bounds.apply(PressureField::SoftCap, 60.0, -100.0);
        assert!(result >= 50.0, "Soft cap should not go below 50");

        // Trying to set entropy_coeff above maximum
        let result = bounds.apply(PressureField::EntropyCoeff, 0.0009, 0.01);
        assert!(result <= 0.001, "Entropy coeff should not exceed 0.001");

        // Step size clamping
        let result = bounds.apply(PressureField::SoftCap, 180.0, 100.0);
        assert!(result <= 190.0, "Step should be clamped to max_step=10");
    }

    #[test]
    fn test_healthy_drift_toward_default() {
        let mut cortex = AdaptiveCortex::new();
        let report = healthy_report(100);

        // Start with non-default values
        let response = cortex.prescribe(
            &report, 220, 0.00005, 0.005, 0.00003, 0.30, 0.05, 0.60,
        );

        // Should see drift mutations toward defaults
        assert!(response.has_mutations, "Expected drift mutations for healthy organism with non-default values");

        for m in &response.mutations {
            let distance_before = (m.old_value - m.new_value).abs();
            assert!(distance_before > 0.0, "Drift should change value");
        }
    }

    #[test]
    fn test_response_history_tracked() {
        let mut cortex = AdaptiveCortex::new();
        cortex.max_history = 3;

        for i in 0..5 {
            let report = healthy_report(i * 25);
            let _ = cortex.prescribe(
                &report, 180, 0.00002, 0.002, 0.00001, 0.40, 0.02, 0.50,
            );
        }

        assert_eq!(cortex.response_history.len(), 3, "History should be capped at max_history");
    }
}
