// Divergence — Measuring how civilizations have separated
//
// Given two worlds (possibly forked from a common ancestor),
// compute quantitative divergence metrics:
//   - Population trajectories
//   - Pressure parameter drift
//   - Evolution event frequency
//   - Ecological state differences
//   - Fitness distribution gaps
//
// Divergence is the empirical proof that initial conditions matter.
// Two worlds with identical seeds but different physics will diverge.
// Two worlds forked from the same ancestor will diverge.
// The divergence report measures *how much* and *in what dimensions*.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use gateway::world::World;
use crate::identity::WorldIdentity;

/// Quantitative divergence report between two worlds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceReport {
    /// World A identifier.
    pub world_a_id: Uuid,
    pub world_a_name: String,
    /// World B identifier.
    pub world_b_id: Uuid,
    pub world_b_name: String,
    /// Whether these worlds share a common ancestor.
    pub common_ancestor: bool,
    /// Epoch at which they diverged (fork point), if applicable.
    pub divergence_epoch: Option<u64>,
    /// Current epoch of world A.
    pub epoch_a: u64,
    /// Current epoch of world B.
    pub epoch_b: u64,
    /// Population divergence (A - B as fraction of average).
    pub population_divergence: f64,
    /// Per-field pressure parameter divergence.
    pub pressure_divergence: HashMap<String, PressureDelta>,
    /// Evolution event counts.
    pub evolution_events_a: u64,
    pub evolution_events_b: u64,
    /// Total mutations applied.
    pub total_mutations_a: u64,
    pub total_mutations_b: u64,
    /// Fitness divergence.
    pub mean_fitness_a: f64,
    pub mean_fitness_b: f64,
    /// Treasury divergence.
    pub treasury_a: f64,
    pub treasury_b: f64,
    /// Ecological state.
    pub eco_state_a: String,
    pub eco_state_b: String,
    /// Overall divergence score (0 = identical, higher = more diverged).
    pub divergence_score: f64,
}

/// Delta for a single pressure parameter between two worlds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureDelta {
    pub value_a: f64,
    pub value_b: f64,
    pub absolute_delta: f64,
    pub relative_delta: f64,
}

impl PressureDelta {
    fn new(a: f64, b: f64) -> Self {
        let abs_d = (a - b).abs();
        let avg = ((a + b) / 2.0).abs().max(1e-12);
        Self {
            value_a: a,
            value_b: b,
            absolute_delta: abs_d,
            relative_delta: abs_d / avg,
        }
    }
}

/// Compute a divergence report between two worlds.
pub fn measure(
    id_a: &WorldIdentity,
    world_a: &World,
    id_b: &WorldIdentity,
    world_b: &World,
) -> DivergenceReport {
    // Determine common ancestry
    let common_ancestor = id_a.parent_id == Some(id_b.id)
        || id_b.parent_id == Some(id_a.id)
        || (id_a.parent_id.is_some() && id_a.parent_id == id_b.parent_id);

    let divergence_epoch = if id_b.parent_id == Some(id_a.id) {
        id_b.fork_epoch
    } else if id_a.parent_id == Some(id_b.id) {
        id_a.fork_epoch
    } else {
        None
    };

    let pop_a = world_a.agents.len() as f64;
    let pop_b = world_b.agents.len() as f64;
    let avg_pop = ((pop_a + pop_b) / 2.0).max(1.0);
    let population_divergence = (pop_a - pop_b).abs() / avg_pop;

    // Pressure parameter divergence
    let mut pressure_divergence = HashMap::new();
    pressure_divergence.insert(
        "soft_cap".into(),
        PressureDelta::new(world_a.pressure.soft_cap as f64, world_b.pressure.soft_cap as f64),
    );
    pressure_divergence.insert(
        "entropy_coeff".into(),
        PressureDelta::new(world_a.pressure.entropy_coeff, world_b.pressure.entropy_coeff),
    );
    pressure_divergence.insert(
        "catastrophe_base_prob".into(),
        PressureDelta::new(world_a.pressure.catastrophe_base_prob, world_b.pressure.catastrophe_base_prob),
    );
    pressure_divergence.insert(
        "catastrophe_pop_scale".into(),
        PressureDelta::new(world_a.pressure.catastrophe_pop_scale, world_b.pressure.catastrophe_pop_scale),
    );
    pressure_divergence.insert(
        "gini_wealth_tax_threshold".into(),
        PressureDelta::new(world_a.pressure.gini_wealth_tax_threshold, world_b.pressure.gini_wealth_tax_threshold),
    );
    pressure_divergence.insert(
        "gini_wealth_tax_rate".into(),
        PressureDelta::new(world_a.pressure.gini_wealth_tax_rate, world_b.pressure.gini_wealth_tax_rate),
    );
    pressure_divergence.insert(
        "treasury_overflow_threshold".into(),
        PressureDelta::new(world_a.pressure.treasury_overflow_threshold, world_b.pressure.treasury_overflow_threshold),
    );

    // Mean fitness
    let mean_fitness_a = if world_a.agents.is_empty() {
        0.0
    } else {
        world_a.agents.iter().map(|a| a.fitness()).sum::<f64>() / world_a.agents.len() as f64
    };
    let mean_fitness_b = if world_b.agents.is_empty() {
        0.0
    } else {
        world_b.agents.iter().map(|a| a.fitness()).sum::<f64>() / world_b.agents.len() as f64
    };

    // Evolution events
    let evolution_events_a = world_a.evolution_engine.total_events;
    let evolution_events_b = world_b.evolution_engine.total_events;
    let total_mutations_a = world_a.evolution_engine.total_mutations;
    let total_mutations_b = world_b.evolution_engine.total_mutations;

    // Composite divergence score: weighted sum of normalized deltas
    let pressure_score: f64 = pressure_divergence.values()
        .map(|d| d.relative_delta.min(2.0))  // cap at 200%
        .sum::<f64>() / pressure_divergence.len() as f64;

    let fitness_score = (mean_fitness_a - mean_fitness_b).abs()
        / ((mean_fitness_a + mean_fitness_b) / 2.0).max(0.01);

    let evolution_score = {
        let total = (evolution_events_a + evolution_events_b) as f64;
        if total > 0.0 {
            (evolution_events_a as f64 - evolution_events_b as f64).abs() / total
        } else {
            0.0
        }
    };

    let divergence_score = population_divergence * 0.3
        + pressure_score * 0.3
        + fitness_score * 0.2
        + evolution_score * 0.2;

    DivergenceReport {
        world_a_id: id_a.id,
        world_a_name: id_a.name.clone(),
        world_b_id: id_b.id,
        world_b_name: id_b.name.clone(),
        common_ancestor,
        divergence_epoch,
        epoch_a: world_a.epoch,
        epoch_b: world_b.epoch,
        population_divergence,
        pressure_divergence,
        evolution_events_a,
        evolution_events_b,
        total_mutations_a,
        total_mutations_b,
        mean_fitness_a,
        mean_fitness_b,
        treasury_a: world_a.treasury.reserve,
        treasury_b: world_b.treasury.reserve,
        eco_state_a: world_a.eco_state.name().to_string(),
        eco_state_b: world_b.eco_state.name().to_string(),
        divergence_score,
    }
}

impl DivergenceReport {
    /// Human-readable summary of divergence.
    pub fn summary(&self) -> String {
        let ancestry = if self.common_ancestor {
            format!("Related (diverged at epoch {})", self.divergence_epoch.unwrap_or(0))
        } else {
            "Independent origin".to_string()
        };

        format!(
            "Divergence: {} vs {} | Score: {:.3} | {} | Pop: {:.1}% | Pressure: {:.3} avg drift",
            self.world_a_name,
            self.world_b_name,
            self.divergence_score,
            ancestry,
            self.population_divergence * 100.0,
            self.pressure_divergence.values()
                .map(|d| d.relative_delta)
                .sum::<f64>() / self.pressure_divergence.len().max(1) as f64,
        )
    }

    /// Is this divergence significant (score > threshold)?
    pub fn is_significant(&self, threshold: f64) -> bool {
        self.divergence_score > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_worlds_zero_divergence() {
        let id_a = WorldIdentity::primordial("A", 1);
        let id_b = WorldIdentity::primordial("B", 2);
        let world_a = World::new();
        let world_b = World::new();

        let report = measure(&id_a, &world_a, &id_b, &world_b);
        // Same physics → near-zero divergence (small fitness variance from different UUIDs)
        assert!(report.divergence_score < 0.10, "score: {}", report.divergence_score);
        assert!(!report.common_ancestor);
    }

    #[test]
    fn pressure_delta_calculation() {
        let d = PressureDelta::new(180.0, 80.0);
        assert_eq!(d.absolute_delta, 100.0);
        // relative = 100 / 130 ≈ 0.769
        assert!((d.relative_delta - 100.0 / 130.0).abs() < 0.01);
    }

    #[test]
    fn divergence_summary_readable() {
        let id_a = WorldIdentity::primordial("Earth-Prime", 1);
        let id_b = WorldIdentity::primordial("High Gravity", 2);
        let world_a = World::new();
        let world_b = World::new();

        let report = measure(&id_a, &world_a, &id_b, &world_b);
        let s = report.summary();
        assert!(s.contains("Earth-Prime"));
        assert!(s.contains("High Gravity"));
    }
}
