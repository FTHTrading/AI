// World Identity — Cryptographic lineage of a civilization
//
// Every world has a UUID, a name, and a lineage record.
// Primordial worlds have no parent. Forked worlds carry the
// exact State Chain root and Evolution Chain root at their
// point of divergence, making ancestry cryptographically provable.

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Unique identity for a world within the multiverse.
///
/// Lineage fields (parent_id, fork_epoch, fork_state_root, fork_evolution_root)
/// are `None` for primordial worlds and populated for forked worlds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldIdentity {
    /// Unique identifier for this world.
    pub id: Uuid,
    /// Human-readable name (e.g. "Earth-Prime", "High Gravity", "Volcanic").
    pub name: String,
    /// Deterministic seed used for this world's RNG.
    pub seed: u64,
    /// When this world was created.
    pub created_at: DateTime<Utc>,
    /// Parent world's ID (None for primordial worlds).
    pub parent_id: Option<Uuid>,
    /// Epoch at which the parent was forked (None for primordial).
    pub fork_epoch: Option<u64>,
    /// State Chain root at the fork point (cryptographic proof of lineage).
    pub fork_state_root: Option<String>,
    /// Evolution Chain root at the fork point.
    pub fork_evolution_root: Option<String>,
    /// Generation number: 0 for primordial, incremented per fork.
    pub generation: u32,
}

impl WorldIdentity {
    /// Create a new primordial world identity.
    pub fn primordial(name: impl Into<String>, seed: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            seed,
            created_at: Utc::now(),
            parent_id: None,
            fork_epoch: None,
            fork_state_root: None,
            fork_evolution_root: None,
            generation: 0,
        }
    }

    /// Create a forked world identity, inheriting lineage from a parent.
    pub fn forked(
        name: impl Into<String>,
        seed: u64,
        parent: &WorldIdentity,
        fork_epoch: u64,
        state_root: impl Into<String>,
        evolution_root: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            seed,
            created_at: Utc::now(),
            parent_id: Some(parent.id),
            fork_epoch: Some(fork_epoch),
            fork_state_root: Some(state_root.into()),
            fork_evolution_root: Some(evolution_root.into()),
            generation: parent.generation + 1,
        }
    }

    /// Is this a primordial (root) world?
    pub fn is_primordial(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Short display: "Earth-Prime [gen0, abc12345]"
    pub fn label(&self) -> String {
        format!(
            "{} [gen{}, {}]",
            self.name,
            self.generation,
            &self.id.to_string()[..8],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primordial_identity() {
        let id = WorldIdentity::primordial("Earth-Prime", 42);
        assert!(id.is_primordial());
        assert_eq!(id.generation, 0);
        assert!(id.parent_id.is_none());
        assert!(id.label().contains("Earth-Prime"));
        assert!(id.label().contains("gen0"));
    }

    #[test]
    fn forked_identity_lineage() {
        let parent = WorldIdentity::primordial("Earth-Prime", 42);
        let child = WorldIdentity::forked(
            "High Gravity",
            99,
            &parent,
            500,
            "abc123",
            "def456",
        );

        assert!(!child.is_primordial());
        assert_eq!(child.generation, 1);
        assert_eq!(child.parent_id.unwrap(), parent.id);
        assert_eq!(child.fork_epoch.unwrap(), 500);
        assert_eq!(child.fork_state_root.as_deref(), Some("abc123"));
        assert_eq!(child.fork_evolution_root.as_deref(), Some("def456"));
    }

    #[test]
    fn multi_generation_fork() {
        let gen0 = WorldIdentity::primordial("Root", 1);
        let gen1 = WorldIdentity::forked("Child", 2, &gen0, 100, "r1", "e1");
        let gen2 = WorldIdentity::forked("Grandchild", 3, &gen1, 200, "r2", "e2");
        assert_eq!(gen2.generation, 2);
        assert_eq!(gen2.parent_id.unwrap(), gen1.id);
    }
}
