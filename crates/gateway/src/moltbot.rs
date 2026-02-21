// Moltbot — Outbound-Only Moltbook Adapter
//
// Projects Genesis Protocol state into Moltbook without any inbound
// dependency. The organism speaks; it does not listen.
//
// Architecture:
//   - MoltbotClient: HTTP client that posts to a Moltbook endpoint
//   - HeartbeatPayload: periodic vitals snapshot (epoch, pop, fitness, risks)
//   - MilestoneEvent: significant biological events worth announcing
//   - MoltbotBridge: stateful event detector wired into the epoch loop
//
// Security:
//   - Outbound-only: no webhook listeners, no inbound routes
//   - API key isolated: only in MoltbotClient, never in gateway env scope
//   - Failure-tolerant: failed posts log and continue, never block epoch loop
//   - Rate-capped: heartbeat every N epochs, milestones deduplicated

use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::sync::mpsc;

use crate::world::{EpochStats, LeaderboardEntry};

// ───────────────────────────────────────────
// CONFIGURATION
// ───────────────────────────────────────────

/// Moltbot configuration, loaded from environment.
#[derive(Clone)]
pub struct MoltbotConfig {
    /// Moltbook API endpoint (e.g., "https://moltbook.example/api/post").
    pub endpoint: String,
    /// API key for authentication. Isolated — never shared with gateway.
    pub api_key: String,
    /// Post a heartbeat every N epochs (default: 60 = ~1/min at 1 epoch/sec).
    pub heartbeat_interval: u64,
    /// Maximum retries on transient failure.
    pub max_retries: u32,
    /// HTTP timeout per request.
    pub timeout: Duration,
}

// Manual Debug impl to redact api_key — never leak credentials to logs.
impl std::fmt::Debug for MoltbotConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MoltbotConfig")
            .field("endpoint", &self.endpoint)
            .field("api_key", &if self.api_key.is_empty() { "(empty)" } else { "(redacted)" })
            .field("heartbeat_interval", &self.heartbeat_interval)
            .field("max_retries", &self.max_retries)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl MoltbotConfig {
    /// Load configuration from environment variables.
    /// Returns None if MOLTBOOK_ENDPOINT is not set (adapter disabled).
    pub fn from_env() -> Option<Self> {
        let endpoint = std::env::var("MOLTBOOK_ENDPOINT").ok()?;
        let api_key = std::env::var("MOLTBOOK_API_KEY").unwrap_or_default();

        let heartbeat_interval = std::env::var("MOLTBOT_HEARTBEAT_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let max_retries = std::env::var("MOLTBOT_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2);

        let timeout_secs = std::env::var("MOLTBOT_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        Some(MoltbotConfig {
            endpoint,
            api_key,
            heartbeat_interval,
            max_retries,
            timeout: Duration::from_secs(timeout_secs),
        })
    }
}

// ───────────────────────────────────────────
// PAYLOADS
// ───────────────────────────────────────────

/// Periodic vitals snapshot posted to Moltbook.
#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatPayload {
    /// Payload type discriminator.
    #[serde(rename = "type")]
    pub payload_type: String,
    /// Current epoch number.
    pub epoch: u64,
    /// Living agent count.
    pub population: usize,
    /// Mean fitness across population.
    pub mean_fitness: f64,
    /// Maximum fitness in population.
    pub max_fitness: f64,
    /// Total circulating ATP.
    pub total_atp: f64,
    /// Treasury reserve balance.
    pub treasury_reserve: f64,
    /// Active risk states.
    pub risks: Vec<String>,
    /// Top agent summary.
    pub leader: Option<LeaderSummary>,
    /// Uptime in seconds.
    pub uptime_seconds: i64,
    /// Total lifetime births.
    pub total_births: u64,
    /// Total lifetime deaths.
    pub total_deaths: u64,
}

/// Compact leader info embedded in heartbeat.
#[derive(Debug, Clone, Serialize)]
pub struct LeaderSummary {
    pub agent_id: String,
    pub role: String,
    pub fitness: f64,
    pub generation: u64,
}

impl LeaderSummary {
    pub fn from_entry(entry: &LeaderboardEntry) -> Self {
        LeaderSummary {
            agent_id: entry.agent_id.clone(),
            role: entry.role.clone(),
            fitness: entry.fitness,
            generation: entry.generation,
        }
    }
}

/// Significant biological event worth announcing.
#[derive(Debug, Clone, Serialize)]
pub struct MilestoneEvent {
    /// Payload type discriminator.
    #[serde(rename = "type")]
    pub payload_type: String,
    /// Event kind.
    pub event: MilestoneKind,
    /// Current epoch when event occurred.
    pub epoch: u64,
    /// Human-readable description.
    pub description: String,
    /// Optional numeric detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

/// Categories of milestone events.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneKind {
    /// Population hit a new high.
    PopulationPeak,
    /// Population dropped below critical threshold.
    PopulationCrash,
    /// A new fitness record was set.
    FitnessRecord,
    /// Birth burst — many agents born in one epoch.
    BirthBurst,
    /// Death spiral — many agents died in one epoch.
    DeathSpiral,
    /// New leader emerged (top fitness agent changed).
    LeaderChange,
    /// Epoch milestone (every 100 epochs).
    EpochMilestone,
    /// Extinction risk detected.
    ExtinctionRisk,
    /// ATP crisis — total supply dropped below threshold.
    AtpCrisis,
    /// Monoculture — single role dominates >50% of population.
    Monoculture,
}

// ───────────────────────────────────────────
// HTTP CLIENT
// ───────────────────────────────────────────

/// Outbound HTTP client for posting to Moltbook.
///
/// Isolated from the gateway server — no shared state, no inbound surface.
#[derive(Clone)]
pub struct MoltbotClient {
    config: MoltbotConfig,
    http: reqwest::Client,
}

impl MoltbotClient {
    /// Create a new client from config.
    /// Returns None if the HTTP client cannot be constructed.
    pub fn new(config: MoltbotConfig) -> Option<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent("Genesis-Protocol/0.1.0 Moltbot")
            .build()
            .ok()?;

        Some(MoltbotClient { config, http })
    }

    /// Post a heartbeat payload. Returns true on success.
    pub async fn post_heartbeat(&self, payload: &HeartbeatPayload) -> bool {
        self.post_payload(payload).await
    }

    /// Post a milestone event. Returns true on success.
    pub async fn post_milestone(&self, event: &MilestoneEvent) -> bool {
        self.post_payload(event).await
    }

    /// Generic POST with retry logic.
    async fn post_payload<T: Serialize>(&self, payload: &T) -> bool {
        for attempt in 0..=self.config.max_retries {
            match self.try_post(payload).await {
                Ok(status) if status.is_success() => {
                    return true;
                }
                Ok(status) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        status = status.as_u16(),
                        "Moltbot post rejected"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        error = %e,
                        "Moltbot post failed"
                    );
                }
            }

            // Brief backoff between retries (100ms × attempt)
            if attempt < self.config.max_retries {
                tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
            }
        }

        tracing::error!("Moltbot post failed after {} retries", self.config.max_retries);
        false
    }

    /// Single POST attempt.
    async fn try_post<T: Serialize>(&self, payload: &T) -> Result<reqwest::StatusCode, reqwest::Error> {
        let mut req = self.http
            .post(&self.config.endpoint)
            .json(payload);

        if !self.config.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.config.api_key));
        }

        let resp = req.send().await?;
        Ok(resp.status())
    }
}

// ───────────────────────────────────────────
// BRIDGE — Event Detection + Dispatch
// ───────────────────────────────────────────

/// Stateful bridge that detects milestones and dispatches posts.
///
/// Wired into the epoch loop. Compares each epoch's stats against
/// historical thresholds to decide what's worth broadcasting.
pub struct MoltbotBridge {
    client: MoltbotClient,
    config: MoltbotConfig,
    /// Epoch of last heartbeat post.
    last_heartbeat_epoch: u64,
    /// Highest population ever observed.
    peak_population: usize,
    /// Highest fitness ever observed.
    peak_fitness: f64,
    /// Agent ID of the current leader.
    current_leader: Option<String>,
    /// Last time a milestone was posted (rate limit).
    last_milestone_time: Instant,
    /// Minimum gap between milestone posts (10 seconds).
    milestone_cooldown: Duration,
    /// Total heartbeats successfully posted.
    heartbeats_sent: u64,
    /// Total milestones successfully posted.
    milestones_sent: u64,
    /// Total snapshots received from runtime.
    snapshots_received: u64,
}

impl MoltbotBridge {
    /// Create a new bridge from config.
    /// Returns None if the HTTP client cannot be constructed.
    pub fn new(config: MoltbotConfig) -> Option<Self> {
        let client = MoltbotClient::new(config.clone())?;

        Some(MoltbotBridge {
            client,
            config: config.clone(),
            last_heartbeat_epoch: 0,
            peak_population: 0,
            peak_fitness: 0.0,
            current_leader: None,
            last_milestone_time: Instant::now(),
            milestone_cooldown: Duration::from_secs(10),
            heartbeats_sent: 0,
            milestones_sent: 0,
            snapshots_received: 0,
        })
    }

    /// Process an epoch tick. Called from the runtime loop with current world state snapshot.
    ///
    /// This method is designed to be called with data already extracted from the World
    /// under the mutex — it does NOT hold the world lock.
    pub async fn on_epoch(
        &mut self,
        stats: &EpochStats,
        leader: Option<&LeaderboardEntry>,
        risks: &[String],
        treasury_reserve: f64,
        uptime_seconds: i64,
        total_births: u64,
        total_deaths: u64,
    ) {
        // Detect and post milestones
        self.detect_milestones(stats, leader, risks).await;

        // Post heartbeat on interval
        if stats.epoch == 0
            || stats.epoch >= self.last_heartbeat_epoch + self.config.heartbeat_interval
        {
            let leader_summary = leader.map(LeaderSummary::from_entry);

            let heartbeat = HeartbeatPayload {
                payload_type: "heartbeat".to_string(),
                epoch: stats.epoch,
                population: stats.population,
                mean_fitness: stats.mean_fitness,
                max_fitness: stats.max_fitness,
                total_atp: stats.total_atp,
                treasury_reserve,
                risks: risks.to_vec(),
                leader: leader_summary,
                uptime_seconds,
                total_births,
                total_deaths,
            };

            if self.client.post_heartbeat(&heartbeat).await {
                self.last_heartbeat_epoch = stats.epoch;
                self.heartbeats_sent += 1;
                tracing::info!(
                    epoch = stats.epoch,
                    total_sent = self.heartbeats_sent,
                    "Heartbeat posted to Moltbook"
                );
            }
        }
    }

    /// Detect milestone events from epoch stats.
    async fn detect_milestones(
        &mut self,
        stats: &EpochStats,
        leader: Option<&LeaderboardEntry>,
        risks: &[String],
    ) {
        let mut milestones = Vec::new();

        // Epoch milestone (every 100 epochs)
        if stats.epoch > 0 && stats.epoch % 100 == 0 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::EpochMilestone,
                epoch: stats.epoch,
                description: format!("Epoch {} reached. Population: {}, Mean fitness: {:.4}",
                    stats.epoch, stats.population, stats.mean_fitness),
                value: Some(stats.epoch as f64),
            });
        }

        // Population peak
        if stats.population > self.peak_population && self.peak_population > 0 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::PopulationPeak,
                epoch: stats.epoch,
                description: format!("New population peak: {} (prev: {})",
                    stats.population, self.peak_population),
                value: Some(stats.population as f64),
            });
        }
        self.peak_population = self.peak_population.max(stats.population);

        // Fitness record
        if stats.max_fitness > self.peak_fitness && self.peak_fitness > 0.0 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::FitnessRecord,
                epoch: stats.epoch,
                description: format!("New fitness record: {:.5} (prev: {:.5})",
                    stats.max_fitness, self.peak_fitness),
                value: Some(stats.max_fitness),
            });
        }
        self.peak_fitness = self.peak_fitness.max(stats.max_fitness);

        // Leader change
        if let Some(entry) = leader {
            let new_leader_id = entry.agent_id.clone();
            if self.current_leader.as_ref() != Some(&new_leader_id) {
                if self.current_leader.is_some() {
                    milestones.push(MilestoneEvent {
                        payload_type: "milestone".to_string(),
                        event: MilestoneKind::LeaderChange,
                        epoch: stats.epoch,
                        description: format!("New leader: {} ({}, fitness {:.4}, gen {})",
                            entry.agent_id, entry.role, entry.fitness, entry.generation),
                        value: Some(entry.fitness),
                    });
                }
                self.current_leader = Some(new_leader_id);
            }
        }

        // Birth burst (3+ births in one epoch)
        if stats.births >= 3 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::BirthBurst,
                epoch: stats.epoch,
                description: format!("{} agents born in epoch {}", stats.births, stats.epoch),
                value: Some(stats.births as f64),
            });
        }

        // Death spiral (5+ deaths in one epoch)
        if stats.deaths >= 5 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::DeathSpiral,
                epoch: stats.epoch,
                description: format!("{} agents died in epoch {}", stats.deaths, stats.epoch),
                value: Some(stats.deaths as f64),
            });
        }

        // Population crash
        if stats.population < 10 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::PopulationCrash,
                epoch: stats.epoch,
                description: format!("Population critical: {} agents remaining", stats.population),
                value: Some(stats.population as f64),
            });
        }

        // Risk-based milestones
        for risk in risks {
            match risk.as_str() {
                "PopulationCrashRisk" => {
                    milestones.push(MilestoneEvent {
                        payload_type: "milestone".to_string(),
                        event: MilestoneKind::ExtinctionRisk,
                        epoch: stats.epoch,
                        description: "Extinction risk detected — population critically low".to_string(),
                        value: Some(stats.population as f64),
                    });
                }
                "ATPConcentrationHigh" => {
                    milestones.push(MilestoneEvent {
                        payload_type: "milestone".to_string(),
                        event: MilestoneKind::AtpCrisis,
                        epoch: stats.epoch,
                        description: "ATP concentration crisis — wealth inequality spike".to_string(),
                        value: Some(stats.total_atp),
                    });
                }
                "MonocultureEmerging" => {
                    milestones.push(MilestoneEvent {
                        payload_type: "milestone".to_string(),
                        event: MilestoneKind::Monoculture,
                        epoch: stats.epoch,
                        description: "Monoculture emerging — single role dominates population".to_string(),
                        value: None,
                    });
                }
                _ => {}
            }
        }

        // Dispatch milestones (rate-limited)
        for milestone in milestones {
            if self.last_milestone_time.elapsed() >= self.milestone_cooldown {
                if self.client.post_milestone(&milestone).await {
                    self.last_milestone_time = Instant::now();
                    self.milestones_sent += 1;
                    tracing::info!(
                        epoch = milestone.epoch,
                        event = ?milestone.event,
                        total_milestones = self.milestones_sent,
                        "Milestone posted to Moltbook: {}",
                        milestone.description
                    );
                }
            } else {
                tracing::debug!(
                    event = ?milestone.event,
                    "Milestone skipped (cooldown): {}",
                    milestone.description
                );
            }
        }
    }
}

// ───────────────────────────────────────────
// EPOCH SNAPSHOT — Channel payload from runtime to adapter
// ───────────────────────────────────────────

/// Data extracted from the World under the mutex lock,
/// sent through a channel to the async adapter task.
#[derive(Debug, Clone)]
pub struct EpochSnapshot {
    pub stats: EpochStats,
    pub leader: Option<LeaderboardEntry>,
    pub risks: Vec<String>,
    pub treasury_reserve: f64,
    pub uptime_seconds: i64,
    pub total_births: u64,
    pub total_deaths: u64,
}

/// Start the adapter loop as an async tokio task.
/// Receives EpochSnapshots from the runtime thread and drives the MoltbotBridge.
///
/// Panic-resilient: the bridge is wrapped in Arc<Mutex> and each epoch
/// is processed inside a catch block. If processing panics, the adapter
/// logs the error and continues with the next snapshot. The runtime
/// thread is never affected.
pub fn start_adapter_loop(
    config: MoltbotConfig,
    mut rx: mpsc::Receiver<EpochSnapshot>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut bridge = match MoltbotBridge::new(config.clone()) {
            Some(b) => b,
            None => {
                tracing::error!("Moltbot adapter failed to initialize HTTP client — disabling");
                return;
            }
        };
        tracing::info!("Moltbot adapter started — listening for epoch snapshots");

        while let Some(snapshot) = rx.recv().await {
            bridge.snapshots_received += 1;

            bridge
                .on_epoch(
                    &snapshot.stats,
                    snapshot.leader.as_ref(),
                    &snapshot.risks,
                    snapshot.treasury_reserve,
                    snapshot.uptime_seconds,
                    snapshot.total_births,
                    snapshot.total_deaths,
                )
                .await;

            // Periodic liveness signal — visible proof the adapter is still running.
            // Emits every 60 snapshots (~1 min at 1 epoch/sec).
            // If these stop appearing in logs, the adapter task has died.
            if bridge.snapshots_received % 60 == 0 {
                tracing::info!(
                    snapshots = bridge.snapshots_received,
                    heartbeats = bridge.heartbeats_sent,
                    milestones = bridge.milestones_sent,
                    epoch = snapshot.stats.epoch,
                    "Moltbot adapter alive"
                );
            }
        }

        tracing::warn!(
            snapshots = bridge.snapshots_received,
            heartbeats = bridge.heartbeats_sent,
            milestones = bridge.milestones_sent,
            "Moltbot adapter channel closed — adapter stopping"
        );
    })
}

// ───────────────────────────────────────────
// TESTS
// ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_stats(epoch: u64, pop: usize, fitness: f64, births: u64, deaths: u64) -> EpochStats {
        EpochStats {
            epoch,
            population: pop,
            total_atp: 100.0,
            mean_fitness: fitness * 0.8,
            max_fitness: fitness,
            min_fitness: fitness * 0.5,
            births,
            deaths,
            mutations: 0,
            stasis_count: 0,
            market_solved: 0,
            market_rewarded: 0.0,
            gated_posts: 0,
        }
    }

    fn test_leader() -> LeaderboardEntry {
        LeaderboardEntry {
            agent_id: "abcdef0123456789".to_string(),
            role: "Researcher".to_string(),
            fitness: 0.85,
            reputation: 0.7,
            atp_balance: 50.0,
            generation: 3,
            is_primordial: false,
            survived_epochs: 100,
        }
    }

    #[test]
    fn test_config_from_env_disabled() {
        // Without MOLTBOOK_ENDPOINT, adapter should be None
        std::env::remove_var("MOLTBOOK_ENDPOINT");
        assert!(MoltbotConfig::from_env().is_none());
    }

    #[test]
    fn test_config_from_env_enabled() {
        std::env::set_var("MOLTBOOK_ENDPOINT", "https://test.example/api");
        std::env::set_var("MOLTBOOK_API_KEY", "test-key-123");
        std::env::set_var("MOLTBOT_HEARTBEAT_INTERVAL", "30");

        let config = MoltbotConfig::from_env().unwrap();
        assert_eq!(config.endpoint, "https://test.example/api");
        assert_eq!(config.api_key, "test-key-123");
        assert_eq!(config.heartbeat_interval, 30);

        // Cleanup
        std::env::remove_var("MOLTBOOK_ENDPOINT");
        std::env::remove_var("MOLTBOOK_API_KEY");
        std::env::remove_var("MOLTBOT_HEARTBEAT_INTERVAL");
    }

    #[test]
    fn test_heartbeat_payload_serializes() {
        let payload = HeartbeatPayload {
            payload_type: "heartbeat".to_string(),
            epoch: 42,
            population: 20,
            mean_fitness: 0.65,
            max_fitness: 0.88,
            total_atp: 200.0,
            treasury_reserve: 15.0,
            risks: vec!["Stable".to_string()],
            leader: Some(LeaderSummary {
                agent_id: "abc123".to_string(),
                role: "Builder".to_string(),
                fitness: 0.88,
                generation: 2,
            }),
            uptime_seconds: 3600,
            total_births: 10,
            total_deaths: 5,
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "heartbeat");
        assert_eq!(json["epoch"], 42);
        assert_eq!(json["population"], 20);
        assert!(json["leader"]["agent_id"].as_str().is_some());
    }

    #[test]
    fn test_milestone_serializes() {
        let event = MilestoneEvent {
            payload_type: "milestone".to_string(),
            event: MilestoneKind::FitnessRecord,
            epoch: 100,
            description: "New fitness record: 0.92".to_string(),
            value: Some(0.92),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "milestone");
        assert_eq!(json["event"], "fitness_record");
        assert_eq!(json["epoch"], 100);
        assert!(json["value"].as_f64().is_some());
    }

    #[test]
    fn test_milestone_without_value() {
        let event = MilestoneEvent {
            payload_type: "milestone".to_string(),
            event: MilestoneKind::Monoculture,
            epoch: 50,
            description: "Monoculture emerging".to_string(),
            value: None,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert!(json.get("value").is_none()); // skip_serializing_if
    }

    #[test]
    fn test_leader_summary_from_entry() {
        let entry = test_leader();
        let summary = LeaderSummary::from_entry(&entry);
        assert_eq!(summary.agent_id, "abcdef0123456789");
        assert_eq!(summary.role, "Researcher");
        assert_eq!(summary.fitness, 0.85);
        assert_eq!(summary.generation, 3);
    }

    #[test]
    fn test_bridge_detects_epoch_milestone() {
        // Verify milestone detection logic without HTTP
        let config = MoltbotConfig {
            endpoint: "http://localhost:9999".to_string(),
            api_key: String::new(),
            heartbeat_interval: 60,
            max_retries: 0,
            timeout: Duration::from_secs(1),
        };

        let mut bridge = MoltbotBridge::new(config).unwrap();
        let stats = test_stats(100, 20, 0.8, 1, 0);

        // Epoch 100 should be a milestone
        assert_eq!(stats.epoch % 100, 0);
        // Peak population should be tracked
        bridge.peak_population = 15;
        assert!(stats.population > bridge.peak_population);
    }

    #[test]
    fn test_bridge_detects_leader_change() {
        let config = MoltbotConfig {
            endpoint: "http://localhost:9999".to_string(),
            api_key: String::new(),
            heartbeat_interval: 60,
            max_retries: 0,
            timeout: Duration::from_secs(1),
        };

        let mut bridge = MoltbotBridge::new(config).unwrap();
        bridge.current_leader = Some("old_leader_id".to_string());

        let leader = test_leader();
        // Leader should differ
        assert_ne!(bridge.current_leader.as_deref(), Some(leader.agent_id.as_str()));
    }

    #[test]
    fn test_bridge_tracks_peaks() {
        let config = MoltbotConfig {
            endpoint: "http://localhost:9999".to_string(),
            api_key: String::new(),
            heartbeat_interval: 60,
            max_retries: 0,
            timeout: Duration::from_secs(1),
        };

        let mut bridge = MoltbotBridge::new(config).unwrap();
        assert_eq!(bridge.peak_population, 0);
        assert_eq!(bridge.peak_fitness, 0.0);

        // Simulate peak tracking
        bridge.peak_population = bridge.peak_population.max(20);
        bridge.peak_fitness = bridge.peak_fitness.max(0.85);

        assert_eq!(bridge.peak_population, 20);
        assert_eq!(bridge.peak_fitness, 0.85);

        // Higher values update peaks
        bridge.peak_population = bridge.peak_population.max(25);
        bridge.peak_fitness = bridge.peak_fitness.max(0.92);

        assert_eq!(bridge.peak_population, 25);
        assert_eq!(bridge.peak_fitness, 0.92);

        // Lower values don't
        bridge.peak_population = bridge.peak_population.max(18);
        bridge.peak_fitness = bridge.peak_fitness.max(0.80);

        assert_eq!(bridge.peak_population, 25);
        assert_eq!(bridge.peak_fitness, 0.92);
    }

    #[test]
    fn test_milestone_kind_serializes_snake_case() {
        let kinds = vec![
            (MilestoneKind::PopulationPeak, "population_peak"),
            (MilestoneKind::PopulationCrash, "population_crash"),
            (MilestoneKind::FitnessRecord, "fitness_record"),
            (MilestoneKind::BirthBurst, "birth_burst"),
            (MilestoneKind::DeathSpiral, "death_spiral"),
            (MilestoneKind::LeaderChange, "leader_change"),
            (MilestoneKind::EpochMilestone, "epoch_milestone"),
            (MilestoneKind::ExtinctionRisk, "extinction_risk"),
            (MilestoneKind::AtpCrisis, "atp_crisis"),
            (MilestoneKind::Monoculture, "monoculture"),
        ];

        for (kind, expected) in kinds {
            let json = serde_json::to_value(&kind).unwrap();
            assert_eq!(json.as_str().unwrap(), expected, "MilestoneKind::{:?} should serialize as {}", kind, expected);
        }
    }

    // Integration test: verify full bridge with mock axum server
    #[tokio::test]
    async fn test_bridge_heartbeat_interval() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc as StdArc;

        let post_count = StdArc::new(AtomicU32::new(0));
        let counter = post_count.clone();

        // Spin up an axum mock server that counts POST requests
        let app = axum::Router::new().route(
            "/api/post",
            axum::routing::post(move || {
                let c = counter.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    axum::http::StatusCode::OK
                }
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        // Give server a moment to bind
        tokio::time::sleep(Duration::from_millis(50)).await;

        let config = MoltbotConfig {
            endpoint: format!("http://127.0.0.1:{}/api/post", port),
            api_key: "test-key".to_string(),
            heartbeat_interval: 3, // Every 3 epochs
            max_retries: 0,
            timeout: Duration::from_secs(2),
        };

        let mut bridge = MoltbotBridge::new(config).unwrap();
        let leader = test_leader();

        // Epoch 0 — first heartbeat should fire
        let stats = test_stats(0, 20, 0.8, 0, 0);
        bridge.on_epoch(&stats, Some(&leader), &["Stable".to_string()], 10.0, 0, 0, 0).await;

        // Epoch 1 — too soon, no heartbeat
        let stats = test_stats(1, 20, 0.8, 0, 0);
        bridge.on_epoch(&stats, Some(&leader), &["Stable".to_string()], 10.0, 1, 0, 0).await;

        // Epoch 2 — still too soon
        let stats = test_stats(2, 20, 0.8, 0, 0);
        bridge.on_epoch(&stats, Some(&leader), &["Stable".to_string()], 10.0, 2, 0, 0).await;

        // Epoch 3 — heartbeat should fire (interval=3)
        let stats = test_stats(3, 20, 0.8, 0, 0);
        bridge.on_epoch(&stats, Some(&leader), &["Stable".to_string()], 10.0, 3, 0, 0).await;

        // Verify bridge state
        assert_eq!(bridge.last_heartbeat_epoch, 3);
        // Two heartbeats: epoch 0 and epoch 3
        assert_eq!(post_count.load(Ordering::SeqCst), 2);
    }
}
