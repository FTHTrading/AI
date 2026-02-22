// Merge — Cross-world knowledge transfer
//
// After worlds evolve independently, their cortex may discover
// different pressure optima. Merge transfers evolved pressure
// parameters from a source world to a target world.
//
// This is NOT world unification. Agents don't move.
// Resources don't transfer. Only the *lessons* transfer:
// the evolved PressureConfig parameters that the cortex learned.
//
// Strategies:
//   Overwrite  — replace target fields with source values
//   Average    — blend source and target 50/50
//   Weighted   — blend with configurable source weight
//   BestOf     — take whichever world has better health metrics
//
// Each merge is a cryptographic event with a deterministic hash,
// making knowledge transfer auditable.

use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use gateway::world::World;
use genesis_homeostasis::cortex::PressureField;

/// Strategy for blending pressure parameters during a merge.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Replace target values with source values.
    Overwrite,
    /// Average source and target (50/50).
    Average,
    /// Weighted blend: value = source * weight + target * (1 - weight).
    Weighted(f64),
    /// Take the value from whichever world has higher mean fitness.
    BestOf,
}

/// Cryptographic record of a knowledge transfer between worlds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeEvent {
    /// Source world (donor of knowledge).
    pub source_id: Uuid,
    pub source_name: String,
    /// Target world (recipient of knowledge).
    pub target_id: Uuid,
    pub target_name: String,
    /// Epoch of the target world when merge occurred.
    pub target_epoch: u64,
    /// Strategy used for the merge.
    pub strategy: MergeStrategy,
    /// Fields that were transferred.
    pub transferred_fields: Vec<TransferredField>,
    /// Source world's evolution root at time of merge.
    pub source_evolution_root: String,
    /// Target world's evolution root at time of merge.
    pub target_evolution_root: String,
    /// Timestamp of the merge.
    pub merged_at: DateTime<Utc>,
    /// Deterministic hash of the merge event.
    pub merge_hash: String,
}

/// Record of a single field transfer during a merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferredField {
    /// Field name.
    pub field: String,
    /// Source value.
    pub source_value: f64,
    /// Target value before merge.
    pub target_before: f64,
    /// Target value after merge.
    pub target_after: f64,
}

/// Which pressure fields to transfer.
#[derive(Debug, Clone)]
pub enum FieldSelection {
    /// Transfer all pressure fields.
    All,
    /// Transfer specific fields.
    Specific(Vec<PressureField>),
}

impl MergeEvent {
    /// Verify the merge hash (tamper detection).
    pub fn verify(&self) -> bool {
        let expected = compute_merge_hash(
            &self.source_id,
            &self.target_id,
            self.target_epoch,
            &self.source_evolution_root,
            &self.target_evolution_root,
        );
        self.merge_hash == expected
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "Merge: {} → {} at epoch {} ({} fields, {:?}, hash: {}..)",
            self.source_name,
            self.target_name,
            self.target_epoch,
            self.transferred_fields.len(),
            self.strategy,
            &self.merge_hash[..16],
        )
    }
}

/// Execute a knowledge transfer from source world to target world.
///
/// Returns a `MergeEvent` recording the transfer, and mutates
/// the target world's `PressureConfig` according to the strategy.
pub fn execute(
    source_id: Uuid,
    source_name: &str,
    source: &World,
    target_id: Uuid,
    target_name: &str,
    target: &mut World,
    strategy: MergeStrategy,
    fields: &FieldSelection,
) -> MergeEvent {
    let field_list = match fields {
        FieldSelection::All => vec![
            PressureField::SoftCap,
            PressureField::EntropyCoeff,
            PressureField::CatastropheBaseProb,
            PressureField::CatastrophePopScale,
            PressureField::GiniWealthTaxThreshold,
            PressureField::GiniWealthTaxRate,
            PressureField::TreasuryOverflowThreshold,
        ],
        FieldSelection::Specific(ref fields) => fields.clone(),
    };

    // Compute mean fitness for BestOf strategy
    let source_fitness = if source.agents.is_empty() {
        0.0
    } else {
        source.agents.iter().map(|a| a.fitness()).sum::<f64>() / source.agents.len() as f64
    };
    let target_fitness = if target.agents.is_empty() {
        0.0
    } else {
        target.agents.iter().map(|a| a.fitness()).sum::<f64>() / target.agents.len() as f64
    };

    let mut transferred = Vec::new();

    for field in &field_list {
        let (source_val, target_val) = get_field_values(field, source, target);

        let new_val = match strategy {
            MergeStrategy::Overwrite => source_val,
            MergeStrategy::Average => (source_val + target_val) / 2.0,
            MergeStrategy::Weighted(w) => source_val * w + target_val * (1.0 - w),
            MergeStrategy::BestOf => {
                if source_fitness >= target_fitness {
                    source_val
                } else {
                    target_val
                }
            }
        };

        let before = target_val;
        apply_field(field, target, new_val);

        transferred.push(TransferredField {
            field: field.name().to_string(),
            source_value: source_val,
            target_before: before,
            target_after: new_val,
        });
    }

    let source_evo_root = source.evolution_engine.last_evolution_root.clone();
    let target_evo_root = target.evolution_engine.last_evolution_root.clone();

    let merge_hash = compute_merge_hash(
        &source_id,
        &target_id,
        target.epoch,
        &source_evo_root,
        &target_evo_root,
    );

    MergeEvent {
        source_id,
        source_name: source_name.to_string(),
        target_id,
        target_name: target_name.to_string(),
        target_epoch: target.epoch,
        strategy,
        transferred_fields: transferred,
        source_evolution_root: source_evo_root,
        target_evolution_root: target_evo_root,
        merged_at: Utc::now(),
        merge_hash,
    }
}

/// Get source and target values for a pressure field.
fn get_field_values(field: &PressureField, source: &World, target: &World) -> (f64, f64) {
    match field {
        PressureField::SoftCap => (source.pressure.soft_cap as f64, target.pressure.soft_cap as f64),
        PressureField::EntropyCoeff => (source.pressure.entropy_coeff, target.pressure.entropy_coeff),
        PressureField::CatastropheBaseProb => (source.pressure.catastrophe_base_prob, target.pressure.catastrophe_base_prob),
        PressureField::CatastrophePopScale => (source.pressure.catastrophe_pop_scale, target.pressure.catastrophe_pop_scale),
        PressureField::GiniWealthTaxThreshold => (source.pressure.gini_wealth_tax_threshold, target.pressure.gini_wealth_tax_threshold),
        PressureField::GiniWealthTaxRate => (source.pressure.gini_wealth_tax_rate, target.pressure.gini_wealth_tax_rate),
        PressureField::TreasuryOverflowThreshold => (source.pressure.treasury_overflow_threshold, target.pressure.treasury_overflow_threshold),
    }
}

/// Apply a value to a pressure field on a world.
fn apply_field(field: &PressureField, world: &mut World, value: f64) {
    match field {
        PressureField::SoftCap => world.pressure.soft_cap = value as usize,
        PressureField::EntropyCoeff => world.pressure.entropy_coeff = value,
        PressureField::CatastropheBaseProb => world.pressure.catastrophe_base_prob = value,
        PressureField::CatastrophePopScale => world.pressure.catastrophe_pop_scale = value,
        PressureField::GiniWealthTaxThreshold => world.pressure.gini_wealth_tax_threshold = value,
        PressureField::GiniWealthTaxRate => world.pressure.gini_wealth_tax_rate = value,
        PressureField::TreasuryOverflowThreshold => world.pressure.treasury_overflow_threshold = value,
    }
}

/// Deterministic hash of a merge event.
fn compute_merge_hash(
    source_id: &Uuid,
    target_id: &Uuid,
    epoch: u64,
    source_evo_root: &str,
    target_evo_root: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_id.as_bytes());
    hasher.update(target_id.as_bytes());
    hasher.update(epoch.to_le_bytes());
    hasher.update(source_evo_root.as_bytes());
    hasher.update(target_evo_root.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overwrite_merge() {
        let _source = World::new();
        let mut target = World::new();

        // Modify source pressure (simulating evolution)
        let mut modified_source = World::new();
        modified_source.pressure.soft_cap = 100;
        modified_source.pressure.entropy_coeff = 0.00005;

        let event = execute(
            Uuid::new_v4(), "Source",
            &modified_source,
            Uuid::new_v4(), "Target",
            &mut target,
            MergeStrategy::Overwrite,
            &FieldSelection::All,
        );

        assert_eq!(target.pressure.soft_cap, 100);
        assert_eq!(target.pressure.entropy_coeff, 0.00005);
        assert!(event.verify());
        assert_eq!(event.transferred_fields.len(), 7);
    }

    #[test]
    fn average_merge() {
        let _source = World::new();
        let mut target = World::new();

        let mut modified_source = World::new();
        modified_source.pressure.soft_cap = 100; // default is 180

        let _ = execute(
            Uuid::new_v4(), "Source",
            &modified_source,
            Uuid::new_v4(), "Target",
            &mut target,
            MergeStrategy::Average,
            &FieldSelection::All,
        );

        // Average of 100 and 180 = 140
        assert_eq!(target.pressure.soft_cap, 140);
    }

    #[test]
    fn weighted_merge() {
        let mut modified_source = World::new();
        modified_source.pressure.soft_cap = 100;

        let mut target = World::new();
        // target soft_cap = 180

        let _ = execute(
            Uuid::new_v4(), "Source",
            &modified_source,
            Uuid::new_v4(), "Target",
            &mut target,
            MergeStrategy::Weighted(0.25),
            &FieldSelection::All,
        );

        // 100 * 0.25 + 180 * 0.75 = 25 + 135 = 160
        assert_eq!(target.pressure.soft_cap, 160);
    }

    #[test]
    fn selective_field_merge() {
        let mut modified_source = World::new();
        modified_source.pressure.soft_cap = 100;
        modified_source.pressure.entropy_coeff = 0.00005;
        modified_source.pressure.catastrophe_base_prob = 0.999;

        let mut target = World::new();
        let original_cat_prob = target.pressure.catastrophe_base_prob;

        let event = execute(
            Uuid::new_v4(), "Source",
            &modified_source,
            Uuid::new_v4(), "Target",
            &mut target,
            MergeStrategy::Overwrite,
            &FieldSelection::Specific(vec![
                PressureField::SoftCap,
                PressureField::EntropyCoeff,
            ]),
        );

        assert_eq!(target.pressure.soft_cap, 100);
        assert_eq!(target.pressure.entropy_coeff, 0.00005);
        // catastrophe_base_prob should NOT have changed
        assert_eq!(target.pressure.catastrophe_base_prob, original_cat_prob);
        assert_eq!(event.transferred_fields.len(), 2);
    }

    #[test]
    fn merge_hash_tamper_detection() {
        let source = World::new();
        let mut target = World::new();

        let mut event = execute(
            Uuid::new_v4(), "Source",
            &source,
            Uuid::new_v4(), "Target",
            &mut target,
            MergeStrategy::Average,
            &FieldSelection::All,
        );

        assert!(event.verify());

        // Tamper
        event.target_epoch = 999;
        assert!(!event.verify());
    }

    #[test]
    fn merge_summary_readable() {
        let source = World::new();
        let mut target = World::new();

        let event = execute(
            Uuid::new_v4(), "Earth-Prime",
            &source,
            Uuid::new_v4(), "High Gravity",
            &mut target,
            MergeStrategy::Overwrite,
            &FieldSelection::All,
        );

        let s = event.summary();
        assert!(s.contains("Earth-Prime"));
        assert!(s.contains("High Gravity"));
        assert!(s.contains("7 fields"));
    }
}
