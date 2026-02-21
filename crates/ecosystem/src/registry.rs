use std::collections::HashMap;

use chrono::{DateTime, Utc};
use genesis_dna::{AgentDNA, AgentID};
use serde::{Deserialize, Serialize};

use crate::errors::EcosystemError;

/// Online status of an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is active and responsive.
    Online,
    /// Agent is alive but not actively processing.
    Idle,
    /// Agent is in hibernation due to ATP depletion.
    Stasis,
    /// Agent is permanently offline (dead).
    Dead,
}

/// A registered agent in the ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredAgent {
    /// Agent identifier.
    pub id: AgentID,
    /// Display name.
    pub name: String,
    /// Current status.
    pub status: AgentStatus,
    /// The phylum (task specialization cluster) this agent belongs to.
    pub phylum: String,
    /// Neighbors in the P2P mesh (connected agents).
    pub neighbors: Vec<AgentID>,
    /// DNA genome hex (for verification).
    pub genome_hex: String,
    /// Generation number.
    pub generation: u64,
    /// Fitness score.
    pub fitness: f64,
    /// Last heartbeat timestamp.
    pub last_heartbeat: DateTime<Utc>,
    /// Registration timestamp.
    pub registered_at: DateTime<Utc>,
}

/// Registry of all agents in the ecosystem.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AgentRegistry {
    agents: HashMap<AgentID, RegisteredAgent>,
    phyla: HashMap<String, Vec<AgentID>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new agent from its DNA.
    pub fn register(
        &mut self,
        dna: &AgentDNA,
        name: impl Into<String>,
        phylum: impl Into<String>,
    ) -> Result<&RegisteredAgent, EcosystemError> {
        if self.agents.contains_key(&dna.id) {
            return Err(EcosystemError::AgentAlreadyRegistered(dna.id.to_string()));
        }

        let phylum_name = phylum.into();
        let now = Utc::now();

        let agent = RegisteredAgent {
            id: dna.id,
            name: name.into(),
            status: AgentStatus::Online,
            phylum: phylum_name.clone(),
            neighbors: Vec::new(),
            genome_hex: dna.genome_hex(),
            generation: dna.generation,
            fitness: dna.fitness(),
            last_heartbeat: now,
            registered_at: now,
        };

        self.agents.insert(dna.id, agent);
        self.phyla
            .entry(phylum_name)
            .or_default()
            .push(dna.id);

        Ok(&self.agents[&dna.id])
    }

    /// Get an agent by ID.
    pub fn get(&self, id: &AgentID) -> Option<&RegisteredAgent> {
        self.agents.get(id)
    }

    /// Get a mutable reference to an agent.
    pub fn get_mut(&mut self, id: &AgentID) -> Option<&mut RegisteredAgent> {
        self.agents.get_mut(id)
    }

    /// Update agent status.
    pub fn set_status(
        &mut self,
        id: &AgentID,
        status: AgentStatus,
    ) -> Result<(), EcosystemError> {
        let agent = self
            .agents
            .get_mut(id)
            .ok_or_else(|| EcosystemError::AgentNotRegistered(id.to_string()))?;
        agent.status = status;
        Ok(())
    }

    /// Record a heartbeat from an agent.
    pub fn heartbeat(&mut self, id: &AgentID) -> Result<(), EcosystemError> {
        let agent = self
            .agents
            .get_mut(id)
            .ok_or_else(|| EcosystemError::AgentNotRegistered(id.to_string()))?;
        agent.last_heartbeat = Utc::now();
        if agent.status == AgentStatus::Idle {
            agent.status = AgentStatus::Online;
        }
        Ok(())
    }

    /// Connect two agents as neighbors in the mesh.
    pub fn connect(&mut self, a: &AgentID, b: &AgentID) -> Result<(), EcosystemError> {
        if !self.agents.contains_key(a) {
            return Err(EcosystemError::AgentNotRegistered(a.to_string()));
        }
        if !self.agents.contains_key(b) {
            return Err(EcosystemError::AgentNotRegistered(b.to_string()));
        }

        if let Some(agent_a) = self.agents.get_mut(a) {
            if !agent_a.neighbors.contains(b) {
                agent_a.neighbors.push(*b);
            }
        }
        if let Some(agent_b) = self.agents.get_mut(b) {
            if !agent_b.neighbors.contains(a) {
                agent_b.neighbors.push(*a);
            }
        }
        Ok(())
    }

    /// Get all agents in a specific phylum.
    pub fn phylum_members(&self, phylum: &str) -> Vec<&RegisteredAgent> {
        self.phyla
            .get(phylum)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.agents.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all registered phylum names.
    pub fn phyla(&self) -> Vec<&str> {
        self.phyla.keys().map(|s| s.as_str()).collect()
    }

    /// Total number of registered agents.
    pub fn count(&self) -> usize {
        self.agents.len()
    }

    /// Get all online agents.
    pub fn online_agents(&self) -> Vec<&RegisteredAgent> {
        self.agents
            .values()
            .filter(|a| a.status == AgentStatus::Online)
            .collect()
    }

    /// Get agents sorted by fitness (highest first).
    pub fn leaderboard(&self, limit: usize) -> Vec<&RegisteredAgent> {
        let mut agents: Vec<&RegisteredAgent> = self.agents.values().collect();
        agents.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        agents.truncate(limit);
        agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genesis_dna::AgentDNA;

    #[test]
    fn test_register_and_lookup() {
        let mut registry = AgentRegistry::new();
        let dna = AgentDNA::from_entropy(&[0xABu8; 64], false).unwrap();

        registry.register(&dna, "Agent-1", "coding").unwrap();
        assert!(registry.get(&dna.id).is_some());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = AgentRegistry::new();
        let dna = AgentDNA::from_entropy(&[0xCDu8; 64], false).unwrap();

        registry.register(&dna, "Agent-1", "coding").unwrap();
        assert!(registry.register(&dna, "Agent-1-dup", "coding").is_err());
    }

    #[test]
    fn test_mesh_connection() {
        let mut registry = AgentRegistry::new();
        let dna1 = AgentDNA::from_entropy(&[0x11u8; 64], false).unwrap();
        let dna2 = AgentDNA::from_entropy(&[0x22u8; 64], false).unwrap();

        registry.register(&dna1, "A", "coding").unwrap();
        registry.register(&dna2, "B", "coding").unwrap();
        registry.connect(&dna1.id, &dna2.id).unwrap();

        assert!(registry.get(&dna1.id).unwrap().neighbors.contains(&dna2.id));
        assert!(registry.get(&dna2.id).unwrap().neighbors.contains(&dna1.id));
    }

    #[test]
    fn test_phylum_query() {
        let mut registry = AgentRegistry::new();
        let dna1 = AgentDNA::from_entropy(&[0x33u8; 64], false).unwrap();
        let dna2 = AgentDNA::from_entropy(&[0x44u8; 64], false).unwrap();
        let dna3 = AgentDNA::from_entropy(&[0x55u8; 64], false).unwrap();

        registry.register(&dna1, "Coder-1", "coding").unwrap();
        registry.register(&dna2, "Coder-2", "coding").unwrap();
        registry.register(&dna3, "Artist-1", "creativity").unwrap();

        assert_eq!(registry.phylum_members("coding").len(), 2);
        assert_eq!(registry.phylum_members("creativity").len(), 1);
    }
}
