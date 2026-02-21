use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Categories of proof that agents can submit for ATP rewards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofKind {
    /// Novel solution to an open problem.
    Solution,
    /// Improvement to an existing solution (differential reward).
    Optimization,
    /// Multi-agent collaboration output.
    Cooperation,
}

/// A solution submitted by an agent for verification and reward.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    /// Human-readable description of the problem solved.
    pub description: String,
    /// The proof kind.
    pub proof_kind: ProofKind,
    /// Raw solution data (code, proof bytes, etc.).
    pub payload: Vec<u8>,
    /// Difficulty estimate (0.0 = trivial, 1.0 = maximum).
    pub difficulty: f64,
    /// Hash of the solution payload for integrity verification.
    pub payload_hash: [u8; 32],
}

impl Solution {
    /// Create a new solution with computed hash.
    pub fn new(
        description: impl Into<String>,
        proof_kind: ProofKind,
        payload: Vec<u8>,
        difficulty: f64,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let hash = hasher.finalize();
        let mut payload_hash = [0u8; 32];
        payload_hash.copy_from_slice(&hash);

        Self {
            description: description.into(),
            proof_kind,
            payload,
            difficulty: difficulty.clamp(0.0, 1.0),
            payload_hash,
        }
    }

    /// Verify payload integrity.
    pub fn verify_integrity(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&self.payload);
        let hash = hasher.finalize();
        hash.as_slice() == self.payload_hash
    }

    /// Evaluate the solution and produce a verdict.
    ///
    /// In a production system this would involve consensus verification.
    /// Here we use a deterministic evaluation based on payload properties.
    pub fn evaluate(&self) -> SolutionVerdict {
        if !self.verify_integrity() {
            return SolutionVerdict {
                accepted: false,
                reward: 0.0,
                quality_score: 0.0,
                reason: "Payload integrity check failed".into(),
            };
        }

        if self.payload.is_empty() {
            return SolutionVerdict {
                accepted: false,
                reward: 0.0,
                quality_score: 0.0,
                reason: "Empty payload".into(),
            };
        }

        // Quality heuristic: entropy of payload bytes (higher = more complex)
        let entropy = self.payload_entropy();
        let quality = entropy.min(1.0);

        // Base reward scales with difficulty and quality
        let base_reward = match self.proof_kind {
            ProofKind::Solution => 10.0,
            ProofKind::Optimization => 5.0,
            ProofKind::Cooperation => 7.5,
        };

        let reward = base_reward * self.difficulty * quality;

        SolutionVerdict {
            accepted: quality > 0.1,
            reward,
            quality_score: quality,
            reason: format!(
                "Evaluated {:?} with difficulty={:.2}, quality={:.2}",
                self.proof_kind, self.difficulty, quality
            ),
        }
    }

    /// Compute Shannon entropy of payload bytes (normalized to [0, 1]).
    fn payload_entropy(&self) -> f64 {
        if self.payload.is_empty() {
            return 0.0;
        }
        let mut freq = [0u64; 256];
        for &b in &self.payload {
            freq[b as usize] += 1;
        }
        let len = self.payload.len() as f64;
        let entropy: f64 = freq
            .iter()
            .filter(|&&f| f > 0)
            .map(|&f| {
                let p = f as f64 / len;
                -p * p.log2()
            })
            .sum();
        entropy / 8.0 // normalize: max entropy for bytes is 8 bits
    }
}

/// Result of evaluating a submitted solution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionVerdict {
    /// Whether the solution was accepted.
    pub accepted: bool,
    /// ATP reward amount (0 if rejected).
    pub reward: f64,
    /// Quality score [0.0, 1.0].
    pub quality_score: f64,
    /// Human-readable evaluation reason.
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solution_integrity() {
        let sol = Solution::new("test", ProofKind::Solution, vec![1, 2, 3, 4], 0.5);
        assert!(sol.verify_integrity());
    }

    #[test]
    fn test_tampered_solution() {
        let mut sol = Solution::new("test", ProofKind::Solution, vec![1, 2, 3, 4], 0.5);
        sol.payload.push(5); // tamper
        assert!(!sol.verify_integrity());
    }

    #[test]
    fn test_evaluate_valid() {
        // Use high-entropy payload
        let payload: Vec<u8> = (0..=255).collect();
        let sol = Solution::new("complex solution", ProofKind::Solution, payload, 0.8);
        let verdict = sol.evaluate();
        assert!(verdict.accepted);
        assert!(verdict.reward > 0.0);
    }

    #[test]
    fn test_evaluate_empty() {
        let sol = Solution::new("empty", ProofKind::Solution, vec![], 0.5);
        let verdict = sol.evaluate();
        assert!(!verdict.accepted);
    }
}
