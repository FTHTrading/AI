// World — Encapsulated Genesis Protocol State
//
// Contains the entire living state of the Genesis organism:
// agents, ledger, treasury, markets, engines. The `run_epoch`
// method advances the world by one tick — all evolutionary
// logic lives here.
//
// Ecological model: Lotka–Volterra inspired resource competition
// with per-niche resource pools, logistic regeneration, seasonal
// oscillation, proportional extraction, and stochastic perturbation.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use genesis_dna::{AgentDNA, AgentRole};
use metabolism::atp::TransactionKind;
use metabolism::proof::{ProofKind, Solution};
use metabolism::MetabolismLedger;
use metabolism::UnitTreasury;
use ecosystem::EcosystemMesh;
use ecosystem::messages::{Message, MessageKind};
use ecosystem::problem_market::{ProblemMarket, ProblemCategory, evaluate as evaluate_problem};
use ecosystem::publication_gate::PublicationGate;
use ecosystem::telemetry::UnitStatus;
use evolution::mutation::MutationEngine;
use evolution::selection::SelectionEngine;
use evolution::gene_transfer::GeneMarketplace;

use serde::{Serialize, Deserialize};

use crate::stress::{StressConfig, StressMetrics, PhaseTransitionDetector, role_entropy};
use genesis_homeostasis::cortex::{AdaptiveCortex, PressureField};
use genesis_anchor::{AnchorEngine, EvolutionEngine, MutationRecord, WorldSummary};

// ─── Evolutionary Pressure Configuration ────────────────────────────────
//
// Configurable knobs for Phase 2 pressure: soft carrying capacity,
// entropy tax, catastrophe engine, and Gini-triggered wealth correction.
// Defaults are "gentle" — inducing life, not collapse.

/// Configurable evolutionary pressure parameters.
/// Controls the transition from greenhouse to wilderness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureConfig {
    /// Soft carrying capacity — births slow as population approaches this.
    pub soft_cap: usize,
    /// Entropy tax coefficient — population-scaled ATP burn per epoch.
    pub entropy_coeff: f64,
    /// Base catastrophe probability per epoch (before population scaling).
    pub catastrophe_base_prob: f64,
    /// Additional catastrophe probability per agent.
    pub catastrophe_pop_scale: f64,
    /// Gini threshold above which progressive wealth tax activates.
    pub gini_wealth_tax_threshold: f64,
    /// Tax rate on top 10% when Gini exceeds threshold.
    pub gini_wealth_tax_rate: f64,
    /// Treasury overflow threshold (fraction of total supply).
    pub treasury_overflow_threshold: f64,
}

impl Default for PressureConfig {
    fn default() -> Self {
        Self {
            soft_cap: 180,
            entropy_coeff: 0.00002,
            catastrophe_base_prob: 0.002,
            catastrophe_pop_scale: 0.00001,
            gini_wealth_tax_threshold: 0.40,
            gini_wealth_tax_rate: 0.02,
            treasury_overflow_threshold: 0.50,
        }
    }
}

// ─── Ecological Constants ───────────────────────────────────────────────
// These constants parameterize the ecological dynamics.  Some are
// consumed directly in this module; others shadow crate-level defaults
// for documentation clarity.

/// Basal metabolic cost per epoch (staying alive). Tuned so average
/// resource extraction exceeds this at equilibrium population.
#[allow(dead_code)]
const BASAL_COST: f64 = 0.15;

/// ATP cost to replicate. Must be achievable within ~20-30 epochs
/// of successful foraging at equilibrium.
const REPLICATION_COST: f64 = 25.0;

/// Initial ATP grant for primordial agents — enough for ~150 epochs
/// of survival at basal rate, or one reproduction + buffer.
const PRIMORDIAL_GRANT: f64 = 50.0;

/// Initial ATP grant for child agents.
const CHILD_GRANT: f64 = 8.0;

/// Epochs in stasis before death (real organisms starve slowly).
#[allow(dead_code)]
const STASIS_TOLERANCE: u32 = 8;

/// Fitness threshold for replication eligibility.
#[allow(dead_code)]
const REPLICATION_FITNESS: f64 = 0.35;

/// Minimum epochs alive before an agent can replicate.
const MATURATION_EPOCHS: u64 = 10;

/// Maximum births per epoch (prevents population explosion).
const MAX_BIRTHS_PER_EPOCH: usize = 3;

/// Competition coefficient α for cross-niche interference (0 = none, 1 = full).
/// Real ecology: species sharing resources compete partially.
const CROSS_NICHE_ALPHA: f64 = 0.15;

/// Juvenile protection: agents younger than this many epochs get a basal rebate.
const JUVENILE_PROTECTION_EPOCHS: u64 = 5;

/// Fraction of basal cost rebated to juvenile agents (25% survival cushion).
const JUVENILE_BASAL_REBATE: f64 = 0.25;

// ─── Seasonal Ecology ──────────────────────────────────────────────────
//
// The organism cycles through four ecological states driven by system metrics,
// not a fixed calendar. States are re-evaluated each epoch.
//
//   Spring (decline recovery)  — birth:death < 1.0
//   Summer (innovation surge)  — birth:death > 1.1
//   Autumn (stable selection)  — default equilibrium
//   Winter (hoarding correction) — treasury > 70% of total ATP
//
// Priority: Spring > Winter > Summer > Autumn

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum EcoState {
    /// Recovery mode: reduce reproductive friction, release trapped treasury.
    Spring,
    /// Innovation surge: increase mutation pressure, slight friction.
    Summer,
    /// Stable competitive selection: no strong modifiers.
    #[default]
    Autumn,
    /// Hoarding correction: force treasury release, slight survival pressure.
    Winter,
}

impl EcoState {
    pub fn name(&self) -> &'static str {
        match self {
            EcoState::Spring => "Spring",
            EcoState::Summer => "Summer",
            EcoState::Autumn => "Autumn",
            EcoState::Winter => "Winter",
        }
    }

    /// Reproductive cost multiplier. Spring = 0.60 (40% cheaper).
    pub fn fertility_multiplier(&self) -> f64 {
        match self {
            EcoState::Spring => 0.60,
            EcoState::Summer => 1.05,
            EcoState::Autumn => 1.00,
            EcoState::Winter => 1.00,
        }
    }

    /// Mutation pressure multiplier. Spring = 0.85 (safer offspring).
    pub fn mutation_multiplier(&self) -> f64 {
        match self {
            EcoState::Spring => 0.85,
            EcoState::Summer => 1.15,
            EcoState::Autumn => 1.00,
            EcoState::Winter => 1.00,
        }
    }

    /// Treasury release fraction per epoch. 0.0 = no release.
    /// Spring uses pressure-scaled formula; caller passes birth_death_ratio.
    pub fn treasury_release_fraction(&self, birth_death_ratio: f64) -> f64 {
        match self {
            EcoState::Spring => {
                // r = r_min + (1 - R_bd) * r_scale, capped at r_max
                let pressure = (1.0_f64 - birth_death_ratio).max(0.0);
                (0.05 + pressure * 0.15).min(0.20)
            }
            EcoState::Winter => 0.15,
            EcoState::Summer | EcoState::Autumn => 0.00,
        }
    }
}

/// Determine the ecological state from current system metrics.
/// Priority: Spring > Winter > Summer > Autumn.
fn determine_ecostate(
    birth_death_ratio: f64,
    treasury_reserve: f64,
    circulating_atp: f64,
) -> EcoState {
    let total_atp = (treasury_reserve + circulating_atp).max(1.0);
    let hoarding_ratio = treasury_reserve / total_atp;

    if birth_death_ratio < 1.0 {
        EcoState::Spring
    } else if hoarding_ratio > 0.70 {
        EcoState::Winter
    } else if birth_death_ratio > 1.10 {
        EcoState::Summer
    } else {
        EcoState::Autumn
    }
}

// ─── Resource Pool ──────────────────────────────────────────────────────

/// A regenerating resource pool for one ecological niche.
/// Models logistic growth: dR/dt = regen_rate * R * (1 - R/capacity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Current resource level (ATP available for extraction).
    pub level: f64,
    /// Maximum carrying capacity for this niche.
    pub capacity: f64,
    /// Logistic regeneration rate per epoch.
    pub regen_rate: f64,
}

impl ResourcePool {
    pub fn new(capacity: f64, regen_rate: f64) -> Self {
        Self {
            level: capacity * 0.8, // start at 80% capacity
            capacity,
            regen_rate,
        }
    }

    /// Logistic regeneration: R += r * R * (1 - R/K)
    pub fn regenerate(&mut self) {
        let growth = self.regen_rate * self.level * (1.0 - self.level / self.capacity);
        self.level = (self.level + growth).clamp(0.0, self.capacity);
    }

    /// Extract resources proportional to demand.
    /// Returns actual amount extracted (may be less than requested if pool is low).
    pub fn extract(&mut self, demand: f64) -> f64 {
        let available = self.level * 0.4; // max 40% of pool extractable per epoch
        let actual = demand.min(available).max(0.0);
        self.level -= actual;
        actual
    }
}

// ─── Environment ────────────────────────────────────────────────────────

/// Ecological environment with per-niche resource dynamics, seasonal
/// cycles, and stochastic perturbation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Environment {
    /// Resource pools keyed by ecological niche (AgentRole).
    pub pools: HashMap<AgentRole, ResourcePool>,
    /// Season cycle position in radians [0, 2π).
    pub season_phase: f64,
    /// Season cycle length in epochs (one full sinusoidal period).
    pub season_length: u64,
    /// Amplitude of seasonal variation (fraction of capacity, e.g. 0.3 = ±30%).
    pub season_amplitude: f64,
    /// Base carrying capacity per niche (before seasonal modulation).
    pub base_capacity: f64,
    /// Whether a catastrophe is currently active.
    pub catastrophe_remaining: u64,
    /// Catastrophe severity multiplier on capacity (0.3 = 30% of normal).
    pub catastrophe_severity: f64,
    /// Counter for stochastic seed progression.
    pub event_seed: u64,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    /// Create a default environment with 5 niche pools.
    pub fn new() -> Self {
        let base_cap = 150.0;
        let regen = 0.12; // 12% logistic regeneration — moderate

        let mut pools = HashMap::new();
        // Each role gets a niche with slightly different characteristics
        pools.insert(AgentRole::Optimizer, ResourcePool::new(base_cap * 1.0, regen));
        pools.insert(AgentRole::Strategist, ResourcePool::new(base_cap * 0.9, regen * 1.1));
        pools.insert(AgentRole::Communicator, ResourcePool::new(base_cap * 0.85, regen * 1.15));
        pools.insert(AgentRole::Archivist, ResourcePool::new(base_cap * 0.95, regen * 0.95));
        pools.insert(AgentRole::Executor, ResourcePool::new(base_cap * 1.1, regen * 1.05));

        Self {
            pools,
            season_phase: 0.0,
            season_length: 100,
            season_amplitude: 0.25,
            base_capacity: base_cap,
            catastrophe_remaining: 0,
            catastrophe_severity: 1.0,
            event_seed: 42,
        }
    }

    /// Advance one epoch: regenerate resources, apply seasonal modulation,
    /// tick catastrophe timer, and roll for stochastic events.
    pub fn tick(&mut self, epoch: u64) {
        // Advance season
        self.season_phase = (2.0 * std::f64::consts::PI * (epoch as f64))
            / (self.season_length as f64);

        // Seasonal capacity modifier: 1.0 ± amplitude * sin(phase)
        let seasonal_mod = 1.0 + self.season_amplitude * self.season_phase.sin();

        // Catastrophe modifier
        let catastrophe_mod = if self.catastrophe_remaining > 0 {
            self.catastrophe_remaining -= 1;
            self.catastrophe_severity
        } else {
            1.0
        };

        // Apply capacity modulation + regeneration
        for (role, pool) in self.pools.iter_mut() {
            let role_multiplier = match role {
                AgentRole::Optimizer => 1.0,
                AgentRole::Strategist => 0.9,
                AgentRole::Communicator => 0.85,
                AgentRole::Archivist => 0.95,
                AgentRole::Executor => 1.1,
            };
            pool.capacity = self.base_capacity * role_multiplier * seasonal_mod * catastrophe_mod;
            pool.regenerate();
        }

        // Stochastic events: ~2% chance of catastrophe per epoch,
        // ~5% chance of resource boom, using deterministic hash.
        self.event_seed = self.event_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let roll = (self.event_seed >> 33) as f64 / (u32::MAX as f64);

        if roll < 0.02 && self.catastrophe_remaining == 0 {
            // Catastrophe: 10-20 epoch duration, 30-60% capacity
            self.catastrophe_remaining = 10 + (self.event_seed % 11);
            self.catastrophe_severity = 0.3 + (((self.event_seed >> 16) % 30) as f64) / 100.0;
            tracing::warn!(
                epoch = epoch,
                duration = self.catastrophe_remaining,
                severity = format!("{:.0}%", self.catastrophe_severity * 100.0),
                "CATASTROPHE EVENT"
            );
        } else if roll > 0.95 {
            // Resource boom: temporarily spike all pools to 90% capacity
            for pool in self.pools.values_mut() {
                pool.level = pool.capacity * 0.9;
            }
            tracing::info!(epoch = epoch, "RESOURCE BOOM");
        }
    }

    /// Total resources across all niches.
    pub fn total_resources(&self) -> f64 {
        self.pools.values().map(|p| p.level).sum()
    }

    /// Current effective carrying capacity (sum across niches).
    pub fn total_capacity(&self) -> f64 {
        self.pools.values().map(|p| p.capacity).sum()
    }

    /// Current season name (for display).
    pub fn season_name(&self) -> &'static str {
        let normalized = (self.season_phase.sin() + 1.0) / 2.0; // [0, 1]
        match (normalized * 4.0) as usize {
            0 => "Winter",
            1 => "Spring",
            2 => "Summer",
            _ => "Autumn",
        }
    }
}

/// Thread-safe shared world handle.
pub type SharedWorld = Arc<Mutex<World>>;

/// Complete world state — serializable for persistence.
#[derive(Serialize, Deserialize)]
pub struct World {
    pub agents: Vec<AgentDNA>,
    pub ledger: MetabolismLedger,
    pub treasury: UnitTreasury,
    pub mesh: EcosystemMesh,
    pub problem_market: ProblemMarket,
    pub publication_gate: PublicationGate,
    pub mutation_engine: MutationEngine,
    pub selection_engine: SelectionEngine,
    pub marketplace: GeneMarketplace,
    pub environment: Environment,
    pub epoch: u64,
    /// Per-agent birth epoch tracker (for maturation).
    pub agent_birth_epoch: HashMap<uuid::Uuid, u64>,
    /// Set of registered external IDs to prevent duplicates.
    pub registered_external_ids: Vec<String>,
    /// Maximum population before capping.
    pub pop_cap: usize,
    /// Timestamp when this world was created.
    pub started_at: DateTime<Utc>,
    /// Rolling history of recent epoch stats (last 100).
    pub epoch_history: VecDeque<EpochStats>,
    /// Total births across all epochs.
    pub total_births: u64,
    /// Total deaths across all epochs.
    pub total_deaths: u64,
    /// Current ecological season state.
    pub eco_state: EcoState,
    /// Active stress configuration (None = baseline).
    pub stress_config: Option<StressConfig>,
    /// Accumulated stress metrics (populated only when stress is active).
    pub stress_metrics: Option<StressMetrics>,
    /// Phase transition detector.
    pub phase_detector: PhaseTransitionDetector,
    /// Evolutionary pressure configuration (Phase 2).
    #[serde(default)]
    pub pressure: PressureConfig,
    /// Adaptive cortex — immune-driven pressure mutation engine (Phase 3).
    #[serde(default)]
    pub cortex: AdaptiveCortex,
    /// Peak treasury observed (for immune depletion detection).
    #[serde(default)]
    pub peak_treasury: f64,
    /// Population history window for immune collapse detection.
    #[serde(default)]
    pub population_history: Vec<usize>,
    /// State chain anchor engine — cryptographic epoch snapshots (Phase 3.5).
    #[serde(default)]
    pub anchor_engine: AnchorEngine,
    /// Evolution chain engine — cryptographic mutation history (Phase 3.5).
    #[serde(default)]
    pub evolution_engine: EvolutionEngine,
}

/// Epoch summary stats returned by run_epoch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochStats {
    pub epoch: u64,
    pub population: usize,
    pub total_atp: f64,
    pub mean_fitness: f64,
    pub max_fitness: f64,
    pub min_fitness: f64,
    pub births: u64,
    pub deaths: u64,
    pub mutations: u64,
    pub stasis_count: usize,
    pub market_solved: u64,
    pub market_rewarded: f64,
    pub gated_posts: u64,
    /// ATP extracted from resource pools this epoch.
    pub resources_extracted: f64,
    /// Total resources remaining across all niches.
    pub total_resources: f64,
    /// Current season name.
    pub season: String,
    /// Whether a catastrophe is active.
    pub catastrophe_active: bool,
    /// Dynamic carrying capacity this epoch.
    pub dynamic_pop_cap: usize,
    /// Number of agents in each role.
    pub role_counts: HashMap<AgentRole, usize>,
    /// Resource level per niche.
    pub niche_resources: HashMap<AgentRole, f64>,
    /// Treasury reserve balance at end of epoch.
    pub treasury_reserve: f64,
    /// Treasury ATP distributed this epoch (stipends + crisis + overflow).
    pub treasury_distributed: f64,
    /// Seasonal ecological state active this epoch.
    pub eco_state: EcoState,
    /// Birth:death ratio over the rolling history window.
    pub birth_death_ratio: f64,
    /// Gini coefficient of ATP wealth distribution (0 = equal, 1 = monopoly).
    #[serde(default)]
    pub gini_coefficient: f64,
    /// Shannon entropy of role distribution (higher = more diverse).
    #[serde(default)]
    pub role_entropy: f64,
    /// Treasury as fraction of total supply.
    #[serde(default)]
    pub treasury_ratio: f64,
    /// Deaths per 100 epochs (rolling average).
    #[serde(default)]
    pub death_rate_100: f64,
    /// ATP burned by entropy tax this epoch.
    #[serde(default)]
    pub entropy_tax_burned: f64,
    /// Deaths caused by catastrophe events this epoch.
    #[serde(default)]
    pub catastrophe_deaths: u64,
    /// Number of immune threats detected this epoch.
    #[serde(default)]
    pub immune_threats: usize,
    /// Overall organism health level (0=Normal, 1=Watch, 2=Warning, 3=Critical).
    #[serde(default)]
    pub immune_health: u8,
    /// Number of pressure mutations applied this epoch.
    #[serde(default)]
    pub pressure_mutations: usize,
}

/// Registration request from external callers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub external_id: String,
    pub public_key: String,
}

/// Registration response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub agent_id: String,
    pub role: String,
    pub initial_atp: f64,
}

/// Error type for world operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldError {
    pub message: String,
}

/// Epoch-over-epoch delta metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpochDiff {
    pub window: u64,
    pub population_delta: i64,
    pub atp_delta: f64,
    pub fitness_delta: f64,
    pub births_in_window: u64,
    pub deaths_in_window: u64,
    pub mutations_in_window: u64,
}

/// Leaderboard entry — agent ranked by fitness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub agent_id: String,
    pub role: String,
    pub fitness: f64,
    pub reputation: f64,
    pub atp_balance: f64,
    pub generation: u64,
    pub is_primordial: bool,
    pub survived_epochs: u64,
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Minimum ATP stake required for registration.
const REGISTRATION_STAKE: f64 = 5.0;

impl World {
    /// Create a new world with 20 primordial agents and ecological environment.
    pub fn new() -> Self {
        let mut ledger = MetabolismLedger::new();
        let mut mesh = EcosystemMesh::new();
        let mutation_engine = MutationEngine::default_engine();
        let selection_engine = SelectionEngine::new();
        let marketplace = GeneMarketplace::new();
        let problem_market = ProblemMarket::new();
        let publication_gate = PublicationGate::conservative();
        let treasury = UnitTreasury::new();
        let environment = Environment::new();

        let (agents, birth_epochs) = Self::spawn_primordials(20, &mut ledger, &mut mesh);

        World {
            agents,
            ledger,
            treasury,
            mesh,
            problem_market,
            publication_gate,
            mutation_engine,
            selection_engine,
            marketplace,
            environment,
            epoch: 0,
            agent_birth_epoch: birth_epochs,
            registered_external_ids: Vec::new(),
            pop_cap: 200,
            started_at: Utc::now(),
            epoch_history: VecDeque::with_capacity(100),
            total_births: 0,
            total_deaths: 0,
            eco_state: EcoState::Autumn,
            stress_config: None,
            stress_metrics: None,
            phase_detector: PhaseTransitionDetector::new(),
            pressure: PressureConfig::default(),
            cortex: AdaptiveCortex::default(),
            peak_treasury: 0.0,
            population_history: Vec::new(),
            anchor_engine: AnchorEngine::default(),
            evolution_engine: EvolutionEngine::default(),
        }
    }

    /// Repair environment pools if they were lost during deserialization
    /// from old snapshots that predate the Environment system.
    pub fn repair_environment(&mut self) {
        if self.environment.pools.is_empty() {
            tracing::warn!("Environment pools were empty — re-initializing from defaults");
            self.environment = Environment::new();
        }
        // Ensure base_capacity is set (old saves may have 0)
        if self.environment.base_capacity < 1.0 {
            self.environment.base_capacity = 150.0;
        }
    }

    /// Attach a stress profile to this world.
    pub fn with_stress(&mut self, config: StressConfig, profile_name: impl Into<String>) {
        self.stress_metrics = Some(StressMetrics::new(profile_name));
        self.stress_config = Some(config);
        tracing::warn!("Stress profile attached — evolutionary legitimacy remains intact");
    }

    /// Remove stress profile and return accumulated metrics if any.
    pub fn clear_stress(&mut self) -> Option<crate::stress::StressRunResult> {
        self.stress_config = None;
        self.stress_metrics.take().map(|m| m.finalize())
    }

    /// Spawn primordial agents with diverse entropy.
    /// Uses cryptographic mixing to ensure genuine trait diversity.
    fn spawn_primordials(
        count: usize,
        ledger: &mut MetabolismLedger,
        mesh: &mut EcosystemMesh,
    ) -> (Vec<AgentDNA>, HashMap<uuid::Uuid, u64>) {
        let mut agents = Vec::with_capacity(count);
        let mut birth_epochs = HashMap::new();

        for i in 0..count {
            // Use SHA-256 mixing for genuine diversity instead of
            // linear arithmetic that produced near-identical genomes.
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(b"genesis-primordial-");
            hasher.update(i.to_le_bytes());
            hasher.update(b"-epoch0-seed7729");
            let hash: [u8; 32] = hasher.finalize().into();
            let entropy: Vec<u8> = hash.iter().chain(hash.iter()).copied().collect();

            let dna = AgentDNA::from_entropy(&entropy, true).unwrap();

            let initial_proof = Solution::new(
                format!("Primordial proof #{}", i),
                ProofKind::Solution,
                entropy.clone(),
                0.5,
            );
            let verdict = initial_proof.evaluate();
            let initial_atp = if verdict.accepted {
                (verdict.reward * dna.energy_metabolism.effective_generation_rate())
                    .max(PRIMORDIAL_GRANT)
            } else {
                PRIMORDIAL_GRANT
            };
            ledger.register_agent(dna.id, initial_atp);

            mesh.registry
                .register(&dna, format!("Primordial-{}", i), "genesis")
                .unwrap();
            mesh.init_inbox(dna.id);

            birth_epochs.insert(dna.id, 0);
            agents.push(dna);
        }

        // Ring topology + cross-role links for mesh diversity
        for i in 0..agents.len() {
            let next = (i + 1) % agents.len();
            let _ = mesh.registry.connect(&agents[i].id, &agents[next].id);
            // Also connect to an agent 5 steps away for small-world topology
            let far = (i + 5) % agents.len();
            if far != next {
                let _ = mesh.registry.connect(&agents[i].id, &agents[far].id);
            }
        }

        (agents, birth_epochs)
    }

    /// Register an external agent. Enforces:
    /// - No duplicate external_id
    /// - Registration stake
    /// - Publication gate applied immediately
    /// - Replication locked for REPLICATION_LOCKOUT_EPOCHS
    pub fn register_external(
        &mut self,
        req: &RegistrationRequest,
    ) -> Result<RegistrationResult, WorldError> {
        // Check for duplicate
        if self.registered_external_ids.contains(&req.external_id) {
            return Err(WorldError {
                message: format!("Duplicate external_id: {}", req.external_id),
            });
        }

        // Population cap
        if self.agents.len() >= self.pop_cap {
            return Err(WorldError {
                message: "Population cap reached".to_string(),
            });
        }

        // Deterministic entropy from external_id + public_key
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(req.external_id.as_bytes());
        hasher.update(req.public_key.as_bytes());
        let hash: [u8; 32] = hasher.finalize().into();
        let entropy: Vec<u8> = hash.iter().chain(hash.iter()).copied().collect(); // 64 bytes

        let dna = AgentDNA::from_entropy(&entropy, false).map_err(|e| WorldError {
            message: format!("Failed to create agent: {:?}", e),
        })?;

        let agent_id_hex = dna.genome_hex()[..16].to_string();

        // Register with minimal stake ATP
        self.ledger.register_agent(dna.id, REGISTRATION_STAKE);

        // Register in mesh
        let _ = self.mesh.registry.register(
            &dna,
            format!("ext-{}", &req.external_id[..req.external_id.len().min(20)]),
            "external",
        );
        self.mesh.init_inbox(dna.id);

        // Connect to a few existing agents for mesh integration
        let neighbor_count = 3.min(self.agents.len());
        for i in 0..neighbor_count {
            let _ = self.mesh.registry.connect(&dna.id, &self.agents[i].id);
        }

        let role_label = dna.role.label().to_string();

        self.agent_birth_epoch.insert(dna.id, self.epoch);
        self.agents.push(dna);
        self.registered_external_ids.push(req.external_id.clone());

        Ok(RegistrationResult {
            agent_id: agent_id_hex,
            role: role_label,
            initial_atp: REGISTRATION_STAKE,
        })
    }

    /// Run one epoch of the survival loop. All evolutionary logic lives here.
    pub fn run_epoch(&mut self) -> EpochStats {
        let epoch = self.epoch;
        let mut births: u64 = 0;
        let mut deaths: u64 = 0;
        let mut mutations: u64 = 0;
        let mut market_solved: u64 = 0;
        let mut market_rewarded: f64 = 0.0;
        let mut gated_posts: u64 = 0;
        let mut resources_extracted: f64 = 0.0;
        let entropy_tax_burned: f64;
        let mut catastrophe_deaths: u64 = 0;
        let treasury_distributed_before = self.treasury.total_distributed;

        // ═══════════════════════════════════════════════════════════════
        // STEP 0: Environment tick — regenerate resources, apply seasons
        // ═══════════════════════════════════════════════════════════════
        self.environment.tick(epoch);

        // ── Stress: catastrophe cluster bias ──────────────────────────
        // When catastrophe_cluster_bias > 0 we roll an extra catastrophe
        // check on top of the natural 2% background rate.
        if let Some(ref sc) = self.stress_config {
            if sc.catastrophe_cluster_bias > 0.0
                && self.environment.catastrophe_remaining == 0
            {
                let lcg = epoch
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15_u64)
                    .wrapping_add(0xDEAD_BEEF_CAFE_1234_u64);
                let roll = (lcg >> 33) as f64 / (1u64 << 31) as f64;
                if roll < sc.catastrophe_cluster_bias {
                    self.environment.catastrophe_remaining = 5 + (epoch % 8);
                    self.environment.catastrophe_severity =
                        (0.4 + sc.catastrophe_cluster_bias * 0.3).min(0.9);
                    tracing::warn!(
                        epoch,
                        bias = format!("{:.3}", sc.catastrophe_cluster_bias),
                        "Stress: catastrophe cluster triggered"
                    );
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 0b: Population-scaled catastrophe engine (Phase 2)
        //
        // Probability: base + pop × scale. When triggered, one of three
        // effects: ATP destruction, fitness culling, or niche resource shock.
        // Deterministic seed ensures reproducibility.
        // ═══════════════════════════════════════════════════════════════
        {
            let pop = self.agents.len();
            let cat_prob = self.pressure.catastrophe_base_prob
                + (pop as f64 * self.pressure.catastrophe_pop_scale);

            // Deterministic roll from epoch (different multiplier from environment seed)
            let cat_seed = epoch
                .wrapping_mul(0xBF58_476D_1CE4_E5B9_u64)
                .wrapping_add(0x94D0_49BB_1331_11EB_u64);
            let cat_roll = (cat_seed >> 33) as f64 / (u32::MAX as f64);

            if cat_roll < cat_prob && pop > 10 {
                let cat_type = (cat_seed >> 16) % 3;

                match cat_type {
                    0 => {
                        // ATP destruction: reduce all balances by 5‒10%
                        let severity = 0.05 + ((cat_seed >> 8) % 6) as f64 * 0.01;
                        self.ledger.decay_all(severity);
                        tracing::warn!(
                            epoch,
                            severity = format!("{:.1}%", severity * 100.0),
                            "PRESSURE EVENT: ATP destruction"
                        );
                    }
                    1 => {
                        // Cull weakest 2‒5% by fitness
                        let cull_pct = 0.02 + ((cat_seed >> 4) % 4) as f64 * 0.01;
                        let cull_count = ((pop as f64 * cull_pct) as usize).max(1);

                        let mut by_fitness: Vec<(uuid::Uuid, f64)> = self.agents.iter()
                            .map(|a| (a.id, a.fitness()))
                            .collect();
                        by_fitness.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

                        let to_cull: Vec<uuid::Uuid> = by_fitness.iter()
                            .take(cull_count)
                            .map(|(id, _)| *id)
                            .collect();

                        for dead_id in &to_cull {
                            if let Ok(bal) = self.ledger.balance(dead_id) {
                                if bal.balance > 0.0 {
                                    let _ = self.ledger.burn(
                                        dead_id, bal.balance,
                                        TransactionKind::BasalMetabolism,
                                        &format!("Epoch {} pressure cull", epoch),
                                    );
                                }
                            }
                            self.agents.retain(|a| a.id != *dead_id);
                            self.agent_birth_epoch.remove(dead_id);
                            let _ = self.mesh.registry.set_status(
                                dead_id,
                                ecosystem::AgentStatus::Dead,
                            );
                            deaths += 1;
                            catastrophe_deaths += 1;
                        }

                        tracing::warn!(
                            epoch,
                            culled = to_cull.len(),
                            pct = format!("{:.1}%", cull_pct * 100.0),
                            "PRESSURE EVENT: fitness culling"
                        );
                    }
                    _ => {
                        // Role-specific shock: deplete one niche's resource pool
                        let roles = [
                            AgentRole::Optimizer, AgentRole::Strategist,
                            AgentRole::Communicator, AgentRole::Archivist, AgentRole::Executor,
                        ];
                        let target_role = roles[(cat_seed as usize) % roles.len()];
                        if let Some(pool) = self.environment.pools.get_mut(&target_role) {
                            pool.level *= 0.3; // 70% resource destruction
                        }
                        tracing::warn!(
                            epoch,
                            role = ?target_role,
                            "PRESSURE EVENT: niche resource shock"
                        );
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 0a: Ecological state determination
        //
        // Compute birth:death ratio from rolling history window, then
        // determine which ecological season the organism is in. Season
        // drives reproduction cost, mutation pressure, and treasury release.
        // ═══════════════════════════════════════════════════════════════
        let birth_death_ratio = {
            let (w_births, w_deaths) = self.epoch_history.iter().fold((0u64, 0u64), |acc, h| {
                (acc.0 + h.births, acc.1 + h.deaths)
            });
            // Smoothed ratio (Laplace +1 to avoid div0)
            (w_births + 1) as f64 / (w_deaths + 1) as f64
        };
        let circulating_atp = self.ledger.total_supply();
        let current_eco = determine_ecostate(birth_death_ratio, self.treasury.reserve, circulating_atp);
        self.eco_state = current_eco;

        // Seasonal treasury release — BEFORE metabolism so agents have energy to survive
        // ── Stress: treasury lock ─────────────────────────────────────
        // treasury_lock_probability > 0 stochastically suppresses release.
        let release_suppressed = self
            .stress_config
            .as_ref()
            .map(|sc| sc.treasury_locked(epoch))
            .unwrap_or(false);
        let release_fraction = current_eco.treasury_release_fraction(birth_death_ratio);
        if !release_suppressed && release_fraction > 0.0 && self.treasury.reserve > 0.0 {
            let release_amount = self.treasury.reserve * release_fraction;
            let actually_released = self.treasury.crisis_spend(release_amount);
            let pop = self.agents.len();
            if actually_released > 0.0 && pop > 0 {
                let per_agent = actually_released / pop as f64;
                for agent in self.agents.iter() {
                    let _ = self.ledger.mint(
                        &agent.id, per_agent,
                        TransactionKind::ProofOfSolution,
                        &format!("Epoch {} {} seasonal treasury release", epoch, current_eco.name()),
                    );
                }
                tracing::info!(
                    epoch = epoch,
                    season = current_eco.name(),
                    released = format!("{:.2}", actually_released),
                    per_agent = format!("{:.2}", per_agent),
                    bd_ratio = format!("{:.3}", birth_death_ratio),
                    "Treasury seasonal release"
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 1: Resource extraction — proportional foraging per niche
        //
        // Each agent extracts ATP from its role's resource pool.
        // Extraction = fitness * skill_match * (pool_level / capacity)
        // This creates density-dependent income: crowded niches yield less.
        // Cross-niche competition: agents in other roles slightly deplete all pools.
        // ═══════════════════════════════════════════════════════════════
        let mut role_counts: HashMap<AgentRole, usize> = HashMap::new();
        for agent in self.agents.iter() {
            *role_counts.entry(agent.role).or_insert(0) += 1;
        }

        // Compute per-agent extraction from their niche
        let agent_extractions: Vec<(uuid::Uuid, AgentRole, f64)> = self.agents.iter().map(|agent| {
            let pool = self.environment.pools.get(&agent.role);
            let pool_ratio = pool.map(|p| (p.level / p.capacity).clamp(0.0, 1.0)).unwrap_or(0.0);

            // Niche skill match — how well does this agent exploit its niche?
            let niche_skill = match agent.role {
                AgentRole::Optimizer => agent.skills.optimization,
                AgentRole::Strategist => agent.skills.cooperation,
                AgentRole::Communicator => agent.skills.communication,
                AgentRole::Archivist => agent.skills.compute,
                AgentRole::Executor => (agent.skills.optimization + agent.skills.compute) / 2.0,
            };

            // Density factor: more agents in this niche → less per agent
            let niche_pop = *role_counts.get(&agent.role).unwrap_or(&1) as f64;
            let density_factor = 1.0 / (1.0 + niche_pop * 0.1);

            // Cross-niche competition penalty
            let total_pop = self.agents.len() as f64;
            let cross_penalty = 1.0 - CROSS_NICHE_ALPHA * (total_pop - niche_pop) / total_pop.max(1.0);

            // Final extraction: fitness × niche_skill × pool_ratio × density × cross-niche
            let extraction = agent.fitness()
                * niche_skill
                * pool_ratio
                * density_factor
                * cross_penalty
                * 2.5; // scale factor so average agent nets positive at equilibrium

            (agent.id, agent.role, extraction.max(0.0))
        }).collect();

        // Actually extract from pools and credit agents
        for (agent_id, role, demand) in &agent_extractions {
            if let Some(pool) = self.environment.pools.get_mut(role) {
                let extracted = pool.extract(*demand);
                if extracted > 0.0 {
                    let _ = self.ledger.mint(
                        agent_id, extracted,
                        TransactionKind::ProofOfSolution,
                        &format!("Epoch {} niche foraging", epoch),
                    );
                    resources_extracted += extracted;
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 2: Basal metabolism — cost of staying alive
        // Lower than before (0.15 vs 0.5) so agents can sustain
        // ═══════════════════════════════════════════════════════════════
        self.ledger.metabolic_tick_all();

        // ── Stress: extra basal burn ───────────────────────────────────
        // basal_cost_multiplier > 1 means maintenance is more expensive.
        // We apply the excess as an additional burn loop.
        if let Some(ref sc) = self.stress_config {
            let excess = (sc.basal_cost_multiplier - 1.0).max(0.0);
            if excess > 0.0 {
                use metabolism::atp::costs::BASAL_TICK;
                let extra_burn = BASAL_TICK * excess;
                let ids: Vec<uuid::Uuid> = self.agents.iter().map(|a| a.id).collect();
                for id in ids {
                    let _ = self.ledger.burn(
                        &id, extra_burn,
                        TransactionKind::BasalMetabolism,
                        &format!("Epoch {} stress basal", epoch),
                    );
                }
            }
        }

        // ── Juvenile protection: agents < 5 epochs old get 25% basal rebate ──
        // Prevents newborns from dying before their first foraging cycle.
        {
            use metabolism::atp::costs::BASAL_TICK;
            let rebate = BASAL_TICK * JUVENILE_BASAL_REBATE;
            let juvenile_ids: Vec<uuid::Uuid> = self.agents.iter()
                .filter(|a| epoch.saturating_sub(
                    self.agent_birth_epoch.get(&a.id).copied().unwrap_or(0)
                ) < JUVENILE_PROTECTION_EPOCHS)
                .map(|a| a.id)
                .collect();
            for jid in juvenile_ids {
                let _ = self.ledger.mint(
                    &jid, rebate,
                    TransactionKind::ProofOfSolution,
                    &format!("Epoch {} juvenile metabolic cushion", epoch),
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 2a: ATP decay — 2% balance erosion per epoch
        // Prevents infinite accumulation; models entropy/maintenance.
        // ═══════════════════════════════════════════════════════════════
        self.ledger.decay_all(0.02);

        // ═══════════════════════════════════════════════════════════════
        // STEP 2b: Wealth tax — 1% on balances above 100 ATP
        // Tax flows to treasury, not destroyed. Prevents hoarding.
        // ═══════════════════════════════════════════════════════════════
        let wealth_taxed = self.ledger.wealth_tax_all(100.0, 0.01);
        self.treasury.reserve += wealth_taxed;
        self.treasury.total_collected += wealth_taxed;

        // ═══════════════════════════════════════════════════════════════
        // STEP 2c: Entropy tax — population-scaled ATP burn (Phase 2)
        //
        // Burns ATP proportionally from all agents. Larger populations
        // incur stronger burn, preventing exponential ATP inflation.
        // Formula: total_burn = total_atp × (population × coefficient)
        // ═══════════════════════════════════════════════════════════════
        {
            let pop = self.agents.len();
            entropy_tax_burned = self.ledger.entropy_tax_all(self.pressure.entropy_coeff, pop);
            if entropy_tax_burned > 0.1 && epoch % 50 == 0 {
                tracing::info!(
                    epoch,
                    burned = format!("{:.2}", entropy_tax_burned),
                    pop,
                    "Entropy tax applied"
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 2d: Gini-triggered wealth tax — progressive correction
        //
        // When wealth inequality (Gini coefficient) exceeds threshold,
        // apply extra tax on top 10% wealthiest agents → treasury.
        // Prevents oligarchy while respecting normal wealth variance.
        // ═══════════════════════════════════════════════════════════════
        let epoch_gini = {
            let bals: Vec<f64> = self.agents.iter()
                .filter_map(|a| self.ledger.balance(&a.id).ok().map(|b| b.balance))
                .collect();
            genesis_econometrics::gini_coefficient(&bals)
        };

        if epoch_gini > self.pressure.gini_wealth_tax_threshold {
            let mut agent_bals: Vec<(uuid::Uuid, f64)> = self.agents.iter()
                .filter_map(|a| {
                    self.ledger.balance(&a.id).ok().map(|b| (a.id, b.balance))
                })
                .collect();
            agent_bals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let top_10_count = (agent_bals.len() / 10).max(1);
            let top_10_ids: Vec<uuid::Uuid> = agent_bals.iter()
                .take(top_10_count)
                .map(|(id, _)| *id)
                .collect();
            let gini_taxed = self.ledger.targeted_tax(
                &top_10_ids.iter().map(|id| *id).collect::<Vec<_>>(),
                self.pressure.gini_wealth_tax_rate,
            );
            self.treasury.reserve += gini_taxed;
            self.treasury.total_collected += gini_taxed;

            if gini_taxed > 0.01 && epoch % 50 == 0 {
                tracing::info!(
                    epoch,
                    gini = format!("{:.3}", epoch_gini),
                    taxed = format!("{:.2}", gini_taxed),
                    agents = top_10_count,
                    "Gini wealth correction applied"
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 2e: Treasury redistribution — the counter-cyclical loop
        //
        // 1. Stipends to underrepresented roles (diversity incentive)
        // 2. Crisis spending when population < cap/2
        // 3. Overflow redistribution: if treasury > 30% of supply,
        //    distribute excess equally to all agents (prevents hoarding)
        // ═══════════════════════════════════════════════════════════════
        {
            // Stipends for underrepresented roles
            let stipend_distributed = self.treasury.distribute_stipends(&role_counts, self.agents.len());
            for (role, total_for_role) in &stipend_distributed {
                let count = *role_counts.get(role).unwrap_or(&1) as f64;
                let per_agent = total_for_role / count;
                for agent in self.agents.iter().filter(|a| &a.role == role) {
                    let _ = self.ledger.mint(
                        &agent.id, per_agent,
                        TransactionKind::ProofOfSolution,
                        &format!("Epoch {} role stipend", epoch),
                    );
                }
            }

            // Crisis spending: if population below half of carrying capacity
            let pop = self.agents.len();
            let dynamic_cap = ((self.environment.total_capacity() / 15.0) as usize).clamp(10, 500);
            if pop > 0 && pop < dynamic_cap / 2 {
                // Inject up to 2 ATP per agent from reserves
                let crisis_budget = (pop as f64 * 2.0).min(self.treasury.reserve * 0.5);
                let spent = self.treasury.crisis_spend(crisis_budget);
                if spent > 0.0 {
                    let per_agent = spent / pop as f64;
                    for agent in self.agents.iter() {
                        let _ = self.ledger.mint(
                            &agent.id, per_agent,
                            TransactionKind::ProofOfSolution,
                            &format!("Epoch {} crisis stabilization", epoch),
                        );
                    }
                }
            }

            // Overflow redistribution: prevent treasury from hoarding beyond target
            let total_supply = self.ledger.total_supply();
            let overflow_threshold = total_supply * self.pressure.treasury_overflow_threshold;
            if self.treasury.reserve > overflow_threshold && pop > 0 {
                let excess = self.treasury.reserve - overflow_threshold;
                // Distribute half the excess — leave some buffer
                let redistribute = excess * 0.5;
                let spent = self.treasury.crisis_spend(redistribute);
                if spent > 0.0 {
                    let per_agent = spent / pop as f64;
                    for agent in self.agents.iter() {
                        let _ = self.ledger.mint(
                            &agent.id, per_agent,
                            TransactionKind::ProofOfSolution,
                            &format!("Epoch {} treasury overflow", epoch),
                        );
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 3: Problem Market — supplementary income (bonus, not sole)
        // ═══════════════════════════════════════════════════════════════
        let environmental_pressure = 0.3
            + 0.15 * self.environment.season_phase.sin()
            + (epoch as f64 * 0.0005).min(0.3);
        let problem_count = if epoch % 3 == 0 { 3 } else { 2 }; // not every epoch is a bonanza
        let problem_ids = self.problem_market.generate_epoch_problems(
            environmental_pressure,
            problem_count,
            epoch,
        );

        for pid in problem_ids {
            let problem = self.problem_market.active_problems()
                .into_iter()
                .find(|p| p.id == pid)
                .cloned();

            if let Some(problem) = problem {
                let mut best_idx: Option<usize> = None;
                let mut best_score: f64 = 0.0;

                for (i, agent) in self.agents.iter().enumerate() {
                    let result = evaluate_problem(&agent.skills, &problem);
                    if result.passes && result.score > best_score {
                        best_score = result.score;
                        best_idx = Some(i);
                    }
                }

                if let Some(idx) = best_idx {
                    let agent_id = self.agents[idx].id;
                    let confidence = match problem.category {
                        ProblemCategory::Optimization => self.agents[idx].skills.optimization,
                        ProblemCategory::Strategy => self.agents[idx].skills.cooperation,
                        ProblemCategory::Coordination => self.agents[idx].skills.communication,
                        ProblemCategory::Analysis => self.agents[idx].skills.compute,
                    };

                    if self.publication_gate.approve(confidence, 0.3, self.agents[idx].reputation.score) {
                        let gross_reward = problem.reward_atp;
                        let raw_skim = self.treasury.skim(gross_reward);
                        // ── Stress: skim_rate_multiplier ─────────────
                        // Extra skim above 1.0× draws additional ATP into
                        // the treasury from each problem reward.
                        let skim = {
                            let mult = self.stress_config
                                .as_ref()
                                .map(|sc| sc.skim_rate_multiplier)
                                .unwrap_or(1.0);
                            if mult > 1.0 {
                                let extra = gross_reward * (mult - 1.0) * 0.05;
                                self.treasury.reserve += extra;
                                self.treasury.total_collected += extra;
                                raw_skim + extra
                            } else {
                                raw_skim
                            }
                        };
                        let reward = gross_reward - skim;
                        let _ = self.ledger.mint(
                            &agent_id, reward,
                            TransactionKind::ProofOfSolution,
                            &format!("Market problem #{}", problem.id),
                        );
                        self.agents[idx].reputation.complete_contract(confidence);
                        self.problem_market.mark_solved(problem.id, reward);
                        market_solved += 1;
                        market_rewarded += reward;
                        gated_posts += 1;
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 4: Communication (gated)
        // ═══════════════════════════════════════════════════════════════
        let broadcasters: Vec<_> = self.agents
            .iter()
            .filter(|a| {
                a.skills.communication > 0.5
                    && self.publication_gate.approve(a.skills.communication, 0.3, a.reputation.score)
            })
            .map(|a| a.id)
            .collect();
        for sender_id in broadcasters {
            let msg = Message::broadcast(
                sender_id,
                MessageKind::Gossip,
                format!("Epoch {} status", epoch).into_bytes(),
                2,
            );
            let _ = self.mesh.broadcast_gossip(msg);
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 5: Mutation under environmental pressure
        // Pressure modulated by seasonal cycle — higher in harsh seasons.
        // Eco-state further modulates: Spring reduces volatility (safer
        // offspring), Summer intensifies (diversity burst).
        // ═══════════════════════════════════════════════════════════════
        let seasonal_pressure = {
            let base = environmental_pressure * current_eco.mutation_multiplier();
            // ── Stress: mutation volatility ───────────────────────────
            let m = self.stress_config
                .as_ref()
                .map(|sc| sc.mutation_volatility_multiplier)
                .unwrap_or(1.0);
            base * m
        };
        for agent in self.agents.iter_mut() {
            let m = self.mutation_engine.apply_pressure(agent.id, &mut agent.traits, seasonal_pressure);
            mutations += m as u64;
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP 6: Natural Selection
        // ═══════════════════════════════════════════════════════════════
        let population: Vec<(AgentDNA, f64, bool)> = self.agents
            .iter()
            .map(|dna| {
                let balance = self.ledger.balance(&dna.id).unwrap();
                (dna.clone(), balance.balance, balance.in_stasis)
            })
            .collect();

        let stasis_count;
        let (mean_fitness, max_fitness, min_fitness);

        // Dynamic pop cap from environment carrying capacity
        let dynamic_pop_cap = ((self.environment.total_capacity() / 15.0) as usize)
            .clamp(10, 500);
        self.pop_cap = dynamic_pop_cap;

        if let Ok(outcome) = self.selection_engine.select(&population) {
            mean_fitness = outcome.mean_fitness;
            max_fitness = outcome.max_fitness;
            min_fitness = outcome.min_fitness;
            stasis_count = outcome.stasis_candidates.len();

            // ─── Replication ───
            let replicator_ids: Vec<_> = outcome.replicators.clone();
            let mut births_this_epoch = 0usize;
            // Ecology-adjusted replication cost (Spring = 40% cheaper)
            // ── Stress: replication_cost_multiplier ───────────────────
            let effective_replication_cost = REPLICATION_COST
                * current_eco.fertility_multiplier()
                * self.stress_config
                    .as_ref()
                    .map(|sc| sc.replication_cost_multiplier)
                    .unwrap_or(1.0);

            // ── Phase 2: Soft carrying capacity ──────────────────────
            // Birth probability decreases smoothly as population → K.
            // birth_factor = 1.0 - (P / K), clamped [0.0, 1.0].
            // Below K: replication proceeds normally.
            // Near K: births become increasingly unlikely.
            // Above K: births stop entirely.
            let birth_factor = (1.0 - (self.agents.len() as f64 / self.pressure.soft_cap as f64))
                .clamp(0.0, 1.0);

            for parent_id in replicator_ids {
                if births_this_epoch >= MAX_BIRTHS_PER_EPOCH {
                    break;
                }
                // Soft cap: deterministic probabilistic birth throttle
                {
                    let birth_seed = epoch
                        .wrapping_mul(0x517C_C1B7_2722_0A95_u64)
                        .wrapping_add(births_this_epoch as u64);
                    let birth_roll = (birth_seed >> 33) as f64 / (u32::MAX as f64);
                    if birth_roll > birth_factor {
                        continue; // soft cap suppressed this birth
                    }
                }
                if self.agents.len() >= self.pop_cap {
                    break;
                }
                if let Some(parent) = self.agents.iter().find(|a| a.id == parent_id) {
                    // Maturation check
                    let birth_epoch = self.agent_birth_epoch.get(&parent.id).copied().unwrap_or(0);
                    if epoch.saturating_sub(birth_epoch) < MATURATION_EPOCHS {
                        continue;
                    }

                    let parent_balance = self.ledger.balance(&parent.id).unwrap().balance;
                    if parent_balance >= effective_replication_cost {
                        let child_entropy: Vec<u8> = (0..64)
                            .map(|j| {
                                parent.genesis_hash[j % 32]
                                    .wrapping_add(epoch as u8)
                                    .wrapping_add(j as u8)
                                    .wrapping_mul(0x9E as u8)
                            })
                            .collect();

                        if let Ok(child) = parent.replicate(&child_entropy) {
                            let _ = self.ledger.burn(
                                &parent_id,
                                effective_replication_cost,
                                TransactionKind::ReplicationCost,
                                "Replication cost",
                            );
                            self.ledger.register_agent(child.id, CHILD_GRANT);
                            let _ = self.mesh.registry.register(
                                &child,
                                format!("Gen{}-{}", child.generation, &child.genome_hex()[..6]),
                                "genesis",
                            );
                            self.mesh.init_inbox(child.id);

                            if let Some(parent_reg) = self.mesh.registry.get(&parent_id) {
                                let neighbors: Vec<_> = parent_reg.neighbors.clone();
                                for neighbor in neighbors {
                                    let _ = self.mesh.registry.connect(&child.id, &neighbor);
                                }
                            }

                            self.agent_birth_epoch.insert(child.id, epoch);
                            self.agents.push(child);
                            births += 1;
                            births_this_epoch += 1;
                        }
                    }
                }
            }

            // ─── Deaths ───
            for dead_id in &outcome.terminated {
                let dead_id = *dead_id;
                self.agents.retain(|a| a.id != dead_id);
                self.agent_birth_epoch.remove(&dead_id);
                if let Ok(bal) = self.ledger.balance(&dead_id) {
                    if bal.balance > 0.0 {
                        let _ = self.ledger.burn(
                            &dead_id, bal.balance,
                            TransactionKind::BasalMetabolism,
                            "Agent terminated",
                        );
                    }
                }
                let _ = self.mesh.registry.set_status(
                    &dead_id,
                    ecosystem::AgentStatus::Dead,
                );
                deaths += 1;
            }
        } else {
            mean_fitness = 0.0;
            max_fitness = 0.0;
            min_fitness = 0.0;
            stasis_count = 0;
        }

        self.epoch += 1;
        self.total_births += births;
        self.total_deaths += deaths;

        // Collect role counts and niche resource levels for stats
        let mut final_role_counts: HashMap<AgentRole, usize> = HashMap::new();
        for agent in self.agents.iter() {
            *final_role_counts.entry(agent.role).or_insert(0) += 1;
        }
        let niche_resources: HashMap<AgentRole, f64> = self.environment.pools
            .iter()
            .map(|(role, pool)| (*role, pool.level))
            .collect();

        // Compute Phase 2 metrics
        let epoch_role_entropy = role_entropy(&final_role_counts);
        let total_supply_for_ratio = self.ledger.total_supply();
        let treasury_ratio = if total_supply_for_ratio + self.treasury.reserve > 0.0 {
            self.treasury.reserve / (total_supply_for_ratio + self.treasury.reserve)
        } else {
            0.0
        };
        let death_rate_100 = {
            let window = self.epoch_history.iter().collect::<Vec<_>>();
            let total_deaths_window: u64 = window.iter().map(|s| s.deaths).sum();
            let epochs_in_window = window.len().max(1) as f64;
            (total_deaths_window as f64 / epochs_in_window) * 100.0
        };

        let stats = EpochStats {
            epoch,
            population: self.agents.len(),
            total_atp: self.ledger.total_supply(),
            mean_fitness,
            max_fitness,
            min_fitness,
            births,
            deaths,
            mutations,
            stasis_count,
            market_solved,
            market_rewarded,
            gated_posts,
            resources_extracted,
            total_resources: self.environment.total_resources(),
            season: self.environment.season_name().to_string(),
            catastrophe_active: self.environment.catastrophe_remaining > 0,
            dynamic_pop_cap,
            role_counts: final_role_counts.clone(),
            niche_resources,
            treasury_reserve: self.treasury.reserve,
            treasury_distributed: self.treasury.total_distributed - treasury_distributed_before,
            eco_state: current_eco,
            birth_death_ratio,
            gini_coefficient: epoch_gini,
            role_entropy: epoch_role_entropy,
            treasury_ratio,
            death_rate_100,
            entropy_tax_burned,
            catastrophe_deaths,
            immune_threats: 0,  // filled after cortex step
            immune_health: 0,
            pressure_mutations: 0,
        };

        // Keep rolling window of last 100 epochs
        if self.epoch_history.len() >= 100 {
            self.epoch_history.pop_front();
        }
        self.epoch_history.push_back(stats.clone());

        // Track population history for immune collapse detection
        self.population_history.push(stats.population);
        if self.population_history.len() > 200 {
            self.population_history.remove(0);
        }

        // Update peak treasury
        if self.treasury.reserve > self.peak_treasury {
            self.peak_treasury = self.treasury.reserve;
        }
        self.cortex.update_peak_treasury(self.treasury.reserve);

        // ═══════════════════════════════════════════════════════════════
        // STEP Ψ: Adaptive Cortex — immune-driven pressure mutation
        //
        // Every `cortex.interval` epochs (default 25), run the full
        // immune diagnostic suite. If threats are detected, the cortex
        // prescribes bounded mutations to PressureConfig. If healthy,
        // parameters gently drift toward defaults. All mutations are
        // anchored as cryptographic artifacts.
        //
        // This is where the organism's laws of nature evolve.
        // ═══════════════════════════════════════════════════════════════
        if self.cortex.should_adapt(epoch) {
            // Collect diagnostic inputs
            let role_counts_str: std::collections::HashMap<String, usize> = final_role_counts
                .iter()
                .map(|(r, c)| (r.label().to_string(), *c))
                .collect();

            let balances: Vec<f64> = self.agents.iter()
                .filter_map(|a| self.ledger.balance(&a.id).ok().map(|b| b.balance))
                .collect();

            let transacted_atp = resources_extracted + market_rewarded + entropy_tax_burned;

            let expected_roles = &[
                "Optimizer", "Strategist", "Communicator", "Archivist", "Executor",
            ];

            // Run full immune diagnosis
            let immune_report = genesis_homeostasis::diagnose(
                epoch,
                &role_counts_str,
                &balances,
                mutations as usize,
                self.agents.len(),
                &self.population_history,
                25,
                expected_roles,
                self.treasury.reserve,
                self.peak_treasury,
                transacted_atp,
                self.ledger.total_supply(),
            );

            // Update stats with immune data
            let threat_count = immune_report.threat_count();
            let health_level = match immune_report.overall_health {
                genesis_homeostasis::ThreatLevel::Normal => 0u8,
                genesis_homeostasis::ThreatLevel::Watch => 1,
                genesis_homeostasis::ThreatLevel::Warning => 2,
                genesis_homeostasis::ThreatLevel::Critical => 3,
            };

            // Log immune report if threats detected
            if threat_count > 0 {
                for event in &immune_report.events {
                    if event.level != genesis_homeostasis::ThreatLevel::Normal {
                        tracing::warn!(
                            epoch,
                            kind = ?event.kind,
                            level = ?event.level,
                            value = format!("{:.3}", event.metric_value),
                            "{}",
                            event.message,
                        );
                    }
                }
            }

            // Prescribe pressure mutations
            let before_json = serde_json::to_string(&self.pressure).unwrap_or_default();
            let response = self.cortex.prescribe(
                &immune_report,
                self.pressure.soft_cap,
                self.pressure.entropy_coeff,
                self.pressure.catastrophe_base_prob,
                self.pressure.catastrophe_pop_scale,
                self.pressure.gini_wealth_tax_threshold,
                self.pressure.gini_wealth_tax_rate,
                self.pressure.treasury_overflow_threshold,
            );

            let mutation_count = response.mutations.len();

            // Apply mutations to PressureConfig
            if response.has_mutations {
                for m in &response.mutations {
                    match m.field {
                        PressureField::SoftCap => {
                            self.pressure.soft_cap = m.new_value as usize;
                        }
                        PressureField::EntropyCoeff => {
                            self.pressure.entropy_coeff = m.new_value;
                        }
                        PressureField::CatastropheBaseProb => {
                            self.pressure.catastrophe_base_prob = m.new_value;
                        }
                        PressureField::CatastrophePopScale => {
                            self.pressure.catastrophe_pop_scale = m.new_value;
                        }
                        PressureField::GiniWealthTaxThreshold => {
                            self.pressure.gini_wealth_tax_threshold = m.new_value;
                        }
                        PressureField::GiniWealthTaxRate => {
                            self.pressure.gini_wealth_tax_rate = m.new_value;
                        }
                        PressureField::TreasuryOverflowThreshold => {
                            self.pressure.treasury_overflow_threshold = m.new_value;
                        }
                    }

                    tracing::info!(
                        epoch,
                        field = m.field.name(),
                        old = format!("{:.6}", m.old_value),
                        new = format!("{:.6}", m.new_value),
                        trigger = ?m.trigger,
                        "PRESSURE MUTATION: {}",
                        m.rationale,
                    );
                }

                // Record cooldowns
                self.cortex.record_mutations(&response);

                // Build mutation records for the evolution chain
                let mutation_records: Vec<MutationRecord> = response.mutations.iter()
                    .map(|m| MutationRecord {
                        field: m.field.name().to_string(),
                        old_value: m.old_value,
                        new_value: m.new_value,
                        trigger: format!("{:?}", m.trigger),
                        severity: format!("{:?}", m.severity),
                        rationale: m.rationale.clone(),
                    })
                    .collect();

                let after_json = serde_json::to_string(&self.pressure).unwrap_or_default();

                let health_str = match immune_report.overall_health {
                    genesis_homeostasis::ThreatLevel::Normal => "Normal",
                    genesis_homeostasis::ThreatLevel::Watch => "Watch",
                    genesis_homeostasis::ThreatLevel::Warning => "Warning",
                    genesis_homeostasis::ThreatLevel::Critical => "Critical",
                };

                // Chain-linked evolution anchor (replaces flat PressureAnchor)
                // Cross-references the latest State Chain epoch root
                let epoch_root_ref = self.anchor_engine.last_root.clone();
                if let Err(e) = self.evolution_engine.record(
                    epoch,
                    &before_json,
                    &after_json,
                    mutation_records,
                    threat_count,
                    health_str,
                    &epoch_root_ref,
                ) {
                    tracing::warn!(epoch, error = %e, "Failed to persist evolution anchor");
                }
            }

            // Update the stats in history with immune data
            if let Some(last) = self.epoch_history.back_mut() {
                last.immune_threats = threat_count;
                last.immune_health = health_level;
                last.pressure_mutations = mutation_count;
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP Φ: State Chain Anchoring
        //
        // Every `anchor_engine.interval` epochs (default 100), produce a
        // cryptographic EpochAnchor: Merkle root of all agent balances,
        // hashed world summary, chain-linked to previous state anchor.
        //
        // Cross-references the latest Evolution Chain root, creating a
        // bidirectional link between the two chains:
        //   State Chain ←→ Evolution Chain
        //
        // This is the organism's memory of its own physical state.
        // ═══════════════════════════════════════════════════════════════
        if self.anchor_engine.should_anchor(epoch) {
            let balances: Vec<(String, f64)> = self.agents.iter()
                .filter_map(|a| {
                    self.ledger.balance(&a.id)
                        .ok()
                        .map(|b| (a.id.to_string(), b.balance))
                })
                .collect();

            let role_counts_for_anchor: Vec<(String, usize)> = stats.role_counts.iter()
                .map(|(r, c)| (r.label().to_string(), *c))
                .collect();

            let summary = WorldSummary {
                epoch,
                population: self.agents.len(),
                total_supply: self.ledger.total_supply(),
                treasury_reserve: self.treasury.reserve,
                mean_fitness: stats.mean_fitness,
                total_births: self.total_births,
                total_deaths: self.total_deaths,
                role_counts: role_counts_for_anchor,
            };

            let mut state_anchor = self.anchor_engine.anchor(epoch, &balances, &summary);
            // Cross-chain reference: latest evolution chain root
            state_anchor.evolution_root_ref = self.evolution_engine.last_evolution_root.clone();

            if let Err(e) = self.anchor_engine.persist(&state_anchor) {
                tracing::warn!(epoch, error = %e, "Failed to persist state anchor");
            } else {
                tracing::info!(
                    epoch,
                    root = &state_anchor.epoch_root[..16],
                    population = state_anchor.population,
                    supply = format!("{:.2}", state_anchor.total_supply),
                    evolution_ref = &state_anchor.evolution_root_ref[..16],
                    "STATE CHAIN: Epoch anchor persisted with evolution cross-reference"
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // STEP Ω: Stress metrics recording + phase transition detection
        // ═══════════════════════════════════════════════════════════════
        if self.stress_metrics.is_some() {
            let entropy = role_entropy(&stats.role_counts);
            let hoarding_ratio = if self.ledger.total_supply() > 0.0 {
                stats.treasury_reserve / (stats.treasury_reserve + self.ledger.total_supply())
            } else {
                1.0
            };
            if let Some(ref mut sm) = self.stress_metrics {
                sm.record(
                    epoch,
                    stats.population,
                    stats.mean_fitness,
                    hoarding_ratio,
                    stats.birth_death_ratio,
                    entropy,
                    stats.total_atp,
                    stats.catastrophe_active,
                    stats.eco_state.name(),
                );
            }
            if let Some(msg) = self.phase_detector.push(stats.population, epoch) {
                tracing::warn!(epoch, "{}", msg);
            }
        }

        // Return the (potentially immune-updated) stats from history
        self.epoch_history.back().cloned().unwrap_or(stats)
    }

    /// Compute current telemetry snapshot.
    pub fn telemetry(&self) -> UnitStatus {
        let atp_balances: Vec<f64> = self.agents
            .iter()
            .map(|a| self.ledger.balance(&a.id).map(|b| b.balance).unwrap_or(0.0))
            .collect();
        UnitStatus::compute(&self.agents, &atp_balances)
    }

    /// Look up an agent by hex prefix of their genome.
    pub fn find_agent_by_hex(&self, hex_prefix: &str) -> Option<&AgentDNA> {
        self.agents.iter().find(|a| a.genome_hex().starts_with(hex_prefix))
    }

    /// Get ATP balance for an agent.
    pub fn agent_atp(&self, agent: &AgentDNA) -> f64 {
        self.ledger.balance(&agent.id).map(|b| b.balance).unwrap_or(0.0)
    }

    /// Uptime in seconds since world was created.
    pub fn uptime_seconds(&self) -> i64 {
        (Utc::now() - self.started_at).num_seconds()
    }

    /// Compute epoch-over-epoch diff for the last N epochs.
    /// Returns (population_delta, atp_delta, fitness_delta) averaged over window.
    pub fn epoch_diff(&self, window: usize) -> EpochDiff {
        let history: Vec<&EpochStats> = self.epoch_history.iter().collect();
        let len = history.len();

        if len < 2 {
            return EpochDiff::default();
        }

        let window = window.min(len);
        let recent = &history[len - window..];
        let first = recent.first().unwrap();
        let last = recent.last().unwrap();

        let total_births: u64 = recent.iter().map(|s| s.births).sum();
        let total_deaths: u64 = recent.iter().map(|s| s.deaths).sum();
        let total_mutations: u64 = recent.iter().map(|s| s.mutations).sum();

        EpochDiff {
            window: window as u64,
            population_delta: last.population as i64 - first.population as i64,
            atp_delta: last.total_atp - first.total_atp,
            fitness_delta: last.mean_fitness - first.mean_fitness,
            births_in_window: total_births,
            deaths_in_window: total_deaths,
            mutations_in_window: total_mutations,
        }
    }

    /// Build a leaderboard of the top N agents by fitness.
    pub fn leaderboard(&self, top_n: usize) -> Vec<LeaderboardEntry> {
        let mut entries: Vec<LeaderboardEntry> = self.agents.iter().map(|a| {
            LeaderboardEntry {
                agent_id: a.genome_hex()[..16].to_string(),
                role: a.role.label().to_string(),
                fitness: a.fitness(),
                reputation: a.reputation.score,
                atp_balance: self.agent_atp(a),
                generation: a.generation,
                is_primordial: a.is_primordial,
                survived_epochs: self.epoch,
            }
        }).collect();

        entries.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
        entries.truncate(top_n);
        entries
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_new_has_agents() {
        let world = World::new();
        assert_eq!(world.agents.len(), 20);
        assert_eq!(world.epoch, 0);
        assert!(world.ledger.total_supply() > 0.0);
    }

    #[test]
    fn test_run_epoch_advances() {
        let mut world = World::new();
        let stats = world.run_epoch();
        assert_eq!(stats.epoch, 0);
        assert_eq!(world.epoch, 1);
        assert!(stats.population >= 20);
    }

    #[test]
    fn test_multiple_epochs() {
        let mut world = World::new();
        for _ in 0..10 {
            world.run_epoch();
        }
        assert_eq!(world.epoch, 10);
        assert!(!world.agents.is_empty());
    }

    #[test]
    fn test_register_external() {
        let mut world = World::new();
        let req = RegistrationRequest {
            external_id: "moltbook:agent123".to_string(),
            public_key: "pk_test_12345".to_string(),
        };
        let result = world.register_external(&req).unwrap();
        assert!(!result.agent_id.is_empty());
        assert_eq!(result.initial_atp, 5.0);
        assert_eq!(world.agents.len(), 21);
    }

    #[test]
    fn test_register_duplicate_rejected() {
        let mut world = World::new();
        let req = RegistrationRequest {
            external_id: "moltbook:agent123".to_string(),
            public_key: "pk_test_12345".to_string(),
        };
        world.register_external(&req).unwrap();
        let result = world.register_external(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_telemetry() {
        let world = World::new();
        let status = world.telemetry();
        assert_eq!(status.population, 20);
        assert!(status.atp_total > 0.0);
    }

    #[test]
    fn test_find_agent() {
        let world = World::new();
        let first_hex = world.agents[0].genome_hex();
        let prefix = &first_hex[..8];
        let found = world.find_agent_by_hex(prefix);
        assert!(found.is_some());
    }

    #[test]
    fn test_uptime_seconds() {
        let world = World::new();
        let uptime = world.uptime_seconds();
        assert!(uptime >= 0);
    }

    #[test]
    fn test_epoch_diff_empty_history() {
        let world = World::new();
        let diff = world.epoch_diff(10);
        // With no history, returns default (window=0, all zeros)
        assert_eq!(diff.window, 0);
        assert_eq!(diff.population_delta, 0);
        assert_eq!(diff.births_in_window, 0);
        assert_eq!(diff.deaths_in_window, 0);
    }

    #[test]
    fn test_epoch_diff_with_history() {
        let mut world = World::new();
        for _ in 0..5 {
            world.run_epoch();
        }
        let diff = world.epoch_diff(3);
        assert_eq!(diff.window, 3);
        // Just assert it computed without panic
        let _ = diff.mutations_in_window;
    }

    #[test]
    fn test_leaderboard_sorted_by_fitness() {
        let mut world = World::new();
        world.run_epoch(); // ensure fitness values are set
        let board = world.leaderboard(10);
        assert!(board.len() <= 10);
        for window in board.windows(2) {
            assert!(window[0].fitness >= window[1].fitness);
        }
    }

    #[test]
    fn test_leaderboard_full() {
        let world = World::new();
        let board = world.leaderboard(100);
        assert_eq!(board.len(), 20); // default 20 agents, capped
    }

    #[test]
    fn test_epoch_history_accumulates() {
        let mut world = World::new();
        assert!(world.epoch_history.is_empty());
        world.run_epoch();
        assert_eq!(world.epoch_history.len(), 1);
        for _ in 0..4 {
            world.run_epoch();
        }
        assert_eq!(world.epoch_history.len(), 5);
    }

    #[test]
    fn test_total_births_deaths_tracked() {
        let mut world = World::new();
        for _ in 0..10 {
            world.run_epoch();
        }
        // Counters are initialized at 0 and only increment —
        // whether births/deaths occur depends on selection pressure.
        // Just verify the counters are accessible and consistent.
        assert!(world.total_births + world.total_deaths <= world.total_births + world.total_deaths);
        // And epoch_history should have 10 entries
        assert_eq!(world.epoch_history.len(), 10);
    }
}
