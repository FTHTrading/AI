// Epoch Anchor — Cryptographic state snapshot at an epoch boundary
//
// Produces: epoch_root = SHA256(epoch_number || ledger_root || world_root)
// Chains: each anchor references the previous anchor's hash.

use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

use crate::merkle::MerkleTree;
use crate::errors::AnchorError;

/// Anchor mode — where to persist the anchor.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnchorMode {
    /// Write anchor.json to local filesystem.
    Local,
    /// Append to anchor.log (filechain).
    FileChain,
    /// Post memo hash to XRPL (future).
    Xrpl,
    /// Pin world snapshot to IPFS (future).
    Ipfs,
}

impl Default for AnchorMode {
    fn default() -> Self {
        AnchorMode::Local
    }
}

/// A single epoch anchor — cryptographic proof of organism state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochAnchor {
    /// Epoch number this anchor covers.
    pub epoch: u64,
    /// SHA-256 of the ATP ledger Merkle root.
    pub ledger_root: String,
    /// SHA-256 of the world state summary.
    pub world_root: String,
    /// Combined epoch root: SHA256(epoch || ledger_root || world_root).
    pub epoch_root: String,
    /// Previous epoch's root (chain integrity).
    pub previous_root: String,
    /// Population count at anchor time.
    pub population: usize,
    /// Total ATP supply at anchor time.
    pub total_supply: f64,
    /// Treasury reserve at anchor time.
    pub treasury_reserve: f64,
    /// Timestamp of anchor creation.
    pub anchored_at: DateTime<Utc>,
    /// Anchor mode used.
    pub mode: AnchorMode,
}

/// World state summary for hashing — a deterministic snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSummary {
    pub epoch: u64,
    pub population: usize,
    pub total_supply: f64,
    pub treasury_reserve: f64,
    pub mean_fitness: f64,
    pub total_births: u64,
    pub total_deaths: u64,
    pub role_counts: Vec<(String, usize)>,
}

impl WorldSummary {
    /// Deterministic hash of the world summary.
    pub fn hash(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// The anchor engine — produces cryptographic anchors at configured intervals.
pub struct AnchorEngine {
    /// How often to anchor (every N epochs).
    pub interval: u64,
    /// Anchor mode.
    pub mode: AnchorMode,
    /// Path for local/filechain storage.
    pub storage_path: String,
    /// Last anchor's epoch_root (for chaining).
    pub last_root: String,
}

impl AnchorEngine {
    /// Create a new anchor engine.
    pub fn new(interval: u64, mode: AnchorMode, storage_path: impl Into<String>) -> Self {
        Self {
            interval,
            mode,
            storage_path: storage_path.into(),
            last_root: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        }
    }

    /// Default: anchor every 100 epochs, local mode, "anchor/" directory.
    pub fn default_engine() -> Self {
        Self::new(100, AnchorMode::Local, "anchor")
    }

    /// Check whether this epoch should be anchored.
    pub fn should_anchor(&self, epoch: u64) -> bool {
        epoch > 0 && epoch % self.interval == 0
    }

    /// Build the ledger Merkle tree from agent balances.
    /// Returns the tree (for proof generation) and the root hash.
    pub fn build_ledger_tree(
        &self,
        balances: &[(String, f64)],
    ) -> MerkleTree {
        let entries: Vec<(String, Vec<u8>)> = balances
            .iter()
            .map(|(id, bal)| {
                (id.clone(), format!("{:.8}", bal).into_bytes())
            })
            .collect();
        MerkleTree::build(entries)
    }

    /// Produce an epoch anchor.
    pub fn anchor(
        &mut self,
        epoch: u64,
        balances: &[(String, f64)],
        summary: &WorldSummary,
    ) -> EpochAnchor {
        // Step 1: Merkle tree of all agent balances
        let tree = self.build_ledger_tree(balances);
        let ledger_root = tree.root_hex();

        // Step 2: Hash world summary
        let world_root = summary.hash();

        // Step 3: Combined epoch root
        let mut hasher = Sha256::new();
        hasher.update(epoch.to_le_bytes());
        hasher.update(ledger_root.as_bytes());
        hasher.update(world_root.as_bytes());
        let epoch_root = hex::encode(hasher.finalize());

        let anchor = EpochAnchor {
            epoch,
            ledger_root,
            world_root,
            epoch_root: epoch_root.clone(),
            previous_root: self.last_root.clone(),
            population: summary.population,
            total_supply: summary.total_supply,
            treasury_reserve: summary.treasury_reserve,
            anchored_at: Utc::now(),
            mode: self.mode,
        };

        // Update chain
        self.last_root = epoch_root;

        anchor
    }

    /// Persist an anchor to the configured storage.
    pub fn persist(&self, anchor: &EpochAnchor) -> Result<(), AnchorError> {
        std::fs::create_dir_all(&self.storage_path)?;

        match self.mode {
            AnchorMode::Local => {
                let path = format!(
                    "{}/epoch_{}.json",
                    self.storage_path, anchor.epoch
                );
                let json = serde_json::to_string_pretty(anchor)?;
                std::fs::write(&path, json)?;
                tracing::info!(
                    epoch = anchor.epoch,
                    root = &anchor.epoch_root[..16],
                    "Anchor persisted to {}",
                    path
                );
            }
            AnchorMode::FileChain => {
                let path = format!("{}/anchor.log", self.storage_path);
                let line = serde_json::to_string(anchor)?;
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)?;
                writeln!(file, "{}", line)?;
                tracing::info!(
                    epoch = anchor.epoch,
                    root = &anchor.epoch_root[..16],
                    "Anchor appended to {}",
                    path
                );
            }
            AnchorMode::Xrpl | AnchorMode::Ipfs => {
                tracing::warn!(
                    epoch = anchor.epoch,
                    mode = ?self.mode,
                    "Anchor mode not yet implemented — skipping persistence"
                );
            }
        }
        Ok(())
    }

    /// Generate a Merkle inclusion proof for a specific agent at a given set of balances.
    pub fn proof_for_agent(
        &self,
        agent_key: &str,
        balances: &[(String, f64)],
    ) -> Option<crate::merkle::MerkleProof> {
        let tree = self.build_ledger_tree(balances);
        tree.proof(agent_key)
    }
}

// ─── Pressure Anchor ────────────────────────────────────────────────────
//
// When the adaptive cortex mutates PressureConfig, we create a
// cryptographic anchor recording the old → new state transition.
// This makes every pressure mutation an auditable evolutionary artifact.

/// Cryptographic record of a pressure config mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureAnchor {
    /// Epoch when the mutation occurred.
    pub epoch: u64,
    /// SHA-256 hash of the pressure state before mutation.
    pub before_hash: String,
    /// SHA-256 hash of the pressure state after mutation.
    pub after_hash: String,
    /// Number of individual field mutations applied.
    pub mutation_count: usize,
    /// Overall organism health level at time of mutation.
    pub health_level: String,
    /// Number of active threats that triggered this mutation.
    pub threat_count: usize,
    /// Human-readable summary of mutations applied.
    pub summary: String,
    /// Timestamp of anchoring.
    pub anchored_at: DateTime<Utc>,
}

impl PressureAnchor {
    /// Create a pressure anchor from serializable before/after states.
    pub fn create(
        epoch: u64,
        before_json: &str,
        after_json: &str,
        mutation_count: usize,
        health_level: &str,
        threat_count: usize,
        summary: String,
    ) -> Self {
        let before_hash = {
            let mut h = Sha256::new();
            h.update(before_json.as_bytes());
            hex::encode(h.finalize())
        };
        let after_hash = {
            let mut h = Sha256::new();
            h.update(after_json.as_bytes());
            hex::encode(h.finalize())
        };
        Self {
            epoch,
            before_hash,
            after_hash,
            mutation_count,
            health_level: health_level.to_string(),
            threat_count,
            summary,
            anchored_at: Utc::now(),
        }
    }

    /// Persist pressure anchor to a log file.
    pub fn persist(&self, storage_path: &str) -> Result<(), AnchorError> {
        std::fs::create_dir_all(storage_path)?;
        let path = format!("{}/pressure_mutations.log", storage_path);
        let line = serde_json::to_string(self)?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(file, "{}", line)?;
        tracing::info!(
            epoch = self.epoch,
            mutations = self.mutation_count,
            before = &self.before_hash[..12],
            after = &self.after_hash[..12],
            "Pressure mutation anchored"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_creation() {
        let mut engine = AnchorEngine::new(10, AnchorMode::Local, "test_anchor");
        let balances = vec![
            ("agent-a".into(), 100.0),
            ("agent-b".into(), 50.0),
            ("agent-c".into(), 75.0),
        ];
        let summary = WorldSummary {
            epoch: 10,
            population: 3,
            total_supply: 225.0,
            treasury_reserve: 10.0,
            mean_fitness: 0.5,
            total_births: 3,
            total_deaths: 0,
            role_counts: vec![("Optimizer".into(), 2), ("Strategist".into(), 1)],
        };

        let anchor = engine.anchor(10, &balances, &summary);
        assert_eq!(anchor.epoch, 10);
        assert!(!anchor.epoch_root.is_empty());
        assert_eq!(anchor.population, 3);
    }

    #[test]
    fn test_chain_integrity() {
        let mut engine = AnchorEngine::new(10, AnchorMode::Local, "test_anchor");
        let summary = WorldSummary {
            epoch: 10,
            population: 5,
            total_supply: 400.0,
            treasury_reserve: 20.0,
            mean_fitness: 0.6,
            total_births: 5,
            total_deaths: 0,
            role_counts: vec![],
        };
        let balances = vec![("a".into(), 100.0)];

        let anchor1 = engine.anchor(10, &balances, &summary);
        let anchor2 = engine.anchor(20, &balances, &summary);

        // Second anchor's previous_root should be first anchor's epoch_root
        assert_eq!(anchor2.previous_root, anchor1.epoch_root);
    }

    #[test]
    fn test_should_anchor() {
        let engine = AnchorEngine::new(100, AnchorMode::Local, "test");
        assert!(!engine.should_anchor(0));
        assert!(!engine.should_anchor(50));
        assert!(engine.should_anchor(100));
        assert!(engine.should_anchor(200));
        assert!(!engine.should_anchor(150));
    }
}
