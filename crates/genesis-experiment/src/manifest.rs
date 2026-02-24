// Replay Manifest — Reproducible experiment specification
//
// A ReplayManifest contains everything needed to reproduce an experiment:
//   - Experiment configuration (variable, range, metrics)
//   - Physics preset
//   - Base seed
//   - Result hash (for verification)
//
// Given a manifest, anyone can:
//   1. Reconstruct the exact experiment
//   2. Run it independently
//   3. Verify the result hash matches
//
// This is how Genesis becomes peer-verifiable.

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use crate::config::ExperimentConfig;
use crate::runner::ExperimentResult;

/// A reproducible experiment manifest.
///
/// Contains the complete specification needed to replicate an experiment
/// and the hash to verify results match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayManifest {
    /// Unique manifest identifier.
    pub manifest_id: String,
    /// The experiment configuration (complete specification).
    pub config: ExperimentConfig,
    /// When this manifest was created.
    pub created_at: DateTime<Utc>,
    /// SHA-256 hash of the experiment results.
    pub result_hash: String,
    /// SHA-256 hash of this manifest itself (for anchoring).
    pub manifest_hash: String,
    /// Total worlds in the experiment.
    pub total_worlds: usize,
    /// Total epochs run.
    pub total_epochs: u64,
    /// Experiment duration in milliseconds.
    pub duration_ms: i64,
    /// Key findings summary (human-readable).
    pub findings: Vec<String>,
    /// Genesis Protocol version.
    pub protocol_version: String,
}

impl ReplayManifest {
    /// Create a manifest from an experiment result.
    pub fn from_result(result: &ExperimentResult, findings: Vec<String>) -> Self {
        let duration_ms = (result.finished_at - result.started_at).num_milliseconds();

        // Generate manifest ID from config hash
        let manifest_id = {
            let mut hasher = Sha256::new();
            hasher.update(result.config.name.as_bytes());
            hasher.update(result.config.base_seed.to_le_bytes());
            hasher.update(result.started_at.timestamp().to_le_bytes());
            let hash = hex::encode(hasher.finalize());
            format!("GEN-EXP-{}", &hash[..12].to_uppercase())
        };

        let mut manifest = Self {
            manifest_id,
            config: result.config.clone(),
            created_at: Utc::now(),
            result_hash: result.result_hash.clone(),
            manifest_hash: String::new(), // computed below
            total_worlds: result.total_worlds,
            total_epochs: result.total_epochs_run,
            duration_ms,
            findings,
            protocol_version: "0.1.0".into(),
        };

        manifest.manifest_hash = manifest.compute_manifest_hash();
        manifest
    }

    /// Compute SHA-256 hash of the manifest contents.
    fn compute_manifest_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.manifest_id.as_bytes());
        hasher.update(self.result_hash.as_bytes());
        hasher.update(self.config.name.as_bytes());
        hasher.update(self.config.base_seed.to_le_bytes());
        hasher.update(self.config.epochs_per_run.to_le_bytes());
        hasher.update((self.config.runs_per_step as u64).to_le_bytes());
        hasher.update(self.config.sweep.variable.name().as_bytes());
        hasher.update(self.config.sweep.start.to_le_bytes());
        hasher.update(self.config.sweep.end.to_le_bytes());
        hasher.update(self.config.sweep.step.to_le_bytes());
        hasher.update(self.protocol_version.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify that a manifest hash is valid (self-consistent).
    pub fn verify(&self) -> bool {
        self.manifest_hash == self.compute_manifest_hash()
    }

    /// Re-run the experiment from this manifest's config.
    /// Returns true if the result hash matches.
    pub fn replay_and_verify(&self) -> (ExperimentResult, bool) {
        let result = crate::runner::ExperimentRunner::run(&self.config);
        let matches = result.result_hash == self.result_hash;
        (result, matches)
    }

    /// Export manifest as JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Parse manifest from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("═══ Replay Manifest: {} ═══\n", self.manifest_id));
        s.push_str(&format!("Experiment: {}\n", self.config.name));
        s.push_str(&format!("Hypothesis: {}\n", self.config.hypothesis));
        s.push_str(&format!("Variable: {} [{:.6} → {:.6}, step {:.6}]\n",
            self.config.sweep.variable.name(),
            self.config.sweep.start,
            self.config.sweep.end,
            self.config.sweep.step,
        ));
        s.push_str(&format!("Runs/step: {} | Epochs/run: {} | Total worlds: {}\n",
            self.config.runs_per_step,
            self.config.epochs_per_run,
            self.total_worlds,
        ));
        s.push_str(&format!("Base preset: {:?}\n", self.config.base_preset));
        s.push_str(&format!("Base seed: {}\n", self.config.base_seed));
        s.push_str(&format!("Duration: {}ms\n", self.duration_ms));
        s.push_str(&format!("Result hash: {}\n", &self.result_hash[..16]));
        s.push_str(&format!("Manifest hash: {}\n", &self.manifest_hash[..16]));

        if !self.findings.is_empty() {
            s.push_str("\nFindings:\n");
            for (i, finding) in self.findings.iter().enumerate() {
                s.push_str(&format!("  {}. {}\n", i + 1, finding));
            }
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ParameterSweep, Metric, SweepVariable};
    use genesis_multiverse::PhysicsPreset;

    fn mini_config() -> ExperimentConfig {
        ExperimentConfig {
            name: "Manifest Test".into(),
            hypothesis: "Manifests work".into(),
            sweep: ParameterSweep::new(SweepVariable::EntropyCoeff, 0.00002, 0.00002, 0.00001),
            runs_per_step: 1,
            epochs_per_run: 5,
            metrics: vec![Metric::FinalPopulation, Metric::Collapsed],
            base_preset: PhysicsPreset::EarthPrime,
            base_pressure_override: None,
            mutation_rate_override: None,
            cortex_enabled_override: None,
            base_stress_override: None,
            base_seed: 42,
        }
    }

    #[test]
    fn manifest_from_result() {
        let config = mini_config();
        let result = crate::runner::ExperimentRunner::run(&config);
        let manifest = ReplayManifest::from_result(&result, vec!["Test finding".into()]);

        assert!(manifest.manifest_id.starts_with("GEN-EXP-"));
        assert!(!manifest.result_hash.is_empty());
        assert!(!manifest.manifest_hash.is_empty());
        assert_eq!(manifest.findings.len(), 1);
    }

    #[test]
    fn manifest_self_verification() {
        let config = mini_config();
        let result = crate::runner::ExperimentRunner::run(&config);
        let manifest = ReplayManifest::from_result(&result, vec![]);
        assert!(manifest.verify(), "Manifest should self-verify");
    }

    #[test]
    fn manifest_tamper_detection() {
        let config = mini_config();
        let result = crate::runner::ExperimentRunner::run(&config);
        let mut manifest = ReplayManifest::from_result(&result, vec![]);
        // Tamper with the result hash
        manifest.result_hash = "tampered".into();
        // Manifest hash should no longer verify because it was computed with original hash
        // But we need to check that recomputation detects the change
        assert!(!manifest.verify());
    }

    #[test]
    fn manifest_json_roundtrip() {
        let config = mini_config();
        let result = crate::runner::ExperimentRunner::run(&config);
        let manifest = ReplayManifest::from_result(&result, vec!["Finding 1".into()]);

        let json = manifest.to_json();
        let parsed = ReplayManifest::from_json(&json).unwrap();

        assert_eq!(parsed.manifest_id, manifest.manifest_id);
        assert_eq!(parsed.result_hash, manifest.result_hash);
        assert_eq!(parsed.manifest_hash, manifest.manifest_hash);
        assert!(parsed.verify());
    }

    #[test]
    fn manifest_summary_readable() {
        let config = mini_config();
        let result = crate::runner::ExperimentRunner::run(&config);
        let manifest = ReplayManifest::from_result(&result, vec!["Discovery A".into()]);
        let summary = manifest.summary();

        assert!(summary.contains("Manifest Test"));
        assert!(summary.contains("GEN-EXP-"));
        assert!(summary.contains("entropy_coeff"));
        assert!(summary.contains("Discovery A"));
    }
}
