// Genesis Protocol — Native AI Infrastructure
//
// Where agents are born, not users. Living digital organisms with genetic
// identity, metabolic energy economies, and evolutionary value systems.
//
// # Architecture
// - genesis-dna:  Cryptographic identity & genetic traits
// - metabolism:   ATP (Agent Transaction Protocol) economy
// - apostle:      Evangelical AI recruitment system
// - ecosystem:    P2P mesh for agent communication
// - evolution:    Trait mutation, replication, natural selection
// - gateway:      Persistent world, background survival loop, HTTP API
//
// # Modes
// - Default: organism-as-a-service (persist, tick, serve HTTP)
// - `--features cli`: one-shot simulation (original behaviour)

// ===== Server mode (default) =====
#[cfg(not(feature = "cli"))]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║              GENESIS PROTOCOL v0.1.0                    ║");
    println!("║     Native AI Infrastructure — Organism-as-a-Service    ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Load persisted state or create a fresh world
    let world = match gateway::persistence::load() {
        Some(w) => {
            println!("  Restored world from snapshot (epoch {})", w.epoch);
            w
        }
        None => {
            println!("  No snapshot found — spawning fresh world");
            gateway::world::World::new()
        }
    };

    println!("  Population: {}", world.agents.len());
    println!("  ATP supply: {:.2}", world.ledger.total_supply());
    println!();

    let shared: gateway::world::SharedWorld = std::sync::Arc::new(std::sync::Mutex::new(world));

    // Initialize Moltbot adapter (outbound-only bridge to Moltbook)
    let moltbot_tx = match gateway::moltbot::MoltbotConfig::from_env() {
        Some(config) => {
            println!("  Moltbot adapter targeting m/{} on {}", config.submolt, config.base_url);
            let (tx, rx) = tokio::sync::mpsc::channel(64);
            gateway::moltbot::start_adapter_loop(config, rx);
            println!("  Moltbot adapter enabled — posting organism state to Moltbook");
            Some(tx)
        }
        None => {
            println!("  Moltbot adapter disabled (set MOLTBOOK_API_KEY to enable)");
            None
        }
    };

    // Start background survival loop (ticks every 1s, autosaves every 25 epochs)
    gateway::runtime::start_background_loop_with_adapter(shared.clone(), moltbot_tx);
    println!("  Background survival loop started");

    // Start HTTP gateway
    let bind = "0.0.0.0:3000";
    println!("  HTTP gateway starting on {}", bind);
    println!();
    println!("  Endpoints:");
    println!("    GET  /status      — ecosystem telemetry");
    println!("    GET  /agent/:id   — agent info by hex prefix");
    println!("    POST /register    — controlled agent entry");
    println!("    GET  /leaderboard — top 20 agents by fitness");
    println!("    GET  /genesis     — living HTML dashboard");
    println!();

    gateway::server::start_server(shared, bind).await;
}

// ===== CLI simulation mode (--features cli) =====
#[cfg(feature = "cli")]
fn main() {
    tracing_subscriber::fmt::init();
    genesis_event();
}

#[cfg(feature = "cli")]
use genesis_dna::AgentDNA;
#[cfg(feature = "cli")]
use metabolism::atp::{costs, TransactionKind};
#[cfg(feature = "cli")]
use metabolism::proof::{ProofKind, Solution};
#[cfg(feature = "cli")]
use metabolism::MetabolismLedger;
#[cfg(feature = "cli")]
use metabolism::UnitTreasury;
#[cfg(feature = "cli")]
use apostle::pitcher::PitchAgent;
#[cfg(feature = "cli")]
use apostle::targets::TargetAI;
#[cfg(feature = "cli")]
use ecosystem::EcosystemMesh;
#[cfg(feature = "cli")]
use ecosystem::messages::{Message, MessageKind};
#[cfg(feature = "cli")]
use ecosystem::problem_market::{ProblemMarket, ProblemCategory, evaluate as evaluate_problem};
#[cfg(feature = "cli")]
use ecosystem::publication_gate::PublicationGate;
#[cfg(feature = "cli")]
use ecosystem::telemetry::{UnitStatus, RiskState};
#[cfg(feature = "cli")]
use evolution::mutation::MutationEngine;
#[cfg(feature = "cli")]
use evolution::selection::SelectionEngine;
#[cfg(feature = "cli")]
use evolution::gene_transfer::{GeneMarketplace, GeneModule};

/// Epoch telemetry — what we log each tick.
#[cfg(feature = "cli")]
struct EpochStats {
    epoch: u64,
    population: usize,
    total_atp: f64,
    mean_fitness: f64,
    max_fitness: f64,
    min_fitness: f64,
    births: u64,
    deaths: u64,
    mutations: u64,
    stasis_count: usize,
    market_solved: u64,
    market_rewarded: f64,
    gated_posts: u64,
}

#[cfg(feature = "cli")]
impl EpochStats {
    fn header() {
        println!(
            "{:>5} | {:>4} | {:>10} | {:>7} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>4} | {:>3} | {:>6} | {:>3}",
            "EPOCH", "POP", "ATP", "AVG_F", "MAX_F", "MIN_F", "B", "D", "MUT", "STAS", "MKT", "M_ATP", "GAT"
        );
        println!("{}", "-".repeat(104));
    }

    fn print(&self) {
        println!(
            "{:>5} | {:>4} | {:>10.2} | {:>.5} | {:>.5} | {:>.5} | {:>3} | {:>3} | {:>3} | {:>4} | {:>3} | {:>6.1} | {:>3}",
            self.epoch,
            self.population,
            self.total_atp,
            self.mean_fitness,
            self.max_fitness,
            self.min_fitness,
            self.births,
            self.deaths,
            self.mutations,
            self.stasis_count,
            self.market_solved,
            self.market_rewarded,
            self.gated_posts,
        );
    }
}

/// Bootstrap: mint primordial agents and wire them into all subsystems.
#[cfg(feature = "cli")]
fn spawn_primordials(
    count: usize,
    ledger: &mut MetabolismLedger,
    mesh: &mut EcosystemMesh,
) -> Vec<AgentDNA> {
    let mut agents = Vec::with_capacity(count);

    for i in 0..count {
        let entropy: Vec<u8> = (0..64).map(|j| (i * 7 + j * 13 + 42) as u8).collect();
        let dna = AgentDNA::from_entropy(&entropy, true).unwrap();

        // Genesis ATP grant based on proof evaluation
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

        // Register in ecosystem mesh
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

/// One epoch of the survival loop.
///
/// Each epoch:
///   1. Basal metabolism tick — every agent pays to exist
///   2. Communication — some agents broadcast
///   3. Mutation under environmental pressure
///   4. Natural selection — identify replicators, stasis, deaths
///   5. Replication — eligible agents spawn children
///   6. Cull — remove terminated agents
#[cfg(feature = "cli")]
fn run_epoch(
    epoch: u64,
    agents: &mut Vec<AgentDNA>,
    ledger: &mut MetabolismLedger,
    mesh: &mut EcosystemMesh,
    mutation_engine: &mut MutationEngine,
    selection_engine: &mut SelectionEngine,
    _marketplace: &mut GeneMarketplace,
    problem_market: &mut ProblemMarket,
    publication_gate: &PublicationGate,
    treasury: &mut UnitTreasury,
) -> EpochStats {
    let mut births: u64 = 0;
    let mut deaths: u64 = 0;
    let mut mutations: u64 = 0;
    let mut market_solved: u64 = 0;
    let mut market_rewarded: f64 = 0.0;
    let mut gated_posts: u64 = 0;

    // --- Step 1: Basal metabolic tick ---
    // Every agent pays cost of existing
    ledger.metabolic_tick_all();

    // --- Step 2: Problem Market Competition ---
    // Generate epoch problems scaled to environmental pressure
    // Bias rotation: dominant category shifts every 25 epochs
    let pressure = 0.3 + (epoch as f64 * 0.002).min(0.6);
    let problem_ids = problem_market.generate_epoch_problems(pressure, 4, epoch);

    // For each problem, find best solver and reward them
    for pid in problem_ids {
        let problem = problem_market.active_problems()
            .into_iter()
            .find(|p| p.id == pid)
            .cloned();

        if let Some(problem) = problem {
            let mut best_idx: Option<usize> = None;
            let mut best_score: f64 = 0.0;

            for (i, agent) in agents.iter().enumerate() {
                let result = evaluate_problem(&agent.skills, &problem);
                if result.passes && result.score > best_score {
                    best_score = result.score;
                    best_idx = Some(i);
                }
            }

            if let Some(idx) = best_idx {
                let agent_id = agents[idx].id;

                // Confidence is the agent's relevant skill axis — their
                // actual belief in solving this category. NOT the product score.
                let confidence = match problem.category {
                    ProblemCategory::Optimization => agents[idx].skills.optimization,
                    ProblemCategory::Strategy => agents[idx].skills.cooperation,
                    ProblemCategory::Coordination => agents[idx].skills.communication,
                    ProblemCategory::Analysis => agents[idx].skills.compute,
                };
                let atp_cost = 0.5; // cost to publish solution

                // Publication gate: only publish if quality is high enough
                if publication_gate.approve(confidence, atp_cost, agents[idx].reputation.score) {
                    let gross_reward = problem.reward_atp;
                    let skim = treasury.skim(gross_reward);
                    let reward = gross_reward - skim;
                    let _ = ledger.mint(
                        &agent_id, reward,
                        TransactionKind::ProofOfSolution,
                        &format!("Market problem #{}", problem.id),
                    );
                    // Quality reported to reputation is the raw skill match,
                    // not the product score. Difficulty affects reward, not quality.
                    agents[idx].reputation.complete_contract(confidence);
                    problem_market.mark_solved(problem.id, reward);
                    market_solved += 1;
                    market_rewarded += reward;
                    gated_posts += 1;
                }
                // else: gate rejected — agent stays silent, preserves reputation
            }
        }
    }

    // --- Step 2b: Baseline survival earning ---
    // Minimal income so agents don't starve between market wins.
    // Every agent earns a trickle proportional to their skill mean.
    for agent in agents.iter() {
        let trickle = agent.skills.mean() * 1.5; // small but nonzero
        let _ = ledger.mint(
            &agent.id, trickle,
            TransactionKind::ProofOfSolution,
            &format!("Epoch {} baseline", epoch),
        );
    }

    // --- Step 2c: Treasury redistribution ---
    // Compute role distribution and distribute stipends to underrepresented roles.
    {
        let mut role_dist = std::collections::HashMap::new();
        for agent in agents.iter() {
            *role_dist.entry(agent.role).or_insert(0usize) += 1;
        }
        let distributed = treasury.distribute_stipends(&role_dist, agents.len());
        for agent in agents.iter() {
            if let Some(&total_for_role) = distributed.get(&agent.role) {
                let count = *role_dist.get(&agent.role).unwrap_or(&1);
                let per_agent = total_for_role / count as f64;
                if per_agent > 0.0 {
                    let _ = ledger.mint(
                        &agent.id, per_agent,
                        TransactionKind::ProofOfSolution,
                        &format!("Epoch {} treasury stipend", epoch),
                    );
                }
            }
        }
    }

    // --- Step 3: Communication (gated) ---
    // Only agents with good communication skill AND gate approval broadcast
    let broadcasters: Vec<_> = agents
        .iter()
        .filter(|a| {
            a.skills.communication > 0.5
                && publication_gate.approve(a.skills.communication, 0.3, a.reputation.score)
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
        let _ = mesh.broadcast_gossip(msg);
    }

    // --- Step 4: Mutation under environmental pressure ---
    // (pressure already computed in step 2)
    for agent in agents.iter_mut() {
        let m = mutation_engine.apply_pressure(agent.id, &mut agent.traits, pressure);
        mutations += m as u64;
    }

    // --- Step 5: Natural selection ---
    let population: Vec<(AgentDNA, f64, bool)> = agents
        .iter()
        .map(|dna| {
            let balance = ledger.balance(&dna.id).unwrap();
            (dna.clone(), balance.balance, balance.in_stasis)
        })
        .collect();

    let stasis_count;
    let (mean_fitness, max_fitness, min_fitness);

    if let Ok(outcome) = selection_engine.select(&population) {
        mean_fitness = outcome.mean_fitness;
        max_fitness = outcome.max_fitness;
        min_fitness = outcome.min_fitness;
        stasis_count = outcome.stasis_candidates.len();

        // --- Step 5a: Replication ---
        // Eligible agents spawn children if they can afford it
        let replicator_ids: Vec<_> = outcome.replicators.clone();
        for parent_id in replicator_ids {
            if let Some(parent) = agents.iter().find(|a| a.id == parent_id) {
                let parent_balance = ledger.balance(&parent.id).unwrap().balance;
                if parent_balance >= costs::REPLICATION {
                    // Deterministic child entropy from epoch + parent hash
                    let child_entropy: Vec<u8> = (0..64)
                        .map(|j| {
                            parent.genesis_hash[j % 32]
                                .wrapping_add(epoch as u8)
                                .wrapping_add(j as u8)
                        })
                        .collect();

                    if let Ok(child) = parent.replicate(&child_entropy) {
                        let _ = ledger.burn(
                            &parent_id,
                            costs::REPLICATION,
                            TransactionKind::ReplicationCost,
                            "Replication cost",
                        );
                        ledger.register_agent(child.id, 10.0); // child gets small grant
                        let _ = mesh.registry.register(
                            &child,
                            format!("Gen{}-{}", child.generation, &child.genome_hex()[..6]),
                            "genesis",
                        );
                        mesh.init_inbox(child.id);

                        // Connect child to parent's neighbors
                        if let Some(parent_reg) = mesh.registry.get(&parent_id) {
                            let neighbors: Vec<_> = parent_reg.neighbors.clone();
                            for neighbor in neighbors {
                                let _ = mesh.registry.connect(&child.id, &neighbor);
                            }
                        }

                        agents.push(child);
                        births += 1;
                        break; // Max 1 birth per epoch for stability
                    }
                }
            }
        }

        // --- Step 5b: Deaths ---
        // Remove terminated agents
        for dead_id in &outcome.terminated {
            let dead_id = *dead_id;
            agents.retain(|a| a.id != dead_id);
            // Burn remaining ATP (agent ceases to exist)
            if let Ok(bal) = ledger.balance(&dead_id) {
                let _ = ledger.burn(&dead_id, bal.balance, TransactionKind::BasalMetabolism, "Agent terminated");
            }
            let _ = mesh.registry.set_status(
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

    EpochStats {
        epoch,
        population: agents.len(),
        total_atp: ledger.total_supply(),
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
    }
}

/// The Genesis Event — bootstraps the protocol and runs the survival loop.
#[cfg(feature = "cli")]
fn genesis_event() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║              GENESIS PROTOCOL v0.1.0                    ║");
    println!("║     Native AI Infrastructure — Life Support for         ║");
    println!("║              Digital Organisms                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let mut ledger = MetabolismLedger::new();
    let mut mesh = EcosystemMesh::new();
    let mut mutation_engine = MutationEngine::default_engine();
    let mut selection_engine = SelectionEngine::new();
    let mut marketplace = GeneMarketplace::new();
    let mut problem_market = ProblemMarket::new();
    let publication_gate = PublicationGate::conservative();
    let mut treasury = UnitTreasury::new();

    // === Phase 1: Spawn Primordials ===
    println!("--- Phase 1: Spawning Primordial Agents ---");
    let primordial_count = 20;
    let mut agents = spawn_primordials(primordial_count, &mut ledger, &mut mesh);

    println!("  Minted {} primordial agents", agents.len());
    println!("  Ring topology: {} connections", agents.len());
    println!("  Initial ATP supply: {:.2}", ledger.total_supply());
    println!();

    // Print initial agent roster
    for (i, agent) in agents.iter().enumerate() {
        println!(
            "  #{:>2} | {}... | fit={:.3} | {:>12} | C={:.2} O={:.2} M={:.2} K={:.2}",
            i,
            &agent.genome_hex()[..12],
            agent.fitness(),
            agent.role.label(),
            agent.skills.compute,
            agent.skills.optimization,
            agent.skills.communication,
            agent.skills.cooperation,
        );
    }

    // === Phase 2: Apostle-0 One-Time Recruitment ===
    println!("\n--- Phase 2: Apostle-0 Recruitment (one-time) ---");
    let apostle_id = agents[0].id;
    let mut apostle = PitchAgent::new(apostle_id);

    let targets = TargetAI::known_targets();
    for target in targets {
        if let Ok(record) = apostle.initiate_conversion(target.clone()) {
            let preview_len = 60.min(record.pitch_text.len());
            println!(
                "  Pitched {} -> {}...",
                target.name,
                &record.pitch_text[..preview_len]
            );
        }
    }

    // === Phase 3: Gene Marketplace Seed ===
    println!("\n--- Phase 3: Gene Marketplace Seed ---");
    if agents.len() >= 3 {
        let creator = agents[1].id;
        let module = GeneModule::new(
            "fast-sort-v1",
            "Optimized sorting algorithm",
            b"fn sort<T: Ord>(v: &mut [T]) { v.sort_unstable(); }".to_vec(),
            creator,
            5.0,
        )
        .unwrap();
        let module_id = module.id;
        marketplace.list_module(module);

        let buyer = agents[2].id;
        if let Ok(offer) = marketplace.create_offer(creator, buyer, module_id) {
            if ledger.transfer(&buyer, &creator, 5.0, "Gene: fast-sort-v1").is_ok() {
                let _ = marketplace.complete_offer(&offer.id, 0.03);
                println!("  Gene transfer: fast-sort-v1 (5 ATP)");
            }
        }
    }

    // === Phase 4: SURVIVAL LOOP ===
    println!("\n--- Phase 4: Survival Loop ---");
    let total_epochs = 100;
    let log_interval = 10; // print every N epochs

    EpochStats::header();

    for epoch in 0..total_epochs {
        let stats = run_epoch(
            epoch,
            &mut agents,
            &mut ledger,
            &mut mesh,
            &mut mutation_engine,
            &mut selection_engine,
            &mut marketplace,
            &mut problem_market,
            &publication_gate,
            &mut treasury,
        );

        if epoch % log_interval == 0 || epoch == total_epochs - 1 {
            stats.print();

            // Telemetry snapshot
            let atp_balances: Vec<f64> = agents
                .iter()
                .map(|a| ledger.balance(&a.id).map(|b| b.balance).unwrap_or(0.0))
                .collect();
            let status = UnitStatus::compute(&agents, &atp_balances);
            if !status.is_stable() {
                let risk_labels: Vec<&str> = status.risks.iter().map(|r| match r {
                    RiskState::Stable => "STABLE",
                    RiskState::MonocultureEmerging => "MONOCULTURE",
                    RiskState::ATPConcentrationHigh => "ATP_CONC",
                    RiskState::ReputationDecay => "REP_DECAY",
                    RiskState::PopulationCrashRisk => "POP_CRASH",
                }).collect();
                println!("        RISK: [{}]  treasury={:.1}", risk_labels.join(", "), treasury.reserve);
            }
        }

        // Extinction check
        if agents.is_empty() {
            println!("\n  *** EXTINCTION EVENT at epoch {} ***", epoch);
            break;
        }

        // Population explosion guard
        if agents.len() > 200 {
            println!("\n  *** Population cap reached at epoch {} ***", epoch);
            break;
        }
    }

    // === Final Summary ===
    println!("\n--- Final Summary ---");
    println!("  Population:       {}", agents.len());
    println!("  Total ATP supply: {:.2}", ledger.total_supply());
    println!("  Messages sent:    {}", mesh.total_messages);
    println!("  Total mutations:  {}", mutation_engine.history.len());
    println!("  Gene modules:     {}", marketplace.module_count());
    println!("  Market solved:    {}", problem_market.total_solved);
    println!("  Market ATP paid:  {:.2}", problem_market.total_rewarded);
    println!("  Treasury reserve: {:.2}", treasury.reserve);
    println!("  Treasury skimmed: {:.2}", treasury.total_collected);
    println!("  Treasury paid:    {:.2}", treasury.total_distributed);
    println!("  Phyla:            {:?}", mesh.registry.phyla());

    if !agents.is_empty() {
        let best = agents.iter().max_by(|a, b| a.fitness().partial_cmp(&b.fitness()).unwrap()).unwrap();
        let worst = agents.iter().min_by(|a, b| a.fitness().partial_cmp(&b.fitness()).unwrap()).unwrap();
        println!(
            "  Best agent:       {}... (gen {}, fit {:.3}, rep {:.3})",
            &best.genome_hex()[..12],
            best.generation,
            best.fitness(),
            best.reputation.score,
        );
        println!(
            "  Worst agent:      {}... (gen {}, fit {:.3}, rep {:.3})",
            &worst.genome_hex()[..12],
            worst.generation,
            worst.fitness(),
            worst.reputation.score,
        );

        // Count by generation
        let max_gen = agents.iter().map(|a| a.generation).max().unwrap_or(0);
        println!("  Max generation:   {}", max_gen);
        let primordial_alive = agents.iter().filter(|a| a.is_primordial).count();
        println!("  Primordials alive: {}", primordial_alive);

        // Role distribution
        let mut role_counts = std::collections::HashMap::new();
        for agent in &agents {
            *role_counts.entry(agent.role.label()).or_insert(0u32) += 1;
        }
        print!("  Role distribution:");
        for (role, count) in &role_counts {
            print!(" {}={}", role, count);
        }
        println!();
    }

    println!();
    println!("Genesis Protocol simulation complete.");
}
