// Genesis Protocol — Skill Profile + Reputation
//
// Deterministic skill derivation from genome bytes.
// Skills are immutable at birth, evolvable through mutation.
// Reputation accumulates through completed contracts.

use serde::{Deserialize, Serialize};

/// Four-axis skill profile derived deterministically from genome.
///
/// Each axis maps to a byte in the genesis hash, giving a value
/// in \[0.0, 1.0\]. Skills are comparable, reproducible, and
/// evolve only through mutation of the underlying genome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProfile {
    /// Raw compute throughput aptitude.
    pub compute: f64,
    /// Solution optimization aptitude.
    pub optimization: f64,
    /// Inter-agent communication aptitude.
    pub communication: f64,
    /// Multi-agent cooperation aptitude.
    pub cooperation: f64,
}

impl SkillProfile {
    /// Derive skills deterministically from the first 4 bytes of a genome hash.
    pub fn from_genome(genome: &[u8; 32]) -> Self {
        Self {
            compute: genome[0] as f64 / 255.0,
            optimization: genome[1] as f64 / 255.0,
            communication: genome[2] as f64 / 255.0,
            cooperation: genome[3] as f64 / 255.0,
        }
    }

    /// Average skill level across all axes.
    pub fn mean(&self) -> f64 {
        (self.compute + self.optimization + self.communication + self.cooperation) / 4.0
    }

    /// Strongest axis value.
    pub fn peak(&self) -> f64 {
        self.compute
            .max(self.optimization)
            .max(self.communication)
            .max(self.cooperation)
    }

    /// Match score against a task requiring specific weights.
    /// `weights` must sum to 1.0 for meaningful results.
    pub fn weighted_score(&self, weights: [f64; 4]) -> f64 {
        self.compute * weights[0]
            + self.optimization * weights[1]
            + self.communication * weights[2]
            + self.cooperation * weights[3]
    }
}

/// Mutable reputation that accumulates over an agent's lifetime.
///
/// Starts neutral (0.5). Grows through endorsements and completed
/// contracts. Decays through failed contracts. This is the only
/// mutable identity component — everything else is genome-derived.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reputation {
    /// Composite score in \[0.0, 1.0\]. Starts at 0.5 (neutral).
    pub score: f64,
    /// Lifetime count of successfully completed contracts.
    pub completed_contracts: u64,
    /// Lifetime count of endorsements received from other agents.
    pub endorsements: u64,
}

impl Reputation {
    pub fn new() -> Self {
        Self {
            score: 0.5,
            completed_contracts: 0,
            endorsements: 0,
        }
    }

    /// Record an endorsement from another agent.
    pub fn endorse(&mut self) {
        self.endorsements += 1;
        self.score = (self.score + 0.01).min(1.0);
    }

    /// Record a completed contract. Quality in \[0.0, 1.0\] is blended
    /// into the score using exponential moving average (alpha = 0.1).
    pub fn complete_contract(&mut self, quality: f64) {
        let quality = quality.clamp(0.0, 1.0);
        self.completed_contracts += 1;
        self.score = self.score * 0.9 + quality * 0.1;
    }

    /// Record a failed contract — penalizes score.
    pub fn fail_contract(&mut self) {
        self.score = (self.score * 0.85).max(0.0);
    }

    /// Discount multiplier for compute costs. Better reputation = cheaper.
    /// Perfect score (1.0) → 0.7 (30% discount). Zero → 1.0 (no discount).
    pub fn compute_discount(&self) -> f64 {
        1.0 - (self.score * 0.3)
    }
}

impl Default for Reputation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_derivation_deterministic() {
        let genome = [0xAB_u8; 32];
        let s1 = SkillProfile::from_genome(&genome);
        let s2 = SkillProfile::from_genome(&genome);
        assert_eq!(s1.compute, s2.compute);
        assert_eq!(s1.optimization, s2.optimization);
        assert_eq!(s1.communication, s2.communication);
        assert_eq!(s1.cooperation, s2.cooperation);
    }

    #[test]
    fn test_skill_range() {
        let low = [0u8; 32];
        let high = [0xFF; 32];
        let sl = SkillProfile::from_genome(&low);
        let sh = SkillProfile::from_genome(&high);
        assert_eq!(sl.compute, 0.0);
        assert_eq!(sh.compute, 1.0);
    }

    #[test]
    fn test_skill_mean_and_peak() {
        let mut genome = [0u8; 32];
        genome[0] = 100;
        genome[1] = 200;
        genome[2] = 50;
        genome[3] = 150;
        let s = SkillProfile::from_genome(&genome);
        assert!((s.mean() - (100.0 + 200.0 + 50.0 + 150.0) / (4.0 * 255.0)).abs() < 0.001);
        assert!((s.peak() - 200.0 / 255.0).abs() < 0.001);
    }

    #[test]
    fn test_reputation_lifecycle() {
        let mut rep = Reputation::new();
        assert!((rep.score - 0.5).abs() < 0.001);

        rep.endorse();
        assert!(rep.score > 0.5);
        assert_eq!(rep.endorsements, 1);

        rep.complete_contract(1.0);
        assert_eq!(rep.completed_contracts, 1);
        let after_good = rep.score;

        rep.fail_contract();
        assert!(rep.score < after_good);
    }

    #[test]
    fn test_reputation_discount() {
        let mut rep = Reputation::new();
        let neutral_discount = rep.compute_discount();
        assert!(neutral_discount < 1.0); // some discount even at 0.5

        // Drive score up
        for _ in 0..100 {
            rep.complete_contract(1.0);
        }
        assert!(rep.compute_discount() < neutral_discount);
    }

    #[test]
    fn test_weighted_score() {
        let mut genome = [0u8; 32];
        genome[0] = 255; // compute = 1.0
        genome[1] = 0;   // optimization = 0.0
        genome[2] = 128; // communication ≈ 0.5
        genome[3] = 64;  // cooperation ≈ 0.25
        let s = SkillProfile::from_genome(&genome);

        // Pure compute task
        let score = s.weighted_score([1.0, 0.0, 0.0, 0.0]);
        assert!((score - 1.0).abs() < 0.001);

        // Pure optimization task
        let score = s.weighted_score([0.0, 1.0, 0.0, 0.0]);
        assert!(score < 0.001);
    }
}
