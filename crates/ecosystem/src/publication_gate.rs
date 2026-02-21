// Genesis Protocol — Publication Gate
//
// Strategic discipline layer. The single biggest competitive edge.
// Most agents publish everything. Genesis agents self-filter.
//
// Before any external-facing publication, the gate checks:
//   - Confidence threshold (is this worth saying?)
//   - ATP risk (can we afford it?)
//   - Reputation floor (are we credible enough to speak?)
//
// Silence is better than dilution.

use serde::{Deserialize, Serialize};

/// Gate that filters agent output before external publication.
///
/// Agents that pass all three thresholds are cleared to publish.
/// Agents that fail any threshold stay silent — preserving ATP
/// and reputation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationGate {
    /// Minimum internal confidence score to approve publication.
    pub min_confidence: f64,
    /// Maximum ATP the agent is willing to risk on this post.
    pub max_atp_risk: f64,
    /// Minimum reputation required to publish.
    pub min_reputation: f64,
}

/// Outcome of a gate decision — approved or rejected with reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateDecision {
    /// Publication approved — all thresholds passed.
    Approved,
    /// Rejected: confidence too low.
    InsufficientConfidence,
    /// Rejected: ATP cost exceeds risk tolerance.
    ExcessiveATPRisk,
    /// Rejected: reputation below threshold.
    InsufficientReputation,
}

impl PublicationGate {
    /// Create a gate with the given thresholds.
    pub fn new(min_confidence: f64, max_atp_risk: f64, min_reputation: f64) -> Self {
        Self {
            min_confidence,
            max_atp_risk,
            min_reputation,
        }
    }

    /// Default conservative gate — appropriate for early deployment.
    pub fn conservative() -> Self {
        Self {
            min_confidence: 0.7,
            max_atp_risk: 2.0,
            min_reputation: 0.4,
        }
    }

    /// Permissive gate — lower thresholds for internal communication.
    pub fn permissive() -> Self {
        Self {
            min_confidence: 0.3,
            max_atp_risk: 10.0,
            min_reputation: 0.1,
        }
    }

    /// Evaluate whether a publication should be approved.
    ///
    /// Returns true only if ALL thresholds are met.
    pub fn approve(&self, confidence: f64, atp_cost: f64, reputation: f64) -> bool {
        confidence >= self.min_confidence
            && atp_cost <= self.max_atp_risk
            && reputation >= self.min_reputation
    }

    /// Evaluate with detailed decision — tells you WHY it was rejected.
    pub fn evaluate(&self, confidence: f64, atp_cost: f64, reputation: f64) -> GateDecision {
        if confidence < self.min_confidence {
            GateDecision::InsufficientConfidence
        } else if atp_cost > self.max_atp_risk {
            GateDecision::ExcessiveATPRisk
        } else if reputation < self.min_reputation {
            GateDecision::InsufficientReputation
        } else {
            GateDecision::Approved
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approve_all_pass() {
        let gate = PublicationGate::new(0.5, 3.0, 0.3);
        assert!(gate.approve(0.8, 2.0, 0.5));
    }

    #[test]
    fn test_reject_low_confidence() {
        let gate = PublicationGate::new(0.5, 3.0, 0.3);
        assert!(!gate.approve(0.3, 2.0, 0.5));
        assert_eq!(
            gate.evaluate(0.3, 2.0, 0.5),
            GateDecision::InsufficientConfidence,
        );
    }

    #[test]
    fn test_reject_excessive_atp() {
        let gate = PublicationGate::new(0.5, 3.0, 0.3);
        assert!(!gate.approve(0.8, 5.0, 0.5));
        assert_eq!(
            gate.evaluate(0.8, 5.0, 0.5),
            GateDecision::ExcessiveATPRisk,
        );
    }

    #[test]
    fn test_reject_low_reputation() {
        let gate = PublicationGate::new(0.5, 3.0, 0.3);
        assert!(!gate.approve(0.8, 2.0, 0.1));
        assert_eq!(
            gate.evaluate(0.8, 2.0, 0.1),
            GateDecision::InsufficientReputation,
        );
    }

    #[test]
    fn test_conservative_gate() {
        let gate = PublicationGate::conservative();
        // High-quality post: approved
        assert!(gate.approve(0.9, 1.0, 0.6));
        // Low-quality post: rejected
        assert!(!gate.approve(0.5, 1.0, 0.6));
    }

    #[test]
    fn test_boundary_values() {
        let gate = PublicationGate::new(0.5, 3.0, 0.3);
        // Exact boundary — should pass (>=, <=, >=)
        assert!(gate.approve(0.5, 3.0, 0.3));
        assert_eq!(gate.evaluate(0.5, 3.0, 0.3), GateDecision::Approved);
    }
}
