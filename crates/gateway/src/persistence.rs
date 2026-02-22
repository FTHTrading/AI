// Persistence — JSON Snapshot Save/Load
//
// Serializes the entire World state to a JSON file and restores it.
// First version uses flat JSON files — upgradeable to RocksDB later.
// Autosave every N epochs without blocking the survival loop.

use std::path::Path;

use crate::world::World;

/// Default save path.
const DEFAULT_PATH: &str = "world_state.json";

/// Save world state to JSON file.
pub fn save(world: &World) -> Result<(), String> {
    save_to(world, DEFAULT_PATH)
}

/// Save world state to a specific path.
pub fn save_to(world: &World, path: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(world)
        .map_err(|e| format!("Serialization failed: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("Write failed: {}", e))?;
    Ok(())
}

/// Load world state from default path.
pub fn load() -> Option<World> {
    load_from(DEFAULT_PATH)
}

/// Load world state from a specific path.
pub fn load_from(path: &str) -> Option<World> {
    if !Path::new(path).exists() {
        return None;
    }
    let data = std::fs::read_to_string(path).ok()?;
    let mut world: World = serde_json::from_str(&data).ok()?;
    // Repair environment pools lost from old snapshots
    world.repair_environment();
    Some(world)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_roundtrip() {
        let mut world = World::new();
        // Run a few epochs to create interesting state
        for _ in 0..5 {
            world.run_epoch();
        }

        let path = "test_world_roundtrip.json";
        save_to(&world, path).expect("Save should succeed");

        let loaded = load_from(path).expect("Load should return Some");
        assert_eq!(loaded.epoch, world.epoch);
        assert_eq!(loaded.agents.len(), world.agents.len());
        assert!((loaded.ledger.total_supply() - world.ledger.total_supply()).abs() < 0.01);
        assert!((loaded.treasury.reserve - world.treasury.reserve).abs() < 0.01);

        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_load_nonexistent_returns_none() {
        let result = load_from("nonexistent_file_xyz.json");
        assert!(result.is_none());
    }

    #[test]
    fn test_loaded_world_can_continue() {
        let mut world = World::new();
        for _ in 0..3 {
            world.run_epoch();
        }

        let path = "test_world_continue.json";
        save_to(&world, path).expect("Save should succeed");

        let mut loaded = load_from(path).expect("Load should return Some");
        assert_eq!(loaded.epoch, 3);

        // Run more epochs on loaded world
        let stats = loaded.run_epoch();
        assert_eq!(stats.epoch, 3); // epoch was 3 when run, increments to 4
        assert_eq!(loaded.epoch, 4);

        // Cleanup
        let _ = std::fs::remove_file(path);
    }
}
