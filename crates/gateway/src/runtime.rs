// Runtime — Background Survival Loop
//
// Spawns a thread that ticks the world forward one epoch per interval.
// Autosaves every N epochs. The survival loop continues even if
// individual HTTP handler panics.

use std::time::Duration;

use crate::moltbot::EpochSnapshot;
use crate::persistence;
use crate::world::SharedWorld;

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
            let snapshot = {
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

                // Autosave
                if w.epoch % AUTOSAVE_INTERVAL == 0 {
                    if let Err(e) = persistence::save(&w) {
                        tracing::error!("Autosave failed: {}", e);
                    }
                }

                // Extinction guard — stop looping if everything is dead
                if w.agents.is_empty() {
                    tracing::error!("EXTINCTION EVENT at epoch {}", w.epoch);
                    return; // Exit thread
                }

                snapshot
            };
            // Lock released here

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
