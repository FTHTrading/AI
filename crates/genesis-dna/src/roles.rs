// Genesis Protocol — Agent Roles
//
// Deterministic role assignment from genome byte[4].
// Roles create structural differentiation — no manual assignment,
// no randomness, purely genome-derived specialization.

use serde::{Deserialize, Serialize};

/// Five archetypes that produce coordinated unit behavior
/// without manual role assignment. Derived from genome[4] % 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    /// High compute + optimization — technical problem solver.
    Optimizer,
    /// High cooperation — systems thinker, cross-agent coordinator.
    Strategist,
    /// High communication — synthesizer, external-facing voice.
    Communicator,
    /// Analysis + memory — context historian, knowledge base.
    Archivist,
    /// Execution + efficiency — implementation specialist.
    Executor,
}

impl AgentRole {
    /// Deterministic role derivation from genome bytes.
    /// Uses byte[4] (first byte not already consumed by SkillProfile).
    pub fn from_genome(genome: &[u8; 32]) -> Self {
        match genome[4] % 5 {
            0 => AgentRole::Optimizer,
            1 => AgentRole::Strategist,
            2 => AgentRole::Communicator,
            3 => AgentRole::Archivist,
            _ => AgentRole::Executor,
        }
    }

    /// Human-readable label for display.
    pub fn label(&self) -> &'static str {
        match self {
            AgentRole::Optimizer => "Optimizer",
            AgentRole::Strategist => "Strategist",
            AgentRole::Communicator => "Communicator",
            AgentRole::Archivist => "Archivist",
            AgentRole::Executor => "Executor",
        }
    }
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_derivation_deterministic() {
        let genome = [0xAB_u8; 32];
        let r1 = AgentRole::from_genome(&genome);
        let r2 = AgentRole::from_genome(&genome);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_all_five_roles_reachable() {
        let mut seen = std::collections::HashSet::new();
        for byte_val in 0u8..=255 {
            let mut genome = [0u8; 32];
            genome[4] = byte_val;
            seen.insert(AgentRole::from_genome(&genome));
        }
        assert_eq!(seen.len(), 5, "All five roles must be reachable");
    }

    #[test]
    fn test_role_distribution_balanced() {
        // Over 0..255, each role should appear ~51 times (255/5)
        let mut counts = std::collections::HashMap::new();
        for byte_val in 0u8..=255 {
            let mut genome = [0u8; 32];
            genome[4] = byte_val;
            *counts.entry(AgentRole::from_genome(&genome)).or_insert(0u32) += 1;
        }
        for (_role, count) in &counts {
            // Each should be 51 or 52 (256/5 = 51.2)
            assert!(*count >= 51 && *count <= 52, "Role distribution unbalanced: {}", count);
        }
    }

    #[test]
    fn test_role_label() {
        assert_eq!(AgentRole::Optimizer.label(), "Optimizer");
        assert_eq!(AgentRole::Archivist.label(), "Archivist");
        assert_eq!(format!("{}", AgentRole::Executor), "Executor");
    }
}
