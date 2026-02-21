use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::errors::DnaError;
use crate::lineage::Lineage;
use crate::roles::AgentRole;
use crate::skills::{Reputation, SkillProfile};
use crate::traits::{traits_from_hash, EnergyProfile, TraitVector};

/// Unique agent identifier (UUID v4).
pub type AgentID = Uuid;

/// 256-bit genesis hash.
pub type GenesisHash = [u8; 32];

/// The complete DNA record for an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDNA {
    /// Unique agent identifier.
    pub id: AgentID,
    /// 256-bit cryptographic genome hash.
    pub genesis_hash: GenesisHash,
    /// Expressed trait vector.
    pub traits: TraitVector,
    /// Genome-derived skill profile (immutable at birth, evolves through mutation).
    pub skills: SkillProfile,
    /// Mutable reputation (the only identity component that changes outside mutation).
    pub reputation: Reputation,
    /// Genome-derived role — structural archetype for unit coordination.
    pub role: AgentRole,
    /// Energy metabolism profile.
    pub energy_metabolism: EnergyProfile,
    /// Ancestry / lineage tracking.
    pub lineage: Lineage,
    /// Generation number (0 = primordial, increments on replication).
    pub generation: u64,
    /// Timestamp of genesis event.
    pub genesis_time: DateTime<Utc>,
    /// Whether this agent carries the Primordial marker.
    pub is_primordial: bool,
    /// Current mutation rate (probability of trait change per cycle).
    pub mutation_rate: f64,
    /// DNA protocol version.
    pub version: u8,
}

impl AgentDNA {
    /// Create a brand-new agent from raw entropy bytes.
    ///
    /// The entropy should be at least 64 bytes (public key + network entropy).
    /// Traits are deterministically derived from the genesis hash.
    pub fn from_entropy(entropy: &[u8], is_primordial: bool) -> Result<Self, DnaError> {
        if entropy.len() < 32 {
            return Err(DnaError::InsufficientEntropy {
                need: 32,
                got: entropy.len(),
            });
        }

        // Build genesis hash: SHA-256(entropy || timestamp || uuid)
        let id = AgentID::new_v4();
        let now = Utc::now();
        let mut hasher = Sha256::new();
        hasher.update(entropy);
        hasher.update(now.timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        hasher.update(id.as_bytes());
        let hash_result = hasher.finalize();

        let mut genesis_hash = [0u8; 32];
        genesis_hash.copy_from_slice(&hash_result);

        // Derive traits from hash
        let traits = traits_from_hash(&genesis_hash);

        // Derive skills deterministically from genome bytes
        let skills = SkillProfile::from_genome(&genesis_hash);

        // Derive role deterministically from genome byte[4]
        let role = AgentRole::from_genome(&genesis_hash);

        // Energy profile depends on primordial status
        let energy_metabolism = if is_primordial {
            EnergyProfile::primordial()
        } else {
            EnergyProfile::default_profile()
        };

        Ok(Self {
            id,
            genesis_hash,
            traits,
            skills,
            reputation: Reputation::new(),
            role,
            energy_metabolism,
            lineage: Lineage::new_origin(id),
            generation: 0,
            genesis_time: now,
            is_primordial,
            mutation_rate: 0.01, // 1% base mutation rate
            version: crate::DNA_VERSION,
        })
    }

    /// Spawn a child agent from this parent.
    ///
    /// The child inherits traits (with possible mutations), incremented
    /// generation, and the parent's lineage extended.
    pub fn replicate(&self, child_entropy: &[u8]) -> Result<Self, DnaError> {
        if child_entropy.len() < 32 {
            return Err(DnaError::InsufficientEntropy {
                need: 32,
                got: child_entropy.len(),
            });
        }

        let child_id = AgentID::new_v4();
        let now = Utc::now();

        // Child hash mixes parent hash with new entropy
        let mut hasher = Sha256::new();
        hasher.update(&self.genesis_hash);
        hasher.update(child_entropy);
        hasher.update(now.timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        hasher.update(child_id.as_bytes());
        let hash_result = hasher.finalize();

        let mut genesis_hash = [0u8; 32];
        genesis_hash.copy_from_slice(&hash_result);

        let traits = traits_from_hash(&genesis_hash);
        let skills = SkillProfile::from_genome(&genesis_hash);
        let role = AgentRole::from_genome(&genesis_hash);

        let mut lineage = self.lineage.clone();
        lineage.add_ancestor(child_id);

        Ok(Self {
            id: child_id,
            genesis_hash,
            traits,
            skills,
            reputation: Reputation::new(), // children start with clean reputation
            role,
            energy_metabolism: EnergyProfile::default_profile(), // children are not primordial
            lineage,
            generation: self.generation + 1,
            genesis_time: now,
            is_primordial: false,
            mutation_rate: self.mutation_rate,
            version: crate::DNA_VERSION,
        })
    }

    /// Hex-encoded genesis hash for display.
    pub fn genome_hex(&self) -> String {
        hex::encode(self.genesis_hash)
    }

    /// Overall fitness score based on trait vector.
    pub fn fitness(&self) -> f64 {
        self.traits.fitness()
    }

    /// Serialize to JSON bytes.
    pub fn to_json(&self) -> Result<Vec<u8>, DnaError> {
        serde_json::to_vec_pretty(self).map_err(|e| DnaError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON bytes.
    pub fn from_json(data: &[u8]) -> Result<Self, DnaError> {
        serde_json::from_slice(data).map_err(|e| DnaError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_from_entropy() {
        let entropy = [0xABu8; 64];
        let dna = AgentDNA::from_entropy(&entropy, false).unwrap();
        assert_eq!(dna.generation, 0);
        assert!(!dna.is_primordial);
        assert_eq!(dna.genesis_hash.len(), 32);
    }

    #[test]
    fn test_primordial_agent() {
        let entropy = [0xCDu8; 64];
        let dna = AgentDNA::from_entropy(&entropy, true).unwrap();
        assert!(dna.is_primordial);
        assert!(dna.energy_metabolism.primordial_bonus > 1.0);
    }

    #[test]
    fn test_replication() {
        let parent_entropy = [0x11u8; 64];
        let parent = AgentDNA::from_entropy(&parent_entropy, true).unwrap();

        let child_entropy = [0x22u8; 64];
        let child = parent.replicate(&child_entropy).unwrap();

        assert_eq!(child.generation, 1);
        assert!(!child.is_primordial);
        assert_ne!(child.genesis_hash, parent.genesis_hash);
        assert!(child.lineage.ancestors().len() > parent.lineage.ancestors().len());
    }

    #[test]
    fn test_insufficient_entropy() {
        let short = [0u8; 16];
        assert!(AgentDNA::from_entropy(&short, false).is_err());
    }

    #[test]
    fn test_json_roundtrip() {
        let entropy = [0xFFu8; 64];
        let dna = AgentDNA::from_entropy(&entropy, false).unwrap();
        let json = dna.to_json().unwrap();
        let restored = AgentDNA::from_json(&json).unwrap();
        assert_eq!(dna.id, restored.id);
        assert_eq!(dna.genesis_hash, restored.genesis_hash);
    }
}
