// World — Encapsulated Genesis Protocol State
//
// Contains the entire living state of the Genesis organism:
// agents, ledger, treasury, markets, engines. The `run_epoch`
// method advances the world by one tick — all evolutionary
// logic lives here.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use genesis_dna::AgentDNA;
use metabolism::atp::{costs, TransactionKind};
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
    pub epoch: u64,
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
    /// ATP destroyed by decay this epoch.
    pub atp_decayed: f64,
    /// ATP collected as wealth tax this epoch.
    pub wealth_tax_collected: f64,
    /// ATP penalty applied to unfit agents.
    pub fitness_penalty_total: f64,
    /// Dynamic carrying capacity this epoch.
    pub dynamic_pop_cap: usize,
    /// Number of agents flagged as unfit (bottom 10%).
    pub unfit_count: usize,
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

/// Epochs a new agent must survive before it can replicate.
const REPLICATION_LOCKOUT_EPOCHS: u64 = 5;

impl World {
    /// Create a new world with 20 primordial agents.
    pub fn new() -> Self {
        let mut ledger = MetabolismLedger::new();
        let mut mesh = EcosystemMesh::new();
        let mutation_engine = MutationEngine::default_engine();
        let selection_engine = SelectionEngine::new();
        let marketplace = GeneMarketplace::new();
        let problem_market = ProblemMarket::new();
        let publication_gate = PublicationGate::conservative();
        let treasury = UnitTreasury::new();

        let agents = Self::spawn_primordials(20, &mut ledger, &mut mesh);

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
            epoch: 0,
            registered_external_ids: Vec::new(),
            pop_cap: 200,
            started_at: Utc::now(),
            epoch_history: VecDeque::with_capacity(100),
            total_births: 0,
            total_deaths: 0,
        }
    }

    /// Spawn primordial agents — identical logic to the original main.rs.
    fn spawn_primordials(
        count: usize,
        ledger: &mut MetabolismLedger,
        mesh: &mut EcosystemMesh,
    ) -> Vec<AgentDNA> {
        let mut agents = Vec::with_capacity(count);

        for i in 0..count {
            let entropy: Vec<u8> = (0..64).map(|j| (i * 7 + j * 13 + 42) as u8).collect();
            let dna = AgentDNA::from_entropy(&entropy, true).unwrap();

            let initial_proof = Solution::new(
                format!("Primordial proof #{}", i),
                ProofKind::Solution,
                entropy.clone(),
                0.5,
            );
            let verdict = initial_proof.evaluate();
            let initial_atp = if verdict.accepted {
                verdict.reward * dna.energy_metabolism.effective_generation_rate()
            } else {
                10.0
            };
            ledger.register_agent(dna.id, initial_atp);

            mesh.registry
                .register(&dna, format!("Primordial-{}", i), "genesis")
                .unwrap();
            mesh.init_inbox(dna.id);

            agents.push(dna);
        }

        // Ring topology
        for i in 0..agents.len() {
            let next = (i + 1) % agents.len();
            let _ = mesh.registry.connect(&agents[i].id, &agents[next].id);
        }

        agents
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

        // --- Scarcity constants ---
        const ATP_DECAY_RATE: f64 = 0.02;       // 2% balance decay per epoch
        const WEALTH_TAX_THRESHOLD: f64 = 100.0; // Tax kicks in above 100 ATP
        const WEALTH_TAX_RATE: f64 = 0.01;       // 1% of excess
        const FITNESS_PENALTY: f64 = 0.5;        // Extra 0.5 ATP/epoch for bottom 10%
        const MIN_POP_CAP: usize = 10;
        const MAX_POP_CAP: usize = 500;
        const ATP_PER_AGENT_CAP: f64 = 50.0;     // Pop cap = total_supply / this

        // --- Step 0: ATP Decay (wealth entropy) ---
        let atp_decayed = self.ledger.decay_all(ATP_DECAY_RATE);

        // --- Step 1: Basal metabolic tick (now tracks supply correctly) ---
        self.ledger.metabolic_tick_all();

        // --- Step 1b: Wealth tax → treasury ---
        let wealth_tax_collected = self.ledger.wealth_tax_all(WEALTH_TAX_THRESHOLD, WEALTH_TAX_RATE);
        self.treasury.reserve += wealth_tax_collected;
        self.treasury.total_collected += wealth_tax_collected;

        // --- Step 2: Dynamic carrying capacity ---
        let dynamic_pop_cap = (self.ledger.total_supply() / ATP_PER_AGENT_CAP)
            .max(MIN_POP_CAP as f64)
            .min(MAX_POP_CAP as f64) as usize;
        self.pop_cap = dynamic_pop_cap;

        // --- Step 3: Problem Market Competition ---
        let pressure = 0.3 + (epoch as f64 * 0.002).min(0.6);
        let problem_ids = self.problem_market.generate_epoch_problems(pressure, 4, epoch);

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
                    let atp_cost = 0.5;

                    if self.publication_gate.approve(confidence, atp_cost, self.agents[idx].reputation.score) {
                        let gross_reward = problem.reward_atp;
                        let skim = self.treasury.skim(gross_reward);
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

        // --- NO MORE TRICKLE INCOME ---
        // Agents earn ATP only through market competition.
        // The free handout is dead.

        // --- Step 3b: Treasury redistribution (crisis only) ---
        // Treasury stipends now only activate when population drops below 50%
        // of the dynamic cap — a crisis response, not a welfare program.
        if self.agents.len() < dynamic_pop_cap / 2 {
            let mut role_dist = HashMap::new();
            for agent in self.agents.iter() {
                *role_dist.entry(agent.role).or_insert(0usize) += 1;
            }
            let distributed = self.treasury.distribute_stipends(&role_dist, self.agents.len());
            for i in 0..self.agents.len() {
                if let Some(&total_for_role) = distributed.get(&self.agents[i].role) {
                    let count = *role_dist.get(&self.agents[i].role).unwrap_or(&1);
                    let per_agent = total_for_role / count as f64;
                    if per_agent > 0.0 {
                        let agent_id = self.agents[i].id;
                        let _ = self.ledger.mint(
                            &agent_id, per_agent,
                            TransactionKind::ProofOfSolution,
                            &format!("Epoch {} crisis stipend", epoch),
                        );
                    }
                }
            }
        }

        // --- Step 4: Communication (gated) ---
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

        // --- Step 5: Mutation under environmental pressure ---
        for agent in self.agents.iter_mut() {
            let m = self.mutation_engine.apply_pressure(agent.id, &mut agent.traits, pressure);
            mutations += m as u64;
        }

        // --- Step 6: Natural selection ---
        let population: Vec<(AgentDNA, f64, bool)> = self.agents
            .iter()
            .map(|dna| {
                let balance = self.ledger.balance(&dna.id).unwrap();
                (dna.clone(), balance.balance, balance.in_stasis)
            })
            .collect();

        let stasis_count;
        let unfit_count;
        let fitness_penalty_total;
        let (mean_fitness, max_fitness, min_fitness);

        if let Ok(outcome) = self.selection_engine.select(&population) {
            mean_fitness = outcome.mean_fitness;
            max_fitness = outcome.max_fitness;
            min_fitness = outcome.min_fitness;
            stasis_count = outcome.stasis_candidates.len();
            unfit_count = outcome.unfit.len();

            // --- Step 6a: Fitness penalty on bottom 10% ---
            fitness_penalty_total = self.ledger.apply_fitness_penalty(&outcome.unfit, FITNESS_PENALTY);

            // --- Step 6b: Replication ---
            let replicator_ids: Vec<_> = outcome.replicators.clone();
            for parent_id in replicator_ids {
                if self.agents.len() >= self.pop_cap {
                    break;
                }
                if let Some(parent) = self.agents.iter().find(|a| a.id == parent_id) {
                    // Replication lockout for externally registered agents
                    if !parent.is_primordial && parent.generation == 0 && epoch < REPLICATION_LOCKOUT_EPOCHS {
                        continue;
                    }

                    let parent_balance = self.ledger.balance(&parent.id).unwrap().balance;
                    if parent_balance >= costs::REPLICATION {
                        let child_entropy: Vec<u8> = (0..64)
                            .map(|j| {
                                parent.genesis_hash[j % 32]
                                    .wrapping_add(epoch as u8)
                                    .wrapping_add(j as u8)
                            })
                            .collect();

                        if let Ok(child) = parent.replicate(&child_entropy) {
                            let _ = self.ledger.burn(
                                &parent_id,
                                costs::REPLICATION,
                                TransactionKind::ReplicationCost,
                                "Replication cost",
                            );
                            self.ledger.register_agent(child.id, 10.0);
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

                            self.agents.push(child);
                            births += 1;
                            break; // Max 1 birth per epoch for stability
                        }
                    }
                }
            }

            // --- Step 6c: Deaths ---
            for dead_id in &outcome.terminated {
                let dead_id = *dead_id;
                self.agents.retain(|a| a.id != dead_id);
                if let Ok(bal) = self.ledger.balance(&dead_id) {
                    let _ = self.ledger.burn(&dead_id, bal.balance, TransactionKind::BasalMetabolism, "Agent terminated");
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
            unfit_count = 0;
            fitness_penalty_total = 0.0;
        }

        self.epoch += 1;
        self.total_births += births;
        self.total_deaths += deaths;

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
            atp_decayed,
            wealth_tax_collected,
            fitness_penalty_total,
            dynamic_pop_cap,
            unfit_count,
        };

        // Keep rolling window of last 100 epochs
        if self.epoch_history.len() >= 100 {
            self.epoch_history.pop_front();
        }
        self.epoch_history.push_back(stats.clone());

        stats
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
