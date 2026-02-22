// Runtime — Background Survival Loop
//
// Spawns a thread that ticks the world forward one epoch per interval.
// Autosaves every N epochs. The survival loop continues even if
// individual HTTP handler panics.

use std::time::Duration;

use crate::moltbot::EpochSnapshot;
use crate::persistence;
use crate::world::{SharedWorld, World};

/// Default epoch interval (1 second).
const EPOCH_INTERVAL: Duration = Duration::from_secs(1);

/// Autosave every N epochs.
const AUTOSAVE_INTERVAL: u64 = 25;

/// Start the background survival loop on a dedicated thread.
/// Returns the join handle.
pub fn start_background_loop(world: SharedWorld) -> std::thread::JoinHandle<()> {
    start_background_loop_with_adapter(world, None)
}

/// Start the background survival loop with an optional Moltbot adapter channel.
/// When a sender is provided, epoch snapshots are sent to the async adapter task.
pub fn start_background_loop_with_adapter(
    world: SharedWorld,
    moltbot_tx: Option<tokio::sync::mpsc::Sender<EpochSnapshot>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            let (snapshot, docs_json, is_extinct) = {
                // Lock, tick, extract, release — keep lock duration minimal
                let mut w = match world.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        // Mutex poisoned — recover the inner value
                        tracing::warn!("World mutex was poisoned, recovering");
                        poisoned.into_inner()
                    }
                };

                let stats = w.run_epoch();

                if stats.epoch % 10 == 0 {
                    tracing::info!(
                        epoch = stats.epoch,
                        pop = stats.population,
                        atp = format!("{:.1}", stats.total_atp),
                        "Epoch tick"
                    );
                }

                // Extract snapshot for adapter (while we still hold the lock)
                let snapshot = if moltbot_tx.is_some() {
                    let leader = w.leaderboard(1).into_iter().next();
                    let telemetry = w.telemetry();
                    let risks: Vec<String> = telemetry.risks
                        .iter()
                        .map(|r| format!("{:?}", r))
                        .collect();

                    Some(EpochSnapshot {
                        stats: stats.clone(),
                        leader,
                        risks,
                        treasury_reserve: w.treasury.reserve,
                        uptime_seconds: w.uptime_seconds(),
                        total_births: w.total_births,
                        total_deaths: w.total_deaths,
                    })
                } else {
                    None
                };

                // Autosave + docs snapshot every AUTOSAVE_INTERVAL epochs
                let docs_json: Option<String> = if w.epoch % AUTOSAVE_INTERVAL == 0 {
                    if let Err(e) = persistence::save(&w) {
                        tracing::error!("Autosave failed: {}", e);
                    }
                    build_docs_snapshot(&w)
                } else {
                    None
                };

                let is_extinct = w.agents.is_empty();
                if is_extinct {
                    tracing::error!("EXTINCTION EVENT at epoch {}", w.epoch);
                }

                (snapshot, docs_json, is_extinct)
            };
            // Lock released here

            // Exit after extinction (after releasing lock)
            if is_extinct { return; }

            // Write docs/system-state.json for GitHub Pages dashboard
            if let Some(ref json) = docs_json {
                if let Err(e) = std::fs::write("docs/system-state.json", json) {
                    tracing::debug!("docs snapshot write failed: {}", e);
                }
            }

            // Send snapshot to adapter outside the lock (fire-and-forget)
            if let (Some(tx), Some(snap)) = (&moltbot_tx, snapshot) {
                // Use try_send to never block the epoch loop
                if let Err(e) = tx.try_send(snap) {
                    tracing::debug!("Moltbot channel full or closed: {}", e);
                }
            }

            std::thread::sleep(EPOCH_INTERVAL);
        }
    })
}

/// Build a JSON snapshot of the current world state for docs/system-state.json.
/// Called every AUTOSAVE_INTERVAL epochs from the background loop.
fn build_docs_snapshot(w: &World) -> Option<String> {
    let avg_fitness = if w.agents.is_empty() {
        0.0
    } else {
        w.agents.iter().map(|a| a.fitness()).sum::<f64>() / w.agents.len() as f64
    };

    let telemetry = w.telemetry();
    let risks: Vec<String> = telemetry.risks.iter().map(|r| format!("{:?}", r)).collect();

    let mut roles_map = std::collections::HashMap::<String, usize>::new();
    for agent in &w.agents {
        *roles_map.entry(agent.role.label().to_string()).or_insert(0) += 1;
    }

    let agents: Vec<serde_json::Value> = w.agents.iter().map(|a| {
        let survived = w.epoch.saturating_sub(
            w.agent_birth_epoch.get(&a.id).copied().unwrap_or(w.epoch)
        );
        let hex = a.genome_hex();
        let len = hex.len().min(16);
        serde_json::json!({
            "id": &hex[..len],
            "role": a.role.label(),
            "fitness": a.fitness(),
            "reputation": a.reputation.score,
            "atp": w.agent_atp(a),
            "generation": a.generation,
            "is_primordial": a.is_primordial,
            "survived_epochs": survived,
        })
    }).collect();

    let season = w.epoch_history.back()
        .map(|s| s.season.clone())
        .unwrap_or_else(|| "UNKNOWN".to_string());

    let history: Vec<serde_json::Value> = w.epoch_history.iter().rev().take(100).rev()
        .map(|s| serde_json::json!({
            "epoch": s.epoch,
            "population": s.population,
            "total_atp": s.total_atp,
            "mean_fitness": s.mean_fitness,
            "births": s.births,
            "deaths": s.deaths,
            "pop_cap": s.dynamic_pop_cap,
        }))
        .collect();

    let diff = w.epoch_diff(10);

    let doc = serde_json::json!({
        "epoch": w.epoch,
        "population": w.agents.len(),
        "pop_cap": w.pop_cap,
        "avg_fitness": avg_fitness,
        "total_atp": w.ledger.total_supply(),
        "treasury_reserve": w.treasury.reserve,
        "treasury_collected": w.treasury.total_collected,
        "treasury_distributed": w.treasury.total_distributed,
        "market_solved": w.problem_market.total_solved,
        "market_rewarded": w.problem_market.total_rewarded,
        "total_births": w.total_births,
        "total_deaths": w.total_deaths,
        "uptime_seconds": w.uptime_seconds(),
        "season": season,
        "roles": serde_json::json!(roles_map),
        "risks": risks,
        "agents": agents,
        "history": history,
        "epoch_diff": {
            "window": diff.window,
            "population_delta": diff.population_delta,
            "atp_delta": diff.atp_delta,
            "fitness_delta": diff.fitness_delta,
            "births_in_window": diff.births_in_window,
            "deaths_in_window": diff.deaths_in_window,
            "mutations_in_window": diff.mutations_in_window,
        },
    });

    serde_json::to_string(&doc).ok()
}

#[cfg(test)]
mod tests {
    use crate::world::World;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_background_loop_increments_epoch() {
        let world = Arc::new(Mutex::new(World::new()));
        let shared = world.clone();

        let handle = std::thread::spawn(move || {
            // Run a mini version — just 3 ticks, no sleeping
            for _ in 0..3 {
                let mut w = shared.lock().unwrap();
                w.run_epoch();
            }
        });

        handle.join().unwrap();

        let w = world.lock().unwrap();
        assert_eq!(w.epoch, 3);
        assert!(!w.agents.is_empty());
    }
}
