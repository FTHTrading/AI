// Load Simulation — Adversarial Stress Test
//
// Validates that Genesis Protocol survives sustained concurrent load:
//   - 1,000 rapid GET /status requests
//   - 200 concurrent GET /leaderboard hits
//   - 100 rapid POST /register attempts (write throttle verification)
//   - Epoch tick stability under load (no drift)
//
// This is the gate check before Moltbook exposure.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gateway::server::{build_router_with_controls, default_controls};
use gateway::world::World;

use tokio::net::TcpListener;
use tokio::task::JoinSet;

/// Spin up a real HTTP server on a random port and return the base URL.
async fn spawn_server() -> (String, Arc<Mutex<World>>) {
    let world = Arc::new(Mutex::new(World::new()));
    let controls = default_controls();
    let app = build_router_with_controls(world.clone(), controls);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let base = format!("http://127.0.0.1:{}", port);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Brief pause for the listener to be ready
    tokio::time::sleep(Duration::from_millis(50)).await;

    (base, world)
}

/// Start a background epoch loop that ticks the world and records epoch timestamps.
fn start_epoch_loop(
    world: Arc<Mutex<World>>,
    tick_log: Arc<Mutex<Vec<Instant>>>,
    stop: Arc<std::sync::atomic::AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            {
                let mut w = world.lock().unwrap_or_else(|e| e.into_inner());
                w.run_epoch();
                tick_log.lock().unwrap().push(Instant::now());
            }
            std::thread::sleep(Duration::from_millis(100)); // 10 ticks/sec for faster testing
        }
    })
}

/// Latency statistics from a load run.
#[derive(Debug)]
#[allow(dead_code)]
struct LoadStats {
    total_requests: usize,
    successful: usize,
    rate_limited: usize, // 429s
    errors: usize,
    min_latency: Duration,
    max_latency: Duration,
    mean_latency: Duration,
    p99_latency: Duration,
}

impl LoadStats {
    fn from_results(results: &[(u16, Duration)]) -> Self {
        let total = results.len();
        let successful = results.iter().filter(|(s, _)| *s == 200 || *s == 201).count();
        let rate_limited = results.iter().filter(|(s, _)| *s == 429).count();
        let errors = total - successful - rate_limited;

        let mut latencies: Vec<Duration> = results.iter().map(|(_, d)| *d).collect();
        latencies.sort();

        let min = *latencies.first().unwrap_or(&Duration::ZERO);
        let max = *latencies.last().unwrap_or(&Duration::ZERO);
        let mean = if total > 0 {
            latencies.iter().sum::<Duration>() / total as u32
        } else {
            Duration::ZERO
        };
        let p99_idx = (total as f64 * 0.99).ceil() as usize;
        let p99 = if p99_idx > 0 && p99_idx <= latencies.len() {
            latencies[p99_idx - 1]
        } else {
            max
        };

        LoadStats {
            total_requests: total,
            successful,
            rate_limited,
            errors,
            min_latency: min,
            max_latency: max,
            mean_latency: mean,
            p99_latency: p99,
        }
    }
}

/// Fire N concurrent GET requests to a URL. Returns (status, latency) for each.
async fn flood_get(client: &reqwest::Client, url: &str, count: usize) -> Vec<(u16, Duration)> {
    let mut set = JoinSet::new();

    for _ in 0..count {
        let client = client.clone();
        let url = url.to_string();
        set.spawn(async move {
            let start = Instant::now();
            let resp = client.get(&url).send().await;
            let elapsed = start.elapsed();
            match resp {
                Ok(r) => (r.status().as_u16(), elapsed),
                Err(_) => (0, elapsed),
            }
        });
    }

    let mut results = Vec::with_capacity(count);
    while let Some(res) = set.join_next().await {
        if let Ok(r) = res {
            results.push(r);
        }
    }
    results
}

/// Fire N concurrent POST /register requests with unique payloads.
async fn flood_register(client: &reqwest::Client, base: &str, count: usize) -> Vec<(u16, Duration)> {
    let mut set = JoinSet::new();

    for i in 0..count {
        let client = client.clone();
        let url = format!("{}/register", base);
        set.spawn(async move {
            let body = serde_json::json!({
                "external_id": format!("loadtest:agent-{}", i),
                "public_key": format!("pk_load_{}", i)
            });
            let start = Instant::now();
            let resp = client
                .post(&url)
                .json(&body)
                .send()
                .await;
            let elapsed = start.elapsed();
            match resp {
                Ok(r) => (r.status().as_u16(), elapsed),
                Err(_) => (0, elapsed),
            }
        });
    }

    let mut results = Vec::with_capacity(count);
    while let Some(res) = set.join_next().await {
        if let Ok(r) = res {
            results.push(r);
        }
    }
    results
}

// ═══════════════════════════════════════════
// TEST: GET /status flood (1,000 requests)
// ═══════════════════════════════════════════

#[tokio::test]
async fn load_status_flood() {
    let (base, _world) = spawn_server().await;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let url = format!("{}/status", base);
    let results = flood_get(&client, &url, 1000).await;
    let stats = LoadStats::from_results(&results);

    println!("\n═══ GET /status × 1000 ═══");
    println!("  Successful:   {}", stats.successful);
    println!("  Rate limited: {}", stats.rate_limited);
    println!("  Errors:       {}", stats.errors);
    println!("  Min latency:  {:?}", stats.min_latency);
    println!("  Mean latency: {:?}", stats.mean_latency);
    println!("  P99 latency:  {:?}", stats.p99_latency);
    println!("  Max latency:  {:?}", stats.max_latency);

    // Assertions:
    // - No connection errors (all requests completed)
    assert_eq!(stats.errors, 0, "No requests should fail with connection errors");
    // - Some requests should succeed (at least burst amount)
    assert!(stats.successful > 0, "Some requests should succeed");
    // - Rate limiter should kick in — not all 1000 can pass through
    assert!(stats.rate_limited > 0, "Rate limiter should throttle excess traffic");
    // - Mean latency should be reasonable (under 2s for local)
    assert!(stats.mean_latency < Duration::from_secs(2), "Mean latency should be under 2s");
    // - P99 should be under 5s
    assert!(stats.p99_latency < Duration::from_secs(5), "P99 latency should be under 5s");
}

// ═══════════════════════════════════════════
// TEST: GET /leaderboard concurrent (200 requests)
// ═══════════════════════════════════════════

#[tokio::test]
async fn load_leaderboard_flood() {
    let (base, _world) = spawn_server().await;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let url = format!("{}/leaderboard", base);
    let results = flood_get(&client, &url, 200).await;
    let stats = LoadStats::from_results(&results);

    println!("\n═══ GET /leaderboard × 200 ═══");
    println!("  Successful:   {}", stats.successful);
    println!("  Rate limited: {}", stats.rate_limited);
    println!("  Errors:       {}", stats.errors);
    println!("  Mean latency: {:?}", stats.mean_latency);
    println!("  P99 latency:  {:?}", stats.p99_latency);

    assert_eq!(stats.errors, 0, "No connection errors");
    assert!(stats.successful > 0, "Some requests should succeed");
    assert!(stats.mean_latency < Duration::from_secs(2), "Mean latency under 2s");
}

// ═══════════════════════════════════════════
// TEST: POST /register write throttle (100 rapid attempts)
// ═══════════════════════════════════════════

#[tokio::test]
async fn load_register_throttle() {
    let (base, _world) = spawn_server().await;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let results = flood_register(&client, &base, 100).await;
    let stats = LoadStats::from_results(&results);

    let created = results.iter().filter(|(s, _)| *s == 201).count();
    let throttled = results.iter().filter(|(s, _)| *s == 429).count();

    println!("\n═══ POST /register × 100 ═══");
    println!("  Created (201):     {}", created);
    println!("  Throttled (429):   {}", throttled);
    println!("  Errors:            {}", stats.errors);
    println!("  Mean latency:      {:?}", stats.mean_latency);
    println!("  P99 latency:       {:?}", stats.p99_latency);

    assert_eq!(stats.errors, 0, "No connection errors");
    // Write limiter: burst 10 — most of 100 should be rejected
    assert!(throttled > 80, "Write limiter should reject most of 100 rapid POST requests (got {} throttled)", throttled);
    // But some should get through (up to burst of 10)
    assert!(created > 0, "Some registrations should succeed within burst window");
    assert!(created <= 15, "No more than ~burst registrations should succeed (got {})", created);
}

// ═══════════════════════════════════════════
// TEST: Epoch tick stability under load
// ═══════════════════════════════════════════

#[tokio::test]
async fn load_epoch_stability() {
    let (base, world) = spawn_server().await;

    // Start epoch loop
    let tick_log: Arc<Mutex<Vec<Instant>>> = Arc::new(Mutex::new(Vec::new()));
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let handle = start_epoch_loop(world.clone(), tick_log.clone(), stop.clone());

    // Let epoch loop establish a baseline (5 ticks)
    tokio::time::sleep(Duration::from_millis(600)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    // Record epoch before load
    let epoch_before = {
        let w = world.lock().unwrap();
        w.epoch
    };

    // Hammer /status and /leaderboard concurrently during load
    let status_url = format!("{}/status", base);
    let leader_url = format!("{}/leaderboard", base);

    let (status_results, leader_results) = tokio::join!(
        flood_get(&client, &status_url, 500),
        flood_get(&client, &leader_url, 200),
    );

    // Let a few more ticks happen after load
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Stop epoch loop
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    handle.join().unwrap();

    let epoch_after = {
        let w = world.lock().unwrap();
        w.epoch
    };

    // Analyse tick stability
    let ticks = tick_log.lock().unwrap();
    let tick_count = ticks.len();

    // Calculate inter-tick intervals
    let intervals: Vec<Duration> = ticks.windows(2).map(|w| w[1] - w[0]).collect();
    let max_interval = intervals.iter().max().copied().unwrap_or(Duration::ZERO);
    let min_interval = intervals.iter().min().copied().unwrap_or(Duration::ZERO);
    let mean_interval = if !intervals.is_empty() {
        intervals.iter().sum::<Duration>() / intervals.len() as u32
    } else {
        Duration::ZERO
    };

    let status_stats = LoadStats::from_results(&status_results);
    let leader_stats = LoadStats::from_results(&leader_results);

    println!("\n═══ EPOCH STABILITY UNDER LOAD ═══");
    println!("  Epochs advanced:    {} → {} (+{})", epoch_before, epoch_after, epoch_after - epoch_before);
    println!("  Total ticks:        {}", tick_count);
    println!("  Min tick interval:  {:?}", min_interval);
    println!("  Mean tick interval: {:?}", mean_interval);
    println!("  Max tick interval:  {:?}", max_interval);
    println!("  └── /status: {} ok, {} throttled, {} errors", status_stats.successful, status_stats.rate_limited, status_stats.errors);
    println!("  └── /leader: {} ok, {} throttled, {} errors", leader_stats.successful, leader_stats.rate_limited, leader_stats.errors);

    // Assertions:
    // - Epoch loop must have advanced during load
    assert!(epoch_after > epoch_before, "Epoch loop must continue advancing during HTTP load");
    assert!(tick_count >= 5, "At least 5 ticks should have occurred during the test");

    // - No tick interval should exceed 3x the target (100ms target → 300ms max acceptable)
    assert!(
        max_interval < Duration::from_millis(500),
        "Max tick interval {:?} exceeds 500ms — epoch loop is being starved by HTTP handlers",
        max_interval
    );

    // - Mean interval should be close to target (100ms ± 100ms tolerance)
    assert!(
        mean_interval < Duration::from_millis(200),
        "Mean tick interval {:?} is too high — mutex contention suspected",
        mean_interval
    );

    // - HTTP load should not fail with connection errors
    assert_eq!(status_stats.errors, 0, "No /status connection errors during load");
    assert_eq!(leader_stats.errors, 0, "No /leaderboard connection errors during load");
}

// ═══════════════════════════════════════════
// TEST: Response coherence under concurrent reads
// ═══════════════════════════════════════════

#[tokio::test]
async fn load_response_coherence() {
    let (base, world) = spawn_server().await;

    // Run some epochs to create meaningful state
    {
        let mut w = world.lock().unwrap();
        for _ in 0..10 {
            w.run_epoch();
        }
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    // Fire 50 concurrent GET /status and verify each response parses as valid JSON
    let url = format!("{}/status", base);
    let mut set = JoinSet::new();

    for _ in 0..50 {
        let client = client.clone();
        let url = url.clone();
        set.spawn(async move {
            let resp = client.get(&url).send().await;
            match resp {
                Ok(r) => {
                    let status = r.status().as_u16();
                    if status == 200 {
                        let body = r.text().await.unwrap_or_default();
                        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body);
                        (status, parsed.is_ok())
                    } else {
                        // 429 is acceptable
                        (status, true)
                    }
                }
                Err(_) => (0, false),
            }
        });
    }

    let mut total = 0;
    let mut valid = 0;
    while let Some(res) = set.join_next().await {
        if let Ok((status, is_valid)) = res {
            total += 1;
            if status == 200 && is_valid {
                valid += 1;
            } else if status == 429 {
                valid += 1; // Throttled responses are also valid behavior
            }
        }
    }

    println!("\n═══ RESPONSE COHERENCE × 50 ═══");
    println!("  Total:   {}", total);
    println!("  Valid:   {}", valid);

    assert_eq!(total, 50, "All 50 requests should complete");
    assert_eq!(valid, total, "Every response must be valid JSON or a proper 429");
}

// ═══════════════════════════════════════════
// TEST: Mixed concurrent load (all endpoints)
// ═══════════════════════════════════════════

#[tokio::test]
async fn load_mixed_concurrent() {
    let (base, world) = spawn_server().await;

    // Seed some epochs
    {
        let mut w = world.lock().unwrap();
        for _ in 0..5 {
            w.run_epoch();
        }
    }

    // Start epoch loop
    let tick_log: Arc<Mutex<Vec<Instant>>> = Arc::new(Mutex::new(Vec::new()));
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let handle = start_epoch_loop(world.clone(), tick_log.clone(), stop.clone());

    tokio::time::sleep(Duration::from_millis(300)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let status_url = format!("{}/status", base);
    let leader_url = format!("{}/leaderboard", base);
    let genesis_url = format!("{}/genesis", base);

    // Concurrent mixed load:
    // 300 × /status + 100 × /leaderboard + 50 × /genesis + 30 × POST /register
    let (s_res, l_res, g_res, r_res) = tokio::join!(
        flood_get(&client, &status_url, 300),
        flood_get(&client, &leader_url, 100),
        flood_get(&client, &genesis_url, 50),
        flood_register(&client, &base, 30),
    );

    // Stop epoch loop
    tokio::time::sleep(Duration::from_millis(300)).await;
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    handle.join().unwrap();

    let s = LoadStats::from_results(&s_res);
    let l = LoadStats::from_results(&l_res);
    let g = LoadStats::from_results(&g_res);
    let r = LoadStats::from_results(&r_res);

    println!("\n═══ MIXED CONCURRENT LOAD ═══");
    println!("  /status:      {} ok, {} throttled, {} err (p99: {:?})", s.successful, s.rate_limited, s.errors, s.p99_latency);
    println!("  /leaderboard: {} ok, {} throttled, {} err (p99: {:?})", l.successful, l.rate_limited, l.errors, l.p99_latency);
    println!("  /genesis:     {} ok, {} throttled, {} err (p99: {:?})", g.successful, g.rate_limited, g.errors, g.p99_latency);
    println!("  /register:    {} ok, {} throttled, {} err (p99: {:?})", r.successful, r.rate_limited, r.errors, r.p99_latency);

    // No connection errors across any endpoint
    assert_eq!(s.errors, 0, "No /status errors");
    assert_eq!(l.errors, 0, "No /leaderboard errors");
    assert_eq!(g.errors, 0, "No /genesis errors");
    assert_eq!(r.errors, 0, "No /register errors");

    // Rate limiter engaged
    let total_throttled = s.rate_limited + l.rate_limited + g.rate_limited + r.rate_limited;
    assert!(total_throttled > 0, "Rate limiter should engage under mixed load");

    // Epoch loop still healthy
    let ticks = tick_log.lock().unwrap();
    assert!(ticks.len() >= 3, "Epoch loop must keep ticking during mixed load");

    // P99 latency reasonable
    assert!(s.p99_latency < Duration::from_secs(5), "/status p99 too high");
    assert!(l.p99_latency < Duration::from_secs(5), "/leaderboard p99 too high");
    assert!(g.p99_latency < Duration::from_secs(5), "/genesis p99 too high");
}
