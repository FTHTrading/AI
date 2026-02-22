// Evolution Chain — Cryptographic lineage of organism law changes
//
// When the adaptive cortex mutates PressureConfig, each mutation event
// becomes a hash-linked anchor in the Evolution Chain. Unlike the State
// Chain (EpochAnchor), which snapshots the organism's body, the Evolution
// Chain records WHY the organism's physics changed.
//
// Chain structure:
//   evolution_root = SHA256(epoch || before_hash || after_hash || previous_root || epoch_root_ref)
//
// Cross-chain link:
//   Each EvolutionAnchor cryptographically binds to the latest State Chain
//   position (epoch_root_ref), making evolution provenance verifiable
//   against the organism's physical state at mutation time.
//
// Dual-chain architecture:
//
//   State Chain:     [E100] ← [E200] ← [E300] ← [E400]
//                      ↑         ↑         ↑
//   Evolution Chain: [P25]←[P50]←[P75]←[P100]←[P125]←[P150]←...
//
// Each evolution event carries a cryptographic reference to the most
// recent epoch root. Each epoch anchor carries an informational reference
// to the most recent evolution root. Two independent, verifiable chains
// with cryptographic cross-references.

use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

use crate::errors::AnchorError;

/// The genesis (null) root — used as previous_root for the first anchor in either chain.
pub const GENESIS_ROOT: &str = "0000000000000000000000000000000000000000000000000000000000000000";

// ─── Types ──────────────────────────────────────────────────────────────

/// A single mutation record within an evolution anchor.
///
/// Captures the exact field change, what triggered it, and why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationRecord {
    /// Which pressure field was mutated (e.g. "soft_cap", "entropy_coeff").
    pub field: String,
    /// Value before mutation.
    pub old_value: f64,
    /// Value after mutation.
    pub new_value: f64,
    /// What immune threat triggered this mutation (e.g. "PopulationCollapse").
    pub trigger: String,
    /// Severity level at the time (e.g. "Warning", "Critical").
    pub severity: String,
    /// Human-readable rationale for the mutation.
    pub rationale: String,
}

/// A chain-linked cryptographic record of pressure evolution.
///
/// Each anchor contains:
/// - SHA-256 hashes of the PressureConfig before and after mutation
/// - Individual mutation records for auditability
/// - A chain-linked `evolution_root` (incorporating the previous root)
/// - A cross-chain `epoch_root_ref` (binding to the state chain)
///
/// The `evolution_root` hash includes `epoch_root_ref`, so the evolution
/// chain's integrity is cryptographically tied to the state chain's position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionAnchor {
    /// Epoch when the mutations occurred.
    pub epoch: u64,
    /// SHA-256 of the serialized PressureConfig before mutations.
    pub before_hash: String,
    /// SHA-256 of the serialized PressureConfig after mutations.
    pub after_hash: String,
    /// Chain-linked root: SHA256(epoch || before || after || prev_root || epoch_ref).
    pub evolution_root: String,
    /// Previous evolution anchor's root (chain integrity link).
    pub previous_evolution_root: String,
    /// Cross-chain reference: latest epoch_root from the State Chain.
    /// Included in evolution_root hash — cryptographically bound to state.
    pub epoch_root_ref: String,
    /// Individual mutation records for this evolution event.
    pub mutations: Vec<MutationRecord>,
    /// Number of active immune threats at time of mutation.
    pub threat_count: usize,
    /// Overall organism health level at time of mutation.
    pub health_level: String,
    /// Timestamp of anchoring.
    pub anchored_at: DateTime<Utc>,
}

impl EvolutionAnchor {
    /// Recompute the evolution root from its components (for verification).
    ///
    /// If this doesn't match `self.evolution_root`, the anchor was tampered with.
    pub fn recompute_root(&self) -> String {
        compute_evolution_root(
            self.epoch,
            &self.before_hash,
            &self.after_hash,
            &self.previous_evolution_root,
            &self.epoch_root_ref,
        )
    }
}

// ─── Engine ─────────────────────────────────────────────────────────────

/// Engine that produces, chains, and persists evolution anchors.
///
/// Maintains the chain's last root for linking, tracks cumulative
/// statistics, and keeps an in-memory history window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEngine {
    /// Path for log file persistence.
    pub storage_path: String,
    /// Last evolution anchor's root hash (chain link for next anchor).
    pub last_evolution_root: String,
    /// Total individual field mutations applied across all time.
    #[serde(default)]
    pub total_mutations: u64,
    /// Total evolution events (anchors produced) across all time.
    #[serde(default)]
    pub total_events: u64,
    /// In-memory recent history (not persisted — rebuilt from log if needed).
    #[serde(skip)]
    history: Vec<EvolutionAnchor>,
    /// Maximum in-memory history entries.
    #[serde(skip, default = "default_max_history")]
    max_history: usize,
}

fn default_max_history() -> usize {
    100
}

impl Default for EvolutionEngine {
    fn default() -> Self {
        Self {
            storage_path: "anchor".to_string(),
            last_evolution_root: GENESIS_ROOT.to_string(),
            total_mutations: 0,
            total_events: 0,
            history: Vec::new(),
            max_history: 100,
        }
    }
}

impl EvolutionEngine {
    /// Create a new evolution engine with the given storage path.
    pub fn new(storage_path: impl Into<String>) -> Self {
        Self {
            storage_path: storage_path.into(),
            ..Default::default()
        }
    }

    /// Access the last evolution root (for cross-chain references).
    pub fn last_root(&self) -> &str {
        &self.last_evolution_root
    }

    /// Recent evolution history (in-memory window).
    pub fn recent_history(&self) -> &[EvolutionAnchor] {
        &self.history
    }

    /// Total events produced across all time.
    pub fn event_count(&self) -> u64 {
        self.total_events
    }

    /// Produce a chain-linked evolution anchor from pressure mutation data.
    ///
    /// The anchor is hash-linked to the previous evolution anchor and
    /// cross-referenced to the latest state chain position.
    ///
    /// # Arguments
    /// - `epoch` — current epoch number
    /// - `before_json` — serialized PressureConfig before mutations
    /// - `after_json` — serialized PressureConfig after mutations
    /// - `mutations` — individual mutation records
    /// - `threat_count` — number of active immune threats
    /// - `health_level` — organism health string
    /// - `epoch_root_ref` — latest State Chain epoch root (cross-chain link)
    pub fn anchor(
        &mut self,
        epoch: u64,
        before_json: &str,
        after_json: &str,
        mutations: Vec<MutationRecord>,
        threat_count: usize,
        health_level: &str,
        epoch_root_ref: &str,
    ) -> EvolutionAnchor {
        let before_hash = sha256_hex(before_json);
        let after_hash = sha256_hex(after_json);

        let evolution_root = compute_evolution_root(
            epoch,
            &before_hash,
            &after_hash,
            &self.last_evolution_root,
            epoch_root_ref,
        );

        let mutation_count = mutations.len();

        let anchor = EvolutionAnchor {
            epoch,
            before_hash,
            after_hash,
            evolution_root: evolution_root.clone(),
            previous_evolution_root: self.last_evolution_root.clone(),
            epoch_root_ref: epoch_root_ref.to_string(),
            mutations,
            threat_count,
            health_level: health_level.to_string(),
            anchored_at: Utc::now(),
        };

        // Advance chain state
        self.last_evolution_root = evolution_root;
        self.total_events += 1;
        self.total_mutations += mutation_count as u64;

        // Maintain in-memory history window
        self.history.push(anchor.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        anchor
    }

    /// Persist an evolution anchor to the log file.
    pub fn persist(&self, anchor: &EvolutionAnchor) -> Result<(), AnchorError> {
        std::fs::create_dir_all(&self.storage_path)?;
        let path = format!("{}/evolution.log", self.storage_path);
        let line = serde_json::to_string(anchor)?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(file, "{}", line)?;
        tracing::info!(
            epoch = anchor.epoch,
            mutations = anchor.mutations.len(),
            root = &anchor.evolution_root[..16],
            prev = &anchor.previous_evolution_root[..16],
            epoch_ref = &anchor.epoch_root_ref[..16],
            "EVOLUTION CHAIN: anchor linked and persisted"
        );
        Ok(())
    }

    /// Produce and persist an evolution anchor in one call.
    pub fn record(
        &mut self,
        epoch: u64,
        before_json: &str,
        after_json: &str,
        mutations: Vec<MutationRecord>,
        threat_count: usize,
        health_level: &str,
        epoch_root_ref: &str,
    ) -> Result<EvolutionAnchor, AnchorError> {
        let anchor = self.anchor(
            epoch, before_json, after_json, mutations,
            threat_count, health_level, epoch_root_ref,
        );
        self.persist(&anchor)?;
        Ok(anchor)
    }
}

// ─── Chain Verification ─────────────────────────────────────────────────

/// Result of cross-chain verification between State and Evolution chains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainVerification {
    /// Whether all cross-references are valid.
    pub valid: bool,
    /// Total evolution anchors checked.
    pub evolution_anchors_checked: usize,
    /// Evolution anchors with valid epoch_root_ref (known or genesis).
    pub valid_cross_refs: usize,
    /// Evolution anchors referencing unknown epoch roots.
    pub orphaned_refs: usize,
    /// Human-readable message.
    pub message: String,
}

/// Loadable and verifiable evolution chain (parallel to AnchorChain).
///
/// Supports independent integrity verification and cross-chain
/// verification against the State Chain (AnchorChain).
pub struct EvolutionChain {
    /// Evolution anchors ordered by epoch.
    pub anchors: Vec<EvolutionAnchor>,
}

impl EvolutionChain {
    /// Create an empty chain.
    pub fn new() -> Self {
        Self { anchors: Vec::new() }
    }

    /// Load evolution chain from a log file (one JSON per line).
    pub fn load_from_log(path: &str) -> Result<Self, AnchorError> {
        let data = std::fs::read_to_string(path)?;
        let mut anchors = Vec::new();
        for line in data.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let anchor: EvolutionAnchor = serde_json::from_str(line)?;
            anchors.push(anchor);
        }
        anchors.sort_by_key(|a| a.epoch);
        Ok(Self { anchors })
    }

    /// Add an anchor to the chain (in-memory).
    pub fn push(&mut self, anchor: EvolutionAnchor) {
        self.anchors.push(anchor);
    }

    /// Verify evolution chain integrity.
    ///
    /// For each anchor:
    ///   1. Recompute `evolution_root` from components → detect tampering
    ///   2. Verify `previous_evolution_root` links → detect chain breaks
    pub fn verify(&self) -> crate::chain::ChainVerification {
        use crate::chain::ChainVerification;

        if self.anchors.is_empty() {
            return ChainVerification {
                total_anchors: 0,
                valid: true,
                first_epoch: 0,
                last_epoch: 0,
                break_at: None,
                message: "Empty evolution chain — trivially valid".into(),
            };
        }

        let first_epoch = self.anchors[0].epoch;
        let last_epoch = self.anchors.last().unwrap().epoch;

        for i in 0..self.anchors.len() {
            let anchor = &self.anchors[i];

            // Re-derive evolution_root to detect tampering
            let recomputed = anchor.recompute_root();
            if recomputed != anchor.evolution_root {
                return ChainVerification {
                    total_anchors: self.anchors.len(),
                    valid: false,
                    first_epoch,
                    last_epoch,
                    break_at: Some(anchor.epoch),
                    message: format!(
                        "Evolution epoch {} root tampered: stored {} != recomputed {}",
                        anchor.epoch,
                        &anchor.evolution_root[..16],
                        &recomputed[..16]
                    ),
                };
            }

            // Chain linkage: previous_evolution_root must match prior anchor's evolution_root
            if i > 0 {
                let prev = &self.anchors[i - 1];
                if anchor.previous_evolution_root != prev.evolution_root {
                    return ChainVerification {
                        total_anchors: self.anchors.len(),
                        valid: false,
                        first_epoch,
                        last_epoch,
                        break_at: Some(anchor.epoch),
                        message: format!(
                            "Evolution chain break at epoch {}: previous {} != epoch {} root {}",
                            anchor.epoch,
                            &anchor.previous_evolution_root[..16],
                            prev.epoch,
                            &prev.evolution_root[..16]
                        ),
                    };
                }
            }
        }

        ChainVerification {
            total_anchors: self.anchors.len(),
            valid: true,
            first_epoch,
            last_epoch,
            break_at: None,
            message: format!(
                "Evolution chain valid: {} anchors, epochs {}-{}",
                self.anchors.len(),
                first_epoch,
                last_epoch
            ),
        }
    }

    /// Cross-verify evolution chain references against a state chain.
    ///
    /// Checks that each evolution anchor's `epoch_root_ref` corresponds
    /// to an actual `epoch_root` in the state chain, or is the genesis
    /// root (acceptable for early evolution events before the first state
    /// anchor is produced).
    pub fn cross_verify(&self, state_chain: &crate::chain::AnchorChain) -> CrossChainVerification {
        if self.anchors.is_empty() {
            return CrossChainVerification {
                valid: true,
                evolution_anchors_checked: 0,
                valid_cross_refs: 0,
                orphaned_refs: 0,
                message: "No evolution anchors to cross-verify".into(),
            };
        }

        // Build set of known epoch roots from state chain
        let known_roots: std::collections::HashSet<&str> = state_chain
            .anchors
            .iter()
            .map(|a| a.epoch_root.as_str())
            .collect();

        let mut valid_refs = 0;
        let mut orphaned = 0;

        for anchor in &self.anchors {
            if anchor.epoch_root_ref == GENESIS_ROOT {
                // Early evolution events before first state anchor — acceptable
                valid_refs += 1;
            } else if known_roots.contains(anchor.epoch_root_ref.as_str()) {
                valid_refs += 1;
            } else {
                orphaned += 1;
            }
        }

        let valid = orphaned == 0;
        CrossChainVerification {
            valid,
            evolution_anchors_checked: self.anchors.len(),
            valid_cross_refs: valid_refs,
            orphaned_refs: orphaned,
            message: if valid {
                format!(
                    "Cross-chain valid: all {} evolution anchors reference known state roots",
                    self.anchors.len()
                )
            } else {
                format!(
                    "Cross-chain warning: {} of {} evolution anchors reference unknown state roots",
                    orphaned, self.anchors.len()
                )
            },
        }
    }

    /// Get the latest evolution anchor.
    pub fn latest(&self) -> Option<&EvolutionAnchor> {
        self.anchors.last()
    }

    /// Total individual mutations across the entire evolution chain.
    pub fn total_mutations(&self) -> usize {
        self.anchors.iter().map(|a| a.mutations.len()).sum()
    }

    /// Number of anchors in the chain.
    pub fn len(&self) -> usize {
        self.anchors.len()
    }

    /// Whether the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }
}

impl Default for EvolutionChain {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────

/// SHA-256 hex digest of a string.
fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute evolution root from components.
fn compute_evolution_root(
    epoch: u64,
    before_hash: &str,
    after_hash: &str,
    previous_root: &str,
    epoch_root_ref: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(epoch.to_le_bytes());
    hasher.update(before_hash.as_bytes());
    hasher.update(after_hash.as_bytes());
    hasher.update(previous_root.as_bytes());
    hasher.update(epoch_root_ref.as_bytes());
    hex::encode(hasher.finalize())
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_mutation(field: &str) -> MutationRecord {
        MutationRecord {
            field: field.to_string(),
            old_value: 0.5,
            new_value: 0.6,
            trigger: "TestThreat".to_string(),
            severity: "Warning".to_string(),
            rationale: "Test mutation".to_string(),
        }
    }

    fn fake_epoch_root() -> String {
        "abcd".repeat(16) // 64-char hex
    }

    #[test]
    fn test_engine_chain_linking() {
        let mut engine = EvolutionEngine::new("test_evo");
        let epoch_root = fake_epoch_root();

        let a1 = engine.anchor(
            25, r#"{"soft_cap":180}"#, r#"{"soft_cap":190}"#,
            vec![dummy_mutation("soft_cap")],
            2, "Warning", &epoch_root,
        );
        assert_eq!(a1.previous_evolution_root, GENESIS_ROOT);
        assert!(!a1.evolution_root.is_empty());
        assert_ne!(a1.evolution_root, GENESIS_ROOT);

        let a2 = engine.anchor(
            50, r#"{"entropy":0.00002}"#, r#"{"entropy":0.00001}"#,
            vec![dummy_mutation("entropy_coeff")],
            1, "Watch", &epoch_root,
        );
        assert_eq!(a2.previous_evolution_root, a1.evolution_root);
        assert_ne!(a2.evolution_root, a1.evolution_root);

        assert_eq!(engine.total_events, 2);
        assert_eq!(engine.total_mutations, 2);
    }

    #[test]
    fn test_chain_verification_valid() {
        let mut engine = EvolutionEngine::new("test_evo");
        let epoch_root = fake_epoch_root();

        let mut chain = EvolutionChain::new();
        for epoch in (25..=125).step_by(25) {
            let anchor = engine.anchor(
                epoch as u64,
                &format!(r#"{{"epoch":{}}}"#, epoch),
                &format!(r#"{{"epoch_after":{}}}"#, epoch),
                vec![dummy_mutation("soft_cap")],
                1, "Watch", &epoch_root,
            );
            chain.push(anchor);
        }

        let result = chain.verify();
        assert!(result.valid, "{}", result.message);
        assert_eq!(result.total_anchors, 5);
    }

    #[test]
    fn test_chain_tamper_detection() {
        let mut engine = EvolutionEngine::new("test_evo");
        let epoch_root = fake_epoch_root();

        let mut chain = EvolutionChain::new();
        let a1 = engine.anchor(
            25, "{}", "{}",
            vec![dummy_mutation("soft_cap")],
            1, "Watch", &epoch_root,
        );
        chain.push(a1);

        let mut a2 = engine.anchor(
            50, "{}", "{}",
            vec![dummy_mutation("entropy_coeff")],
            1, "Watch", &epoch_root,
        );
        // Tamper: modify the stored root
        a2.evolution_root = "deadbeef".repeat(8);
        chain.push(a2);

        let result = chain.verify();
        assert!(!result.valid);
        assert_eq!(result.break_at, Some(50));
    }

    #[test]
    fn test_chain_break_detection() {
        let epoch_root = fake_epoch_root();

        // Engine 1 produces the first anchor
        let mut engine1 = EvolutionEngine::new("test_evo");
        let a1 = engine1.anchor(
            25, "{}", "{}",
            vec![dummy_mutation("soft_cap")],
            1, "Watch", &epoch_root,
        );

        // Engine 2 produces a second anchor independently (broken chain)
        let mut engine2 = EvolutionEngine::new("test_evo");
        let a2 = engine2.anchor(
            50, "{}", "{}",
            vec![dummy_mutation("entropy_coeff")],
            1, "Watch", &epoch_root,
        );

        let mut chain = EvolutionChain::new();
        chain.push(a1);
        chain.push(a2);

        let result = chain.verify();
        assert!(!result.valid);
        assert_eq!(result.break_at, Some(50));
    }

    #[test]
    fn test_cross_chain_valid() {
        use crate::anchor::{AnchorEngine, AnchorMode, WorldSummary};
        use crate::chain::AnchorChain;

        // Build a state chain with one anchor
        let mut state_engine = AnchorEngine::new(100, AnchorMode::Local, "test");
        let balances = vec![("a".into(), 100.0)];
        let summary = WorldSummary {
            epoch: 100,
            population: 5,
            total_supply: 500.0,
            treasury_reserve: 25.0,
            mean_fitness: 0.5,
            total_births: 5,
            total_deaths: 0,
            role_counts: vec![],
        };
        let sa = state_engine.anchor(100, &balances, &summary);
        let epoch_root = sa.epoch_root.clone();
        let mut state_chain = AnchorChain::new();
        state_chain.push(sa);

        // Build evolution chain referencing genesis root and known epoch root
        let mut evo_engine = EvolutionEngine::new("test");
        let mut evo_chain = EvolutionChain::new();

        let a1 = evo_engine.anchor(
            25, "{}", "{}",
            vec![dummy_mutation("x")], 1, "Watch", GENESIS_ROOT,
        );
        evo_chain.push(a1);

        let a2 = evo_engine.anchor(
            100, "{}", "{}",
            vec![dummy_mutation("y")], 1, "Watch", &epoch_root,
        );
        evo_chain.push(a2);

        let result = evo_chain.cross_verify(&state_chain);
        assert!(result.valid, "{}", result.message);
        assert_eq!(result.valid_cross_refs, 2);
        assert_eq!(result.orphaned_refs, 0);
    }

    #[test]
    fn test_cross_chain_orphaned() {
        use crate::chain::AnchorChain;

        let state_chain = AnchorChain::new(); // empty state chain
        let fake_root = "beef".repeat(16);

        let mut evo_engine = EvolutionEngine::new("test");
        let mut evo_chain = EvolutionChain::new();
        let a1 = evo_engine.anchor(
            25, "{}", "{}",
            vec![dummy_mutation("x")], 1, "Watch", &fake_root,
        );
        evo_chain.push(a1);

        let result = evo_chain.cross_verify(&state_chain);
        assert!(!result.valid);
        assert_eq!(result.orphaned_refs, 1);
    }

    #[test]
    fn test_evolution_root_deterministic() {
        let mut e1 = EvolutionEngine::new("test");
        let mut e2 = EvolutionEngine::new("test");
        let epoch_root = fake_epoch_root();

        let a1 = e1.anchor(
            25, r#"{"a":1}"#, r#"{"a":2}"#,
            vec![dummy_mutation("x")], 1, "Watch", &epoch_root,
        );
        let a2 = e2.anchor(
            25, r#"{"a":1}"#, r#"{"a":2}"#,
            vec![dummy_mutation("x")], 1, "Watch", &epoch_root,
        );

        // Same inputs → same evolution_root
        assert_eq!(a1.evolution_root, a2.evolution_root);
        assert_eq!(a1.before_hash, a2.before_hash);
        assert_eq!(a1.after_hash, a2.after_hash);
    }

    #[test]
    fn test_total_mutations_tracking() {
        let mut engine = EvolutionEngine::new("test");
        let epoch_root = fake_epoch_root();

        engine.anchor(
            25, "{}", "{}",
            vec![dummy_mutation("a"), dummy_mutation("b"), dummy_mutation("c")],
            3, "Critical", &epoch_root,
        );
        engine.anchor(
            50, "{}", "{}",
            vec![dummy_mutation("d")],
            1, "Watch", &epoch_root,
        );

        assert_eq!(engine.total_mutations, 4);
        assert_eq!(engine.total_events, 2);
    }

    #[test]
    fn test_history_window_cap() {
        let mut engine = EvolutionEngine {
            max_history: 3,
            ..EvolutionEngine::new("test")
        };
        let epoch_root = fake_epoch_root();

        for epoch in (25..=125).step_by(25) {
            engine.anchor(
                epoch as u64, "{}", "{}",
                vec![dummy_mutation("x")], 1, "Watch", &epoch_root,
            );
        }

        // Only last 3 should remain in memory
        assert_eq!(engine.history.len(), 3);
        assert_eq!(engine.history[0].epoch, 75);
        assert_eq!(engine.history[2].epoch, 125);
        // But total_events tracks all
        assert_eq!(engine.total_events, 5);
    }
}
