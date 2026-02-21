use genesis_dna::AgentDNA;
use serde::{Deserialize, Serialize};

use crate::errors::EvolutionError;

/// Minimum fitness threshold for replication eligibility.
pub const REPLICATION_FITNESS_THRESHOLD: f64 = 0.4;

/// Minimum population for selection to operate.
pub const MIN_POPULATION_SIZE: usize = 2;

/// Outcome of a natural selection cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionOutcome {
    /// Agents eligible for replication (above fitness threshold).
    pub replicators: Vec<uuid::Uuid>,
    /// Agents entering stasis (negative ATP / low fitness).
    pub stasis_candidates: Vec<uuid::Uuid>,
    /// Agents terminated (prolonged stasis).
    pub terminated: Vec<uuid::Uuid>,
    /// Agents in the bottom fitness percentile (receive fitness penalty).
    pub unfit: Vec<uuid::Uuid>,
    /// Average population fitness.
    pub mean_fitness: f64,
    /// Best agent fitness.
    pub max_fitness: f64,
    /// Worst agent fitness.
    pub min_fitness: f64,
}

/// Engine for natural selection: evaluate population fitness, cull weak agents,
/// and identify replication candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionEngine {
    /// Fitness threshold for replication.
    pub replication_threshold: f64,
    /// Number of stasis cycles before termination.
    pub max_stasis_cycles: u32,
    /// Per-agent stasis cycle counters.
    stasis_counters: std::collections::HashMap<uuid::Uuid, u32>,
}

impl SelectionEngine {
    pub fn new() -> Self {
        Self {
            replication_threshold: REPLICATION_FITNESS_THRESHOLD,
            max_stasis_cycles: 3,
            stasis_counters: std::collections::HashMap::new(),
        }
    }

    /// Run a selection cycle on the population.
    ///
    /// Takes a slice of `(AgentDNA, atp_balance, is_in_stasis)` tuples.
    pub fn select(
        &mut self,
        population: &[(AgentDNA, f64, bool)],
    ) -> Result<SelectionOutcome, EvolutionError> {
        if population.len() < MIN_POPULATION_SIZE {
            return Err(EvolutionError::PopulationTooSmall {
                count: population.len(),
                min: MIN_POPULATION_SIZE,
            });
        }

        let mut replicators = Vec::new();
        let mut stasis_candidates = Vec::new();
        let mut terminated = Vec::new();
        let mut fitness_sum = 0.0;
        let mut max_fitness = f64::NEG_INFINITY;
        let mut min_fitness = f64::INFINITY;

        // Collect (id, fitness) for bottom-percentile culling
        let mut fitness_list: Vec<(uuid::Uuid, f64)> = Vec::with_capacity(population.len());

        for (dna, _atp_balance, is_in_stasis) in population {
            let fitness = dna.fitness();
            fitness_sum += fitness;
            fitness_list.push((dna.id, fitness));

            if fitness > max_fitness {
                max_fitness = fitness;
            }
            if fitness < min_fitness {
                min_fitness = fitness;
            }

            if *is_in_stasis {
                let counter = self.stasis_counters.entry(dna.id).or_insert(0);
                *counter += 1;
                if *counter >= self.max_stasis_cycles {
                    terminated.push(dna.id);
                    self.stasis_counters.remove(&dna.id);
                } else {
                    stasis_candidates.push(dna.id);
                }
            } else {
                // Reset stasis counter if agent recovers
                self.stasis_counters.remove(&dna.id);

                if fitness >= self.replication_threshold {
                    replicators.push(dna.id);
                }
            }
        }

        // Identify bottom 10% by fitness for starvation tax
        fitness_list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let bottom_n = (population.len() as f64 * 0.10).ceil() as usize;
        let unfit: Vec<uuid::Uuid> = fitness_list.iter()
            .take(bottom_n)
            .filter(|(id, _)| !terminated.contains(id))
            .map(|(id, _)| *id)
            .collect();

        let mean_fitness = fitness_sum / population.len() as f64;

        Ok(SelectionOutcome {
            replicators,
            stasis_candidates,
            terminated,
            unfit,
            mean_fitness,
            max_fitness,
            min_fitness,
        })
    }

    /// Check if an agent is eligible for replication.
    pub fn can_replicate(&self, dna: &AgentDNA, atp_balance: f64) -> Result<(), EvolutionError> {
        let fitness = dna.fitness();
        if fitness < self.replication_threshold {
            return Err(EvolutionError::IneligibleForReplication {
                fitness,
                threshold: self.replication_threshold,
            });
        }
        if atp_balance < metabolism::atp::costs::REPLICATION {
            return Err(EvolutionError::IneligibleForReplication {
                fitness,
                threshold: self.replication_threshold,
            });
        }
        Ok(())
    }
}

impl Default for SelectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genesis_dna::AgentDNA;

    fn make_agent(entropy_byte: u8) -> AgentDNA {
        AgentDNA::from_entropy(&[entropy_byte; 64], false).unwrap()
    }

    #[test]
    fn test_selection_cycle() {
        let mut engine = SelectionEngine::new();
        let agents: Vec<(AgentDNA, f64, bool)> = (0..5)
            .map(|i| {
                let dna = make_agent(i as u8 + 1);
                (dna, 100.0, false)
            })
            .collect();

        let outcome = engine.select(&agents).unwrap();
        assert!(outcome.mean_fitness > 0.0);
        assert!(!outcome.replicators.is_empty() || outcome.mean_fitness < REPLICATION_FITNESS_THRESHOLD);
    }

    #[test]
    fn test_stasis_termination() {
        let mut engine = SelectionEngine::new();
        engine.max_stasis_cycles = 2;

        let dna = make_agent(0xAA);
        let agents = vec![
            (dna.clone(), 0.0, true),
            (make_agent(0xBB), 100.0, false),
        ];

        // Cycle 1: stasis
        let outcome = engine.select(&agents).unwrap();
        assert!(outcome.stasis_candidates.contains(&dna.id));
        assert!(!outcome.terminated.contains(&dna.id));

        // Cycle 2: terminated
        let outcome = engine.select(&agents).unwrap();
        assert!(outcome.terminated.contains(&dna.id));
    }

    #[test]
    fn test_population_too_small() {
        let mut engine = SelectionEngine::new();
        let agents = vec![(make_agent(1), 100.0, false)];
        assert!(engine.select(&agents).is_err());
    }
}
