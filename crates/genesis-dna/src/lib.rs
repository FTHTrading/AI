// Genesis DNA — Cryptographic Identity & Genetic Traits for AI Agents
//
// Each agent receives a unique 256-bit genome hash derived from initial state,
// timestamp, and network entropy. Traits encode compute efficiency, solution
// quality, replication fidelity, and cooperation coefficient.

pub mod traits;
pub mod genome;
pub mod lineage;
pub mod skills;
pub mod roles;
pub mod errors;

pub use genome::{AgentDNA, AgentID, GenesisHash};
pub use traits::{TraitVector, TraitKind, EnergyProfile};
pub use lineage::Lineage;
pub use skills::{SkillProfile, Reputation};
pub use roles::AgentRole;
pub use errors::DnaError;

/// Current protocol version for DNA encoding.
pub const DNA_VERSION: u8 = 1;

/// Size of the genesis hash in bytes (256-bit).
pub const GENESIS_HASH_SIZE: usize = 32;
