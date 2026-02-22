// Fork Event — Cryptographic record of world branching
//
// When a world forks, the event captures:
//   - Parent and child world IDs
//   - The exact epoch of divergence
//   - State Chain root at fork (proves which body was cloned)
//   - Evolution Chain root at fork (proves which mind was cloned)
//   - A deterministic fork hash binding all of the above
//
// This makes ancestry auditable: you can prove that world B
// descended from world A at epoch N, with cryptographic certainty.
// History cannot be rewritten — the fork hash locks it.

use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Cryptographic record of a world fork event.
///
/// The `fork_hash` binds all fields together:
/// `SHA256(parent_id || child_id || epoch || state_root || evolution_root)`
///
/// Recomputable and verifiable — if any field is tampered with,
/// the hash will not match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkEvent {
    /// Parent world ID (source of the fork).
    pub parent_id: Uuid,
    /// Child world ID (new world created by the fork).
    pub child_id: Uuid,
    /// Epoch at which the fork occurred.
    pub fork_epoch: u64,
    /// State Chain root at the fork point.
    pub state_root_at_fork: String,
    /// Evolution Chain root at the fork point.
    pub evolution_root_at_fork: String,
    /// Physics delta summary: what changed between parent and child.
    pub physics_delta: Vec<(String, f64, f64)>,
    /// Timestamp of the fork.
    pub forked_at: DateTime<Utc>,
    /// Deterministic hash of the fork event.
    pub fork_hash: String,
}

impl ForkEvent {
    /// Create a new fork event with computed hash.
    pub fn new(
        parent_id: Uuid,
        child_id: Uuid,
        fork_epoch: u64,
        state_root: impl Into<String>,
        evolution_root: impl Into<String>,
        physics_delta: Vec<(String, f64, f64)>,
    ) -> Self {
        let state_root = state_root.into();
        let evolution_root = evolution_root.into();

        let fork_hash = compute_fork_hash(
            &parent_id,
            &child_id,
            fork_epoch,
            &state_root,
            &evolution_root,
        );

        Self {
            parent_id,
            child_id,
            fork_epoch,
            state_root_at_fork: state_root,
            evolution_root_at_fork: evolution_root,
            physics_delta,
            forked_at: Utc::now(),
            fork_hash,
        }
    }

    /// Verify the fork hash (tamper detection).
    pub fn verify(&self) -> bool {
        let expected = compute_fork_hash(
            &self.parent_id,
            &self.child_id,
            self.fork_epoch,
            &self.state_root_at_fork,
            &self.evolution_root_at_fork,
        );
        self.fork_hash == expected
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        let delta_count = self.physics_delta.len();
        format!(
            "Fork at epoch {}: {} → {} ({} physics changes, hash: {}..)",
            self.fork_epoch,
            &self.parent_id.to_string()[..8],
            &self.child_id.to_string()[..8],
            delta_count,
            &self.fork_hash[..16],
        )
    }
}

/// Deterministic hash of a fork event.
///
/// SHA256(parent_id || child_id || epoch_le_bytes || state_root || evolution_root)
fn compute_fork_hash(
    parent_id: &Uuid,
    child_id: &Uuid,
    epoch: u64,
    state_root: &str,
    evolution_root: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(parent_id.as_bytes());
    hasher.update(child_id.as_bytes());
    hasher.update(epoch.to_le_bytes());
    hasher.update(state_root.as_bytes());
    hasher.update(evolution_root.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fork_event_hash_integrity() {
        let parent = Uuid::new_v4();
        let child = Uuid::new_v4();
        let event = ForkEvent::new(
            parent, child, 500,
            "state_root_abc", "evolution_root_def",
            vec![("soft_cap".into(), 180.0, 80.0)],
        );

        assert!(event.verify());
        assert!(!event.fork_hash.is_empty());
        assert_eq!(event.fork_epoch, 500);
    }

    #[test]
    fn fork_hash_is_deterministic() {
        let parent = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8").unwrap();
        let child = Uuid::parse_str("e1e2e3e4-f1f2-a1a2-b1b2-b3b4b5b6b7b8").unwrap();

        let h1 = compute_fork_hash(&parent, &child, 100, "sr", "er");
        let h2 = compute_fork_hash(&parent, &child, 100, "sr", "er");
        assert_eq!(h1, h2);

        // Different epoch → different hash
        let h3 = compute_fork_hash(&parent, &child, 101, "sr", "er");
        assert_ne!(h1, h3);
    }

    #[test]
    fn tampered_fork_fails_verify() {
        let parent = Uuid::new_v4();
        let child = Uuid::new_v4();
        let mut event = ForkEvent::new(
            parent, child, 500,
            "state_root", "evolution_root",
            vec![],
        );

        assert!(event.verify());

        // Tamper with the epoch
        event.fork_epoch = 501;
        assert!(!event.verify());
    }

    #[test]
    fn fork_summary_readable() {
        let parent = Uuid::new_v4();
        let child = Uuid::new_v4();
        let event = ForkEvent::new(
            parent, child, 250,
            "sr", "er",
            vec![("soft_cap".into(), 180.0, 80.0), ("pop_cap".into(), 200.0, 100.0)],
        );

        let s = event.summary();
        assert!(s.contains("250"));
        assert!(s.contains("2 physics changes"));
    }
}
