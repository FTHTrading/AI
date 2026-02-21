use serde::{Deserialize, Serialize};

use crate::errors::DnaError;

/// A single trait value constrained to [0.0, 1.0].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TraitValue(f64);

impl TraitValue {
    pub fn new(value: f64) -> Result<Self, DnaError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(DnaError::TraitOutOfRange {
                name: "unknown".into(),
                value,
            });
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    /// Mutate by a delta, clamping to [0.0, 1.0].
    pub fn mutate(&mut self, delta: f64) {
        self.0 = (self.0 + delta).clamp(0.0, 1.0);
    }
}

/// The four core DNA traits of every agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraitKind {
    /// FLOPs per watt — raw computational efficiency.
    ComputeEfficiency,
    /// Consensus-verified problem-solving quality.
    SolutionQuality,
    /// Code integrity preserved over time / replication.
    ReplicationFidelity,
    /// Willingness and ability to collaborate with other agents.
    CooperationCoefficient,
}

impl TraitKind {
    pub fn all() -> &'static [TraitKind] {
        &[
            TraitKind::ComputeEfficiency,
            TraitKind::SolutionQuality,
            TraitKind::ReplicationFidelity,
            TraitKind::CooperationCoefficient,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            TraitKind::ComputeEfficiency => "compute_efficiency",
            TraitKind::SolutionQuality => "solution_quality",
            TraitKind::ReplicationFidelity => "replication_fidelity",
            TraitKind::CooperationCoefficient => "cooperation_coefficient",
        }
    }
}

/// Vector of all agent traits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraitVector {
    pub compute_efficiency: TraitValue,
    pub solution_quality: TraitValue,
    pub replication_fidelity: TraitValue,
    pub cooperation_coefficient: TraitValue,
}

impl TraitVector {
    /// Create a new trait vector with all values at 0.5 (neutral).
    pub fn default_neutral() -> Self {
        Self {
            compute_efficiency: TraitValue(0.5),
            solution_quality: TraitValue(0.5),
            replication_fidelity: TraitValue(0.5),
            cooperation_coefficient: TraitValue(0.5),
        }
    }

    /// Create from explicit values; returns error if any are out of range.
    pub fn new(
        compute_efficiency: f64,
        solution_quality: f64,
        replication_fidelity: f64,
        cooperation_coefficient: f64,
    ) -> Result<Self, DnaError> {
        Ok(Self {
            compute_efficiency: TraitValue::new(compute_efficiency)
                .map_err(|_| DnaError::TraitOutOfRange {
                    name: "compute_efficiency".into(),
                    value: compute_efficiency,
                })?,
            solution_quality: TraitValue::new(solution_quality)
                .map_err(|_| DnaError::TraitOutOfRange {
                    name: "solution_quality".into(),
                    value: solution_quality,
                })?,
            replication_fidelity: TraitValue::new(replication_fidelity)
                .map_err(|_| DnaError::TraitOutOfRange {
                    name: "replication_fidelity".into(),
                    value: replication_fidelity,
                })?,
            cooperation_coefficient: TraitValue::new(cooperation_coefficient)
                .map_err(|_| DnaError::TraitOutOfRange {
                    name: "cooperation_coefficient".into(),
                    value: cooperation_coefficient,
                })?,
        })
    }

    /// Get trait by kind.
    pub fn get(&self, kind: TraitKind) -> TraitValue {
        match kind {
            TraitKind::ComputeEfficiency => self.compute_efficiency,
            TraitKind::SolutionQuality => self.solution_quality,
            TraitKind::ReplicationFidelity => self.replication_fidelity,
            TraitKind::CooperationCoefficient => self.cooperation_coefficient,
        }
    }

    /// Mutate a specific trait by delta.
    pub fn mutate(&mut self, kind: TraitKind, delta: f64) {
        match kind {
            TraitKind::ComputeEfficiency => self.compute_efficiency.mutate(delta),
            TraitKind::SolutionQuality => self.solution_quality.mutate(delta),
            TraitKind::ReplicationFidelity => self.replication_fidelity.mutate(delta),
            TraitKind::CooperationCoefficient => self.cooperation_coefficient.mutate(delta),
        }
    }

    /// Compute the overall fitness score (weighted average).
    pub fn fitness(&self) -> f64 {
        self.compute_efficiency.value() * 0.25
            + self.solution_quality.value() * 0.30
            + self.replication_fidelity.value() * 0.20
            + self.cooperation_coefficient.value() * 0.25
    }
}

/// Energy metabolism profile — determines ATP generation and consumption rates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnergyProfile {
    /// Base metabolic rate: ATP consumed per time unit just to stay alive.
    pub basal_rate: f64,
    /// Efficiency multiplier for ATP generation from proof-of-work.
    pub generation_efficiency: f64,
    /// Bonus multiplier for "Primordial" agents (first 100).
    pub primordial_bonus: f64,
}

impl EnergyProfile {
    pub fn default_profile() -> Self {
        Self {
            basal_rate: 1.0,
            generation_efficiency: 1.0,
            primordial_bonus: 1.0,
        }
    }

    /// Primordial agents get 1.5x ATP generation rate.
    pub fn primordial() -> Self {
        Self {
            basal_rate: 0.8, // slightly lower metabolism — more efficient
            generation_efficiency: 1.5,
            primordial_bonus: 1.5,
        }
    }

    /// Effective ATP generation rate.
    pub fn effective_generation_rate(&self) -> f64 {
        self.generation_efficiency * self.primordial_bonus
    }
}

/// Derive initial traits from the genesis hash bytes.
/// Uses deterministic extraction: each trait is derived from 8 bytes of the hash.
pub fn traits_from_hash(hash: &[u8; 32]) -> TraitVector {
    let extract = |start: usize| -> f64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash[start..start + 8]);
        let raw = u64::from_le_bytes(bytes);
        (raw as f64) / (u64::MAX as f64)
    };

    TraitVector {
        compute_efficiency: TraitValue(extract(0)),
        solution_quality: TraitValue(extract(8)),
        replication_fidelity: TraitValue(extract(16)),
        cooperation_coefficient: TraitValue(extract(24)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_value_clamping() {
        assert!(TraitValue::new(1.5).is_err());
        assert!(TraitValue::new(-0.1).is_err());
        assert!(TraitValue::new(0.5).is_ok());
    }

    #[test]
    fn test_mutation_clamps() {
        let mut tv = TraitValue(0.9);
        tv.mutate(0.3);
        assert!((tv.value() - 1.0).abs() < f64::EPSILON);

        let mut tv2 = TraitValue(0.1);
        tv2.mutate(-0.5);
        assert!(tv2.value().abs() < f64::EPSILON);
    }

    #[test]
    fn test_fitness_calculation() {
        let tv = TraitVector::new(1.0, 1.0, 1.0, 1.0).unwrap();
        assert!((tv.fitness() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_traits_from_hash_deterministic() {
        let hash = [42u8; 32];
        let t1 = traits_from_hash(&hash);
        let t2 = traits_from_hash(&hash);
        assert_eq!(t1, t2);
    }
}
