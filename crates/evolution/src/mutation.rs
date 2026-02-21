use genesis_dna::traits::{TraitKind, TraitVector};
use genesis_dna::AgentID;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::errors::EvolutionError;

/// Record of a mutation that occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationEvent {
    /// Agent whose traits were mutated.
    pub agent_id: AgentID,
    /// Which trait was affected.
    pub trait_kind: TraitKind,
    /// The delta applied.
    pub delta: f64,
    /// Old value before mutation.
    pub old_value: f64,
    /// New value after mutation.
    pub new_value: f64,
    /// Environmental pressure that triggered mutation.
    pub pressure: f64,
}

/// Engine that applies mutations to agent traits based on environmental pressure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationEngine {
    /// Base mutation rate (probability per trait per cycle).
    pub base_rate: f64,
    /// Maximum absolute mutation delta.
    pub max_delta: f64,
    /// History of mutations applied.
    pub history: Vec<MutationEvent>,
}

impl MutationEngine {
    pub fn new(base_rate: f64, max_delta: f64) -> Result<Self, EvolutionError> {
        if !(0.0..=1.0).contains(&base_rate) {
            return Err(EvolutionError::InvalidMutationRate(base_rate));
        }
        Ok(Self {
            base_rate,
            max_delta,
            history: Vec::new(),
        })
    }

    /// Default mutation engine with 1% base rate.
    pub fn default_engine() -> Self {
        Self {
            base_rate: 0.01,
            max_delta: 0.05,
            history: Vec::new(),
        }
    }

    /// Apply environmental pressure to potentially mutate an agent's traits.
    ///
    /// Higher `pressure` (0.0–1.0) increases both mutation probability and magnitude.
    /// Returns the number of traits that were mutated.
    pub fn apply_pressure(
        &mut self,
        agent_id: AgentID,
        traits: &mut TraitVector,
        pressure: f64,
    ) -> u32 {
        let mut rng = rand::thread_rng();
        let effective_rate = self.base_rate + (pressure * 0.1); // pressure boosts mutation
        let mut mutations = 0;

        for kind in TraitKind::all() {
            if rng.gen::<f64>() < effective_rate {
                let old_value = traits.get(*kind).value();
                let delta = rng.gen_range(-self.max_delta..=self.max_delta) * (1.0 + pressure);
                traits.mutate(*kind, delta);
                let new_value = traits.get(*kind).value();

                self.history.push(MutationEvent {
                    agent_id,
                    trait_kind: *kind,
                    delta,
                    old_value,
                    new_value,
                    pressure,
                });
                mutations += 1;
            }
        }

        mutations
    }

    /// Adapt mutation rate based on agent performance.
    ///
    /// Low-performing agents get higher mutation rates (more exploration),
    /// high-performing agents get lower rates (exploitation).
    pub fn adapt_rate(&mut self, fitness: f64) {
        // Inverse relationship: low fitness → high mutation
        self.base_rate = ((1.0 - fitness) * 0.05).clamp(0.001, 0.1);
    }

    /// Get recent mutation events.
    pub fn recent_events(&self, n: usize) -> &[MutationEvent] {
        let start = self.history.len().saturating_sub(n);
        &self.history[start..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genesis_dna::traits::TraitVector;
    use uuid::Uuid;

    #[test]
    fn test_mutation_engine_creation() {
        assert!(MutationEngine::new(0.5, 0.1).is_ok());
        assert!(MutationEngine::new(1.5, 0.1).is_err());
    }

    #[test]
    fn test_high_pressure_causes_mutations() {
        let mut engine = MutationEngine::new(0.99, 0.1).unwrap(); // near-certain mutation
        let mut traits = TraitVector::default_neutral();
        let id = Uuid::new_v4();

        let count = engine.apply_pressure(id, &mut traits, 1.0);
        // With 99% rate + pressure, expect most traits to mutate
        assert!(count > 0);
        assert!(!engine.history.is_empty());
    }

    #[test]
    fn test_adapt_rate() {
        let mut engine = MutationEngine::default_engine();

        engine.adapt_rate(0.1); // low fitness
        assert!(engine.base_rate > 0.01);

        engine.adapt_rate(0.9); // high fitness
        assert!(engine.base_rate < 0.01);
    }
}
