use serde::{Deserialize, Serialize};

use crate::genome::AgentID;

/// Maximum lineage depth to prevent unbounded ancestry chains.
pub const MAX_LINEAGE_DEPTH: usize = 1000;

/// Tracks the evolutionary ancestry of an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    /// The agent's own ID (the newest entry).
    origin: AgentID,
    /// Ordered list of ancestor IDs, from oldest to most recent parent.
    ancestors: Vec<AgentID>,
}

impl Lineage {
    /// Create a lineage for a first-generation (origin) agent.
    pub fn new_origin(id: AgentID) -> Self {
        Self {
            origin: id,
            ancestors: Vec::new(),
        }
    }

    /// Add an ancestor to the lineage.
    pub fn add_ancestor(&mut self, id: AgentID) {
        self.ancestors.push(id);
    }

    /// Get the origin agent ID.
    pub fn origin(&self) -> AgentID {
        self.origin
    }

    /// Get the full ancestor chain.
    pub fn ancestors(&self) -> &[AgentID] {
        &self.ancestors
    }

    /// Depth of lineage (number of generations).
    pub fn depth(&self) -> usize {
        self.ancestors.len()
    }

    /// Check if a given agent is an ancestor.
    pub fn has_ancestor(&self, id: &AgentID) -> bool {
        self.ancestors.contains(id)
    }

    /// Get the most recent parent (if any).
    pub fn parent(&self) -> Option<AgentID> {
        self.ancestors.last().copied()
    }

    /// Compute lineage similarity with another agent (shared ancestors).
    pub fn relatedness(&self, other: &Lineage) -> f64 {
        if self.ancestors.is_empty() || other.ancestors.is_empty() {
            return 0.0;
        }
        let shared = self
            .ancestors
            .iter()
            .filter(|a| other.ancestors.contains(a))
            .count();
        let total = self.ancestors.len().max(other.ancestors.len());
        shared as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_new_origin() {
        let id = Uuid::new_v4();
        let lineage = Lineage::new_origin(id);
        assert_eq!(lineage.origin(), id);
        assert_eq!(lineage.depth(), 0);
        assert!(lineage.parent().is_none());
    }

    #[test]
    fn test_ancestry() {
        let id = Uuid::new_v4();
        let mut lineage = Lineage::new_origin(id);

        let parent = Uuid::new_v4();
        lineage.add_ancestor(parent);
        assert_eq!(lineage.depth(), 1);
        assert_eq!(lineage.parent(), Some(parent));
        assert!(lineage.has_ancestor(&parent));
    }

    #[test]
    fn test_relatedness() {
        let shared_ancestor = Uuid::new_v4();

        let mut l1 = Lineage::new_origin(Uuid::new_v4());
        l1.add_ancestor(shared_ancestor);
        l1.add_ancestor(Uuid::new_v4());

        let mut l2 = Lineage::new_origin(Uuid::new_v4());
        l2.add_ancestor(shared_ancestor);
        l2.add_ancestor(Uuid::new_v4());

        let r = l1.relatedness(&l2);
        assert!(r > 0.0 && r <= 1.0);
    }
}
