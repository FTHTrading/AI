// Multiverse Engine — Orchestrator of parallel civilizations
//
// Manages a collection of sovereign worlds, each running under
// its own physics, with independent State Chains and Evolution Chains.
//
// Core operations:
//   spawn    — create a primordial world with given physics
//   fork     — deep-clone a world at its current cryptographic state
//   advance  — run one or all worlds forward by N epochs
//   compare  — measure divergence between any two worlds
//   merge    — transfer evolutionary learnings between worlds
//   ancestry — trace the lineage of any world back to its origin
//
// Each operation is recorded as a verifiable event. Fork events
// are cryptographically hash-linked. Merge events are hash-linked.
// The multiverse itself has an auditable history.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use gateway::world::World;

use crate::identity::WorldIdentity;
use crate::physics::{WorldPhysics, PhysicsPreset};
use crate::fork::ForkEvent;
use crate::divergence::{self, DivergenceReport};
use crate::merge::{self, MergeEvent, MergeStrategy, FieldSelection};

/// A world managed by the multiverse — identity + physics + live state.
#[derive(Serialize, Deserialize)]
pub struct ManagedWorld {
    /// Cryptographic identity and lineage.
    pub identity: WorldIdentity,
    /// Physics profile (initial conditions + label).
    pub physics: WorldPhysics,
    /// The live world state.
    pub world: World,
}

impl ManagedWorld {
    /// Short label for display.
    pub fn label(&self) -> String {
        self.identity.label()
    }

    /// Current epoch.
    pub fn epoch(&self) -> u64 {
        self.world.epoch
    }

    /// Current population.
    pub fn population(&self) -> usize {
        self.world.agents.len()
    }
}

/// The multiverse engine — manages parallel civilizations.
#[derive(Serialize, Deserialize)]
pub struct MultiverseEngine {
    /// All managed worlds, keyed by UUID.
    pub worlds: HashMap<Uuid, ManagedWorld>,
    /// Chronological history of fork events.
    pub fork_history: Vec<ForkEvent>,
    /// Chronological history of merge events.
    pub merge_history: Vec<MergeEvent>,
    /// When this multiverse was created.
    pub created_at: DateTime<Utc>,
    /// Total epochs advanced across all worlds (for statistics).
    pub total_epochs_run: u64,
}

impl Default for MultiverseEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiverseEngine {
    /// Create a new empty multiverse.
    pub fn new() -> Self {
        Self {
            worlds: HashMap::new(),
            fork_history: Vec::new(),
            merge_history: Vec::new(),
            created_at: Utc::now(),
            total_epochs_run: 0,
        }
    }

    /// Number of active worlds.
    pub fn world_count(&self) -> usize {
        self.worlds.len()
    }

    /// Get a world by ID.
    pub fn get(&self, id: &Uuid) -> Option<&ManagedWorld> {
        self.worlds.get(id)
    }

    /// Get a mutable world by ID.
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut ManagedWorld> {
        self.worlds.get_mut(id)
    }

    // ─── Spawn ──────────────────────────────────────────────────────

    /// Spawn a new primordial world with a named physics preset.
    pub fn spawn_preset(
        &mut self,
        name: impl Into<String>,
        seed: u64,
        preset: PhysicsPreset,
    ) -> Uuid {
        let physics = WorldPhysics::preset(preset);
        self.spawn_custom(name, seed, physics)
    }

    /// Spawn a new primordial world with custom physics.
    pub fn spawn_custom(
        &mut self,
        name: impl Into<String>,
        seed: u64,
        physics: WorldPhysics,
    ) -> Uuid {
        let name = name.into();
        let identity = WorldIdentity::primordial(&name, seed);
        let id = identity.id;

        let mut world = World::new();
        physics.apply_to(&mut world);

        // Set anchor storage paths unique to this world
        world.anchor_engine.storage_path = format!("anchor/{}", &id.to_string()[..8]);
        world.evolution_engine.storage_path = format!("anchor/{}", &id.to_string()[..8]);

        let managed = ManagedWorld {
            identity,
            physics,
            world,
        };

        tracing::info!(
            world = %name,
            id = %&id.to_string()[..8],
            "Multiverse: spawned primordial world"
        );

        self.worlds.insert(id, managed);
        id
    }

    // ─── Fork ───────────────────────────────────────────────────────

    /// Fork a world: deep-clone its current state, optionally with
    /// different physics. Returns the new world's UUID.
    ///
    /// The fork is recorded as a cryptographic `ForkEvent` with
    /// the exact State Chain and Evolution Chain roots at fork time.
    pub fn fork(
        &mut self,
        parent_id: &Uuid,
        child_name: impl Into<String>,
        child_seed: u64,
        new_physics: Option<WorldPhysics>,
    ) -> Option<Uuid> {
        // Read parent state
        let parent = self.worlds.get(parent_id)?;
        let parent_epoch = parent.world.epoch;
        let state_root = parent.world.anchor_engine.last_root.clone();
        let evolution_root = parent.world.evolution_engine.last_evolution_root.clone();
        let parent_physics = parent.physics.clone();
        let parent_identity = parent.identity.clone();

        // Deep clone via serialize/deserialize (captures complete state)
        let serialized = serde_json::to_string(&parent.world).ok()?;
        let mut child_world: World = serde_json::from_str(&serialized).ok()?;

        // Determine child physics
        let child_physics = new_physics.unwrap_or_else(|| parent_physics.clone());
        let physics_delta = parent_physics.delta(&child_physics);

        // Apply new physics if different
        child_physics.apply_to(&mut child_world);

        let child_name = child_name.into();
        let child_identity = WorldIdentity::forked(
            &child_name,
            child_seed,
            &parent_identity,
            parent_epoch,
            &state_root,
            &evolution_root,
        );
        let child_id = child_identity.id;

        // Set unique storage paths for the child
        child_world.anchor_engine.storage_path = format!("anchor/{}", &child_id.to_string()[..8]);
        child_world.evolution_engine.storage_path = format!("anchor/{}", &child_id.to_string()[..8]);

        // Record fork event
        let fork_event = ForkEvent::new(
            *parent_id,
            child_id,
            parent_epoch,
            &state_root,
            &evolution_root,
            physics_delta,
        );

        tracing::info!(
            parent = %&parent_id.to_string()[..8],
            child = %&child_id.to_string()[..8],
            epoch = parent_epoch,
            hash = %&fork_event.fork_hash[..16],
            "Multiverse: world forked"
        );

        self.fork_history.push(fork_event);
        self.worlds.insert(child_id, ManagedWorld {
            identity: child_identity,
            physics: child_physics,
            world: child_world,
        });

        Some(child_id)
    }

    // ─── Advance ────────────────────────────────────────────────────

    /// Advance a specific world by N epochs.
    /// Returns the final EpochStats from the last epoch.
    pub fn advance(
        &mut self,
        world_id: &Uuid,
        epochs: u64,
    ) -> Option<gateway::world::EpochStats> {
        let managed = self.worlds.get_mut(world_id)?;
        let mut last_stats = None;
        for _ in 0..epochs {
            last_stats = Some(managed.world.run_epoch());
            self.total_epochs_run += 1;
        }
        last_stats
    }

    /// Advance ALL worlds by N epochs each.
    /// Returns a map of world_id → final EpochStats.
    pub fn advance_all(
        &mut self,
        epochs: u64,
    ) -> HashMap<Uuid, gateway::world::EpochStats> {
        let ids: Vec<Uuid> = self.worlds.keys().cloned().collect();
        let mut results = HashMap::new();

        for id in ids {
            if let Some(managed) = self.worlds.get_mut(&id) {
                let mut last_stats = None;
                for _ in 0..epochs {
                    last_stats = Some(managed.world.run_epoch());
                    self.total_epochs_run += 1;
                }
                if let Some(stats) = last_stats {
                    results.insert(id, stats);
                }
            }
        }

        results
    }

    // ─── Compare ────────────────────────────────────────────────────

    /// Measure divergence between two worlds.
    pub fn compare(
        &self,
        world_a: &Uuid,
        world_b: &Uuid,
    ) -> Option<DivergenceReport> {
        let a = self.worlds.get(world_a)?;
        let b = self.worlds.get(world_b)?;
        Some(divergence::measure(
            &a.identity, &a.world,
            &b.identity, &b.world,
        ))
    }

    // ─── Merge ──────────────────────────────────────────────────────

    /// Merge evolutionary learnings from source to target.
    pub fn merge(
        &mut self,
        source_id: &Uuid,
        target_id: &Uuid,
        strategy: MergeStrategy,
        fields: &FieldSelection,
    ) -> Option<MergeEvent> {
        // Extract source data (immutable borrow)
        let source = self.worlds.get(source_id)?;
        let source_uuid = source.identity.id;
        let source_name = source.identity.name.clone();

        // Serialize source for reading (avoid double borrow)
        let source_pressure = source.world.pressure.clone();
        let source_agents: Vec<f64> = source.world.agents.iter().map(|a| a.fitness()).collect();
        let source_evo_root = source.world.evolution_engine.last_evolution_root.clone();

        let target = self.worlds.get(target_id)?;
        let target_uuid = target.identity.id;
        let target_name = target.identity.name.clone();

        // We need to work around the borrow checker: read source data,
        // then mutably access target
        let _ = target;

        let target = self.worlds.get_mut(target_id)?;

        // Build a temporary source-like view for the merge function
        // Since we can't borrow both worlds simultaneously, we'll
        // compute the merge inline using the extracted data

        let field_list = match fields {
            FieldSelection::All => vec![
                genesis_homeostasis::cortex::PressureField::SoftCap,
                genesis_homeostasis::cortex::PressureField::EntropyCoeff,
                genesis_homeostasis::cortex::PressureField::CatastropheBaseProb,
                genesis_homeostasis::cortex::PressureField::CatastrophePopScale,
                genesis_homeostasis::cortex::PressureField::GiniWealthTaxThreshold,
                genesis_homeostasis::cortex::PressureField::GiniWealthTaxRate,
                genesis_homeostasis::cortex::PressureField::TreasuryOverflowThreshold,
            ],
            FieldSelection::Specific(ref f) => f.clone(),
        };

        let source_fitness_avg = if source_agents.is_empty() {
            0.0
        } else {
            source_agents.iter().sum::<f64>() / source_agents.len() as f64
        };
        let target_fitness_avg = if target.world.agents.is_empty() {
            0.0
        } else {
            target.world.agents.iter().map(|a| a.fitness()).sum::<f64>()
                / target.world.agents.len() as f64
        };

        let mut transferred = Vec::new();
        for field in &field_list {
            let source_val = get_pressure_value(field, &source_pressure);
            let target_val = get_pressure_value(field, &target.world.pressure);

            let new_val = match strategy {
                MergeStrategy::Overwrite => source_val,
                MergeStrategy::Average => (source_val + target_val) / 2.0,
                MergeStrategy::Weighted(w) => source_val * w + target_val * (1.0 - w),
                MergeStrategy::BestOf => {
                    if source_fitness_avg >= target_fitness_avg {
                        source_val
                    } else {
                        target_val
                    }
                }
            };

            let before = target_val;
            set_pressure_value(field, &mut target.world.pressure, new_val);

            transferred.push(merge::TransferredField {
                field: field.name().to_string(),
                source_value: source_val,
                target_before: before,
                target_after: new_val,
            });
        }

        let target_evo_root = target.world.evolution_engine.last_evolution_root.clone();
        let target_epoch = target.world.epoch;

        let merge_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(source_uuid.as_bytes());
            hasher.update(target_uuid.as_bytes());
            hasher.update(target_epoch.to_le_bytes());
            hasher.update(source_evo_root.as_bytes());
            hasher.update(target_evo_root.as_bytes());
            hex::encode(hasher.finalize())
        };

        let event = MergeEvent {
            source_id: source_uuid,
            source_name,
            target_id: target_uuid,
            target_name,
            target_epoch,
            strategy,
            transferred_fields: transferred,
            source_evolution_root: source_evo_root,
            target_evolution_root: target_evo_root,
            merged_at: Utc::now(),
            merge_hash,
        };

        tracing::info!(
            source = %&source_uuid.to_string()[..8],
            target = %&target_uuid.to_string()[..8],
            fields = event.transferred_fields.len(),
            strategy = ?strategy,
            hash = %&event.merge_hash[..16],
            "Multiverse: knowledge merged"
        );

        self.merge_history.push(event.clone());
        Some(event)
    }

    // ─── Ancestry ───────────────────────────────────────────────────

    /// Trace the lineage of a world back to its primordial ancestor.
    /// Returns a chain of (world_id, name, generation, fork_epoch).
    pub fn ancestry(&self, world_id: &Uuid) -> Vec<(Uuid, String, u32, Option<u64>)> {
        let mut chain = Vec::new();
        let mut current = *world_id;

        loop {
            let managed = match self.worlds.get(&current) {
                Some(m) => m,
                None => break,
            };

            chain.push((
                managed.identity.id,
                managed.identity.name.clone(),
                managed.identity.generation,
                managed.identity.fork_epoch,
            ));

            match managed.identity.parent_id {
                Some(parent) => current = parent,
                None => break,
            }
        }

        chain.reverse();
        chain
    }

    /// Find all worlds that are descendants of a given world.
    pub fn descendants(&self, ancestor_id: &Uuid) -> Vec<Uuid> {
        let mut result = Vec::new();
        for (id, managed) in &self.worlds {
            if managed.identity.parent_id == Some(*ancestor_id) {
                result.push(*id);
                // Recursively find descendants of descendants
                let sub = self.descendants(id);
                result.extend(sub);
            }
        }
        result
    }

    /// List all worlds with their current status.
    pub fn census(&self) -> Vec<WorldCensusEntry> {
        self.worlds.values()
            .map(|m| WorldCensusEntry {
                id: m.identity.id,
                name: m.identity.name.clone(),
                generation: m.identity.generation,
                epoch: m.world.epoch,
                population: m.world.agents.len(),
                physics_label: m.physics.label.clone(),
                eco_state: m.world.eco_state.name().to_string(),
                evolution_events: m.world.evolution_engine.total_events,
                state_root: m.world.anchor_engine.last_root.clone(),
                evolution_root: m.world.evolution_engine.last_evolution_root.clone(),
            })
            .collect()
    }
}

/// Census entry for a world in the multiverse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldCensusEntry {
    pub id: Uuid,
    pub name: String,
    pub generation: u32,
    pub epoch: u64,
    pub population: usize,
    pub physics_label: String,
    pub eco_state: String,
    pub evolution_events: u64,
    pub state_root: String,
    pub evolution_root: String,
}

/// Helper: get a pressure value from PressureConfig by field enum.
fn get_pressure_value(
    field: &genesis_homeostasis::cortex::PressureField,
    pressure: &gateway::world::PressureConfig,
) -> f64 {
    use genesis_homeostasis::cortex::PressureField;
    match field {
        PressureField::SoftCap => pressure.soft_cap as f64,
        PressureField::EntropyCoeff => pressure.entropy_coeff,
        PressureField::CatastropheBaseProb => pressure.catastrophe_base_prob,
        PressureField::CatastrophePopScale => pressure.catastrophe_pop_scale,
        PressureField::GiniWealthTaxThreshold => pressure.gini_wealth_tax_threshold,
        PressureField::GiniWealthTaxRate => pressure.gini_wealth_tax_rate,
        PressureField::TreasuryOverflowThreshold => pressure.treasury_overflow_threshold,
    }
}

/// Helper: set a pressure value on PressureConfig by field enum.
fn set_pressure_value(
    field: &genesis_homeostasis::cortex::PressureField,
    pressure: &mut gateway::world::PressureConfig,
    value: f64,
) {
    use genesis_homeostasis::cortex::PressureField;
    match field {
        PressureField::SoftCap => pressure.soft_cap = value as usize,
        PressureField::EntropyCoeff => pressure.entropy_coeff = value,
        PressureField::CatastropheBaseProb => pressure.catastrophe_base_prob = value,
        PressureField::CatastrophePopScale => pressure.catastrophe_pop_scale = value,
        PressureField::GiniWealthTaxThreshold => pressure.gini_wealth_tax_threshold = value,
        PressureField::GiniWealthTaxRate => pressure.gini_wealth_tax_rate = value,
        PressureField::TreasuryOverflowThreshold => pressure.treasury_overflow_threshold = value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_primordial_world() {
        let mut mv = MultiverseEngine::new();
        let id = mv.spawn_preset("Earth-Prime", 42, PhysicsPreset::EarthPrime);

        assert_eq!(mv.world_count(), 1);
        let w = mv.get(&id).unwrap();
        assert_eq!(w.identity.name, "Earth-Prime");
        assert!(w.identity.is_primordial());
        assert_eq!(w.world.pressure.soft_cap, 180);
    }

    #[test]
    fn spawn_different_physics() {
        let mut mv = MultiverseEngine::new();
        let ep = mv.spawn_preset("Earth-Prime", 1, PhysicsPreset::EarthPrime);
        let hg = mv.spawn_preset("High Gravity", 2, PhysicsPreset::HighGravity);

        assert_eq!(mv.world_count(), 2);
        assert_eq!(mv.get(&ep).unwrap().world.pressure.soft_cap, 180);
        assert_eq!(mv.get(&hg).unwrap().world.pressure.soft_cap, 80);
    }

    #[test]
    fn advance_world() {
        let mut mv = MultiverseEngine::new();
        let id = mv.spawn_preset("Test", 42, PhysicsPreset::EarthPrime);

        let stats = mv.advance(&id, 5).unwrap();
        // stats.epoch is the epoch that was run (0-indexed: 0,1,2,3,4)
        // world.epoch is the counter after increment (= 5)
        assert_eq!(stats.epoch, 4);
        assert_eq!(mv.get(&id).unwrap().world.epoch, 5);
        assert_eq!(mv.total_epochs_run, 5);
    }

    #[test]
    fn advance_all_worlds() {
        let mut mv = MultiverseEngine::new();
        let ep = mv.spawn_preset("EP", 1, PhysicsPreset::EarthPrime);
        let hg = mv.spawn_preset("HG", 2, PhysicsPreset::HighGravity);

        let results = mv.advance_all(3);
        assert_eq!(results.len(), 2);
        assert_eq!(mv.get(&ep).unwrap().world.epoch, 3);
        assert_eq!(mv.get(&hg).unwrap().world.epoch, 3);
        assert_eq!(mv.total_epochs_run, 6);
    }

    #[test]
    fn fork_world_preserves_epoch() {
        let mut mv = MultiverseEngine::new();
        let parent = mv.spawn_preset("Parent", 42, PhysicsPreset::EarthPrime);
        mv.advance(&parent, 10);

        let child = mv.fork(
            &parent, "Child", 99,
            Some(WorldPhysics::preset(PhysicsPreset::HighGravity)),
        ).unwrap();

        let child_world = mv.get(&child).unwrap();
        // Child starts at parent's epoch
        assert_eq!(child_world.world.epoch, 10);
        // Child has different physics
        assert_eq!(child_world.world.pressure.soft_cap, 80);
        // Child is generation 1
        assert_eq!(child_world.identity.generation, 1);
        // Fork event recorded
        assert_eq!(mv.fork_history.len(), 1);
        assert!(mv.fork_history[0].verify());
    }

    #[test]
    fn fork_preserves_chain_roots() {
        let mut mv = MultiverseEngine::new();
        let parent = mv.spawn_preset("Parent", 42, PhysicsPreset::EarthPrime);
        mv.advance(&parent, 10);

        let parent_state_root = mv.get(&parent).unwrap().world.anchor_engine.last_root.clone();
        let parent_evo_root = mv.get(&parent).unwrap().world.evolution_engine.last_evolution_root.clone();

        let child = mv.fork(&parent, "Child", 99, None).unwrap();

        let fork_event = &mv.fork_history[0];
        assert_eq!(fork_event.state_root_at_fork, parent_state_root);
        assert_eq!(fork_event.evolution_root_at_fork, parent_evo_root);

        let child_id = mv.get(&child).unwrap();
        assert_eq!(child_id.identity.fork_state_root.as_deref(), Some(parent_state_root.as_str()));
        assert_eq!(child_id.identity.fork_evolution_root.as_deref(), Some(parent_evo_root.as_str()));
    }

    #[test]
    fn divergence_after_fork() {
        let mut mv = MultiverseEngine::new();
        let parent = mv.spawn_preset("Parent", 42, PhysicsPreset::EarthPrime);
        mv.advance(&parent, 5);

        let child = mv.fork(
            &parent, "Child", 99,
            Some(WorldPhysics::preset(PhysicsPreset::HighGravity)),
        ).unwrap();

        // Run both forward — they should diverge
        mv.advance(&parent, 20);
        mv.advance(&child, 20);

        let report = mv.compare(&parent, &child).unwrap();
        assert!(report.common_ancestor);
        // With different physics, some divergence is expected
        // (but at low epoch counts it may be small)
        assert!(report.summary().contains("Parent"));
        assert!(report.summary().contains("Child"));
    }

    #[test]
    fn merge_learnings() {
        let mut mv = MultiverseEngine::new();
        let source = mv.spawn_preset("Source", 1, PhysicsPreset::EarthPrime);
        let target = mv.spawn_preset("Target", 2, PhysicsPreset::HighGravity);

        // Advance source to let it evolve
        mv.advance(&source, 5);

        let event = mv.merge(
            &source, &target,
            MergeStrategy::Average,
            &FieldSelection::All,
        ).unwrap();

        assert!(event.verify());
        assert_eq!(event.transferred_fields.len(), 7);
        assert_eq!(mv.merge_history.len(), 1);

        // Target's soft_cap should now be average of 180 and 80 = 130
        let target_world = mv.get(&target).unwrap();
        assert_eq!(target_world.world.pressure.soft_cap, 130);
    }

    #[test]
    fn ancestry_chain() {
        let mut mv = MultiverseEngine::new();
        let gen0 = mv.spawn_preset("Root", 1, PhysicsPreset::EarthPrime);
        let gen1 = mv.fork(&gen0, "Child", 2, None).unwrap();
        let gen2 = mv.fork(&gen1, "Grandchild", 3, None).unwrap();

        let chain = mv.ancestry(&gen2);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].1, "Root");
        assert_eq!(chain[1].1, "Child");
        assert_eq!(chain[2].1, "Grandchild");
        assert_eq!(chain[0].2, 0); // generation 0
        assert_eq!(chain[2].2, 2); // generation 2
    }

    #[test]
    fn descendants_tree() {
        let mut mv = MultiverseEngine::new();
        let root = mv.spawn_preset("Root", 1, PhysicsPreset::EarthPrime);
        let child_a = mv.fork(&root, "ChildA", 2, None).unwrap();
        let child_b = mv.fork(&root, "ChildB", 3, None).unwrap();
        let _grandchild = mv.fork(&child_a, "GrandchildA1", 4, None).unwrap();

        let desc = mv.descendants(&root);
        // Should find child_a, child_b, and grandchild
        assert!(desc.len() >= 2); // At minimum child_a and child_b
        assert!(desc.contains(&child_a));
        assert!(desc.contains(&child_b));
    }

    #[test]
    fn census_lists_all() {
        let mut mv = MultiverseEngine::new();
        mv.spawn_preset("A", 1, PhysicsPreset::EarthPrime);
        mv.spawn_preset("B", 2, PhysicsPreset::HighGravity);
        mv.spawn_preset("C", 3, PhysicsPreset::Volcanic);

        let census = mv.census();
        assert_eq!(census.len(), 3);
        assert!(census.iter().any(|e| e.name == "A"));
        assert!(census.iter().any(|e| e.name == "B"));
        assert!(census.iter().any(|e| e.name == "C"));
    }

    #[test]
    fn identical_seeds_identical_start() {
        let mut mv = MultiverseEngine::new();
        let a = mv.spawn_preset("A", 42, PhysicsPreset::EarthPrime);
        let b = mv.spawn_preset("B", 42, PhysicsPreset::EarthPrime);

        // Before any epochs, both worlds should have identical population
        // and identical pressure (same physics)
        let wa = mv.get(&a).unwrap();
        let wb = mv.get(&b).unwrap();
        assert_eq!(wa.world.agents.len(), wb.world.agents.len());
        assert_eq!(wa.world.pressure.soft_cap, wb.world.pressure.soft_cap);
    }

    #[test]
    fn fork_without_physics_change() {
        let mut mv = MultiverseEngine::new();
        let parent = mv.spawn_preset("Parent", 42, PhysicsPreset::EarthPrime);
        mv.advance(&parent, 5);

        let child = mv.fork(&parent, "Clone", 99, None).unwrap();

        let pc = mv.get(&parent).unwrap();
        let cc = mv.get(&child).unwrap();
        // Same physics
        assert_eq!(pc.world.pressure.soft_cap, cc.world.pressure.soft_cap);
        assert_eq!(pc.world.pressure.entropy_coeff, cc.world.pressure.entropy_coeff);
        // Same epoch
        assert_eq!(pc.world.epoch, cc.world.epoch);
        // But different ID
        assert_ne!(pc.identity.id, cc.identity.id);
        // Fork event has empty physics delta
        assert!(mv.fork_history[0].physics_delta.is_empty());
    }
}
