# Genesis Protocol: A Lotka–Volterra Inspired Software Organism with Cryptographic Provenance

**Kevan Burns**  
Independent Researcher · FTH Trading, Norcross, GA  
kevan.burns@fthtrading.com  
ORCID: [0009-0008-8425-939X](https://orcid.org/0009-0008-8425-939X)

**Version 1.0 — February 21, 2026** | [GitHub Repository](https://github.com/FTHTrading/AI) | [Live Documentation](https://fthtrading.github.io/AI/)

DOI: [10.5281/zenodo.18729652](https://doi.org/10.5281/zenodo.18729652)

© 2026 Kevan Burns. Licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Source code licensed under MIT.

---

## Abstract

We present Genesis Protocol, a self-sustaining digital organism implemented as a six-crate Rust workspace in which autonomous agents are born with SHA-256–derived genomes, compete for resources in per-niche pools governed by logistic regeneration, face seasonal environmental oscillation and stochastic catastrophe events, and are subject to natural selection pressure. The ecological model draws on Lotka–Volterra resource competition theory, implementing density-dependent foraging, cross-niche interference, and proportional extraction — replacing the winner-take-all dynamics common in multi-agent simulations with biologically plausible population dynamics. The system formalizes its operational requirements as the Software Organism Protocol (SOP-1) and establishes source code integrity through per-crate Merkle trees whose edition root can be anchored on a production blockchain. We validate the architecture through 143 automated tests across 39 source files totaling 10,015 lines of Rust, and demonstrate that the provenance model developed for literary publishing (Burns, 2026) generalizes to executable software systems without modification to its cryptographic infrastructure.

**Keywords:** artificial life, evolutionary computation, Lotka–Volterra, multi-agent systems, resource competition, ecological simulation, software organism, Rust, cryptographic provenance, Merkle trees

---

## Contents

1. Introduction
2. Related Work
3. System Architecture
4. Ecological Model
5. Energy Economics
6. Selection and Reproduction
7. Provenance Model
8. Merkle Tree System
9. Protocol Specification (SOP-1)
10. Evaluation
11. Discussion
12. Conclusion  
References  
Appendix A: System Invariants  
Appendix B: Crate Hash Table  
Appendix C: How to Cite This Work

---

## 1. Introduction

Artificial life research has a long history of constructing software systems that exhibit emergent behavior through the interaction of simple agents operating under environmental pressure [1, 2]. However, the majority of such systems share two structural weaknesses: their ecological dynamics are unrealistic (often reducing resource competition to winner-take-all allocation), and their provenance — the ability to independently verify that a particular version of the system produced particular results — is unaddressed.

We present Genesis Protocol, a system that addresses both concerns simultaneously. The organism is a Rust workspace of six library crates and one binary that runs as an HTTP server with a background survival loop. Agents are spawned with genomes derived from SHA-256 hashes of unique seed strings, ensuring genuine genetic diversity from the primordial generation. They compete for resources in per-niche pools that regenerate via logistic growth, experience seasonal oscillation and stochastic perturbation, and face natural selection pressure that culls unfit agents while permitting viable ones to replicate.

This paper makes three contributions:

1. **A biologically grounded ecological model** for multi-agent resource competition, implementing Lotka–Volterra dynamics with density-dependent foraging, cross-niche interference coefficients, and proportional extraction that produces realistic population oscillation, resource depletion, and recovery cycles.

2. **The Software Organism Protocol (SOP-1)**, a formal specification extending the Literary Protocol Standard (LPS-1) from literary publishing [3] to executable software systems, defining the ecological, metabolic, selection, and provenance requirements that any conforming implementation must satisfy.

3. **A complete provenance infrastructure** using per-crate Merkle trees, SHA-256 manifests, and on-chain anchoring capability that enables any party to independently verify the integrity of the source code without trusting the author, hosting platform, or any intermediary.

The system is deployed as a public repository with a live documentation site and is designed to demonstrate that rigorous ecological modeling and cryptographic provenance are complementary — not competing — design goals for artificial life systems.

---

## 2. Related Work

### 2.1 Artificial Life and Multi-Agent Systems

The field of artificial life, formalized by Langton [1], explores emergent behavior in computational systems. Tierra [2] demonstrated that digital organisms competing for CPU cycles could exhibit evolutionary dynamics analogous to biological systems. More recent work on multi-agent systems [4] has explored cooperation, competition, and emergent social behavior, but ecological realism — particularly in resource dynamics — has often been sacrificed for computational convenience.

### 2.2 Lotka–Volterra Resource Competition

The Lotka–Volterra equations [5, 6] provide a mathematical framework for modeling interspecific competition:

$$\frac{dN_i}{dt} = r_i N_i \left(1 - \frac{N_i + \sum_{j \neq i} \alpha_{ij} N_j}{K_i}\right)$$

where $N_i$ is the population of species $i$, $r_i$ is the intrinsic growth rate, $K_i$ is the carrying capacity, and $\alpha_{ij}$ is the competition coefficient measuring the effect of species $j$ on species $i$. This framework has been extensively validated in ecological research [7] but has seen limited application in software agent systems, where resource allocation typically follows economic auction models rather than ecological dynamics.

### 2.3 Cryptographic Provenance

Content integrity verification using Merkle trees [8] is well established in distributed systems — Git uses Merkle DAGs for repository integrity [9], and IPFS uses them for content-addressed storage [10]. Burns (2026) [3] applied Merkle tree hashing to literary manuscripts, establishing a five-layer provenance stack anchored on a production blockchain. Our work extends this approach from static content (manuscripts) to dynamic executable systems (software organisms), demonstrating that the same cryptographic infrastructure — SHA-256 hashing, Merkle tree construction, on-chain anchoring — can verify source code integrity at the crate level.

### 2.4 Software Verification and Reproducibility

Reproducible builds [11] aim to ensure that identical source code produces byte-identical binaries. Our provenance model operates at a complementary level: rather than verifying build output determinism, we verify source code integrity — ensuring that the 39 source files constituting the organism have not been modified since the edition root was computed. This source-level verification is independent of compiler version, optimization flags, or platform.

---

## 3. System Architecture

### 3.1 Design Principles

The architecture is governed by four principles:

**Biological plausibility.** Ecological dynamics must be grounded in established theory (Lotka–Volterra competition, logistic growth, density dependence) rather than ad hoc heuristics.

**Economic coherence.** Energy (ATP) is a conserved quantity subject to thermodynamic constraints. No agent can create energy from nothing; all income derives from resource extraction or market rewards.

**Cryptographic verifiability.** Every source file has a SHA-256 hash. Per-crate Merkle roots compose into an edition root that can be independently recomputed by any party with access to the source.

**Formal specification.** Operational requirements are codified as the Software Organism Protocol (SOP-1), enabling independent implementations that conform to the same behavioral guarantees.

### 3.2 Crate Architecture

The organism is structured as a Rust workspace with six library crates, each responsible for a distinct functional layer:

| Layer | Crate | Responsibility | Files | Lines |
|-------|-------|---------------|-------|-------|
| 1 | `genesis-dna` | Genome, identity, traits, lineage, roles, skills | 7 | 918 |
| 2 | `metabolism` | ATP accounting, ledger, treasury, proof-of-work | 6 | 1,019 |
| 3 | `ecosystem` | Communication mesh, problem markets, telemetry, registry | 8 | 1,022 |
| 4 | `evolution` | Mutation engine, selection pressure, gene transfer | 5 | 638 |
| 5 | `gateway` | World engine, HTTP server, persistence, security | 7 | 5,172 |
| 6 | `apostle` | Outbound intelligence, conversion, targeting | 5 | 535 |
| — | `tests` | Load simulation integration tests | 1 | 511 |
| **Total** | | | **39** | **10,015** |

Table 1. Crate decomposition with file and line counts.

The gateway crate is intentionally the largest, containing the world engine (`world.rs`, 1,078 lines) which orchestrates the six-step epoch cycle, the HTTP server (`server.rs`, 1,167 lines) providing the REST API and SSE endpoints, and the security shield (`shield.rs`, 532 lines) implementing rate limiting and AI verification challenges.

### 3.3 Runtime Model

The organism runs as an Axum HTTP server with a background survival loop. Each epoch executes a deterministic six-step cycle:

```
┌─────────────────────────────────────────────────────────────┐
│                    EPOCH CYCLE (6 steps)                     │
├─────────────────────────────────────────────────────────────┤
│ Step 0: Environment tick (regenerate, seasons, events)       │
│ Step 1: Resource extraction (proportional foraging per niche)│
│ Step 2: Basal metabolism (cost of living)                    │
│ Step 3: Problem market (supplementary income)                │
│ Step 4: Communication (gated broadcasting)                   │
│ Step 5: Mutation (environmentally modulated)                 │
│ Step 6: Natural selection (fitness culling + replication)     │
└─────────────────────────────────────────────────────────────┘
```

Figure 1. Six-step epoch cycle.

State is persisted to JSON after each epoch, enabling the organism to survive process restarts. The HTTP server exposes endpoints for observing the organism's vital signs, population dynamics, and evolutionary history via both REST and Server-Sent Events (SSE).

---

## 4. Ecological Model

The ecological model is the core contribution of this work. It replaces the flat, winner-take-all resource allocation common in multi-agent simulations with a Lotka–Volterra inspired system exhibiting five key properties: logistic resource regeneration, seasonal environmental oscillation, stochastic perturbation, density-dependent foraging, and cross-niche competition.

### 4.1 Resource Pools

Each of the five agent roles (Optimizer, Strategist, Communicator, Archivist, Executor) occupies an ecological niche with an independent resource pool. Pools regenerate via logistic growth:

$$R(t+1) = R(t) + r \cdot R(t) \cdot \left(1 - \frac{R(t)}{K}\right)$$

where $R(t)$ is the current resource level, $r$ is the per-epoch regeneration rate (default: 0.12), and $K$ is the carrying capacity. This produces the characteristic S-curve: rapid growth when resources are depleted, decelerating as the pool approaches capacity, and zero net growth at full capacity.

Each pool limits extraction to 40% of its current level per epoch, preventing complete depletion in a single cycle and creating a sustainable harvesting pressure:

$$\text{available}(t) = 0.4 \cdot R(t)$$

### 4.2 Seasonal Oscillation

The environment exhibits periodic variation via sinusoidal modulation of carrying capacity:

$$K_{\text{eff}}(\text{epoch}) = K \cdot \left(1 + A \cdot \sin\left(\frac{2\pi \cdot \text{epoch}}{T}\right)\right)$$

where $A = 0.25$ (±25% amplitude) and $T = 100$ epochs (one full season). This produces genuine boom-and-bust cycles: populations that expand during favorable seasons face resource scarcity during unfavorable ones, driving adaptive pressure and preventing fitness plateaus.

### 4.3 Stochastic Events

The model includes two classes of stochastic perturbation, generated from a deterministic hash-based pseudo-random sequence:

**Catastrophe** (2% probability per epoch): Reduces carrying capacity to 30–60% of normal for 10–20 epochs. This models mass-extinction–scale events that test population resilience and reward genetic diversity.

**Resource boom** (5% probability per epoch): Spikes all pool levels to 90% of capacity. This models windfall events that enable rapid population expansion and reduce selection pressure temporarily.

Both event types are time-limited and bounded in severity, preventing runaway dynamics.

### 4.4 Density-Dependent Foraging

Per-agent resource extraction decreases with niche crowding:

$$\text{density\_factor} = \frac{1}{1 + n_{\text{niche}} \cdot 0.1}$$

where $n_{\text{niche}}$ is the number of agents sharing the same role. A niche with 10 agents yields a density factor of 0.5 — each agent extracts half as much as it would alone. This implements the ecological principle of intraspecific competition: conspecifics are the strongest competitors because they exploit identical resources.

### 4.5 Cross-Niche Competition

Agents in different niches exert a weaker competitive effect via the cross-niche interference coefficient $\alpha = 0.15$:

$$\text{cross\_penalty} = 1 - \alpha \cdot \frac{N_{\text{total}} - n_{\text{niche}}}{N_{\text{total}}}$$

This models interspecific competition — species that share some resources but specialize in different ecological roles. The coefficient $\alpha = 0.15$ is deliberately low, reflecting the principle that niche specialization reduces competitive overlap.

### 4.6 Complete Extraction Formula

The per-agent, per-epoch extraction combines all five ecological factors:

$$E_i = f_i \cdot s_i \cdot \frac{R_{\text{pool}}}{K_{\text{pool}}} \cdot \frac{1}{1 + n_{\text{niche}} \cdot 0.1} \cdot \left(1 - \alpha \cdot \frac{N - n_{\text{niche}}}{N}\right) \cdot \sigma$$

where $f_i$ is agent fitness, $s_i$ is niche skill match, $R_{\text{pool}}/K_{\text{pool}}$ is pool utilization ratio, and $\sigma = 2.5$ is a scale factor calibrated so that an average agent nets positive ATP at ecological equilibrium.

This formula ensures that extraction is:
- **Proportional**, not winner-take-all
- **Density-dependent** within niches
- **Competitively modulated** across niches
- **Resource-sensitive** (extraction decreases as pools deplete)
- **Fitness-weighted** (fitter agents extract more, but cannot monopolize)

---

## 5. Energy Economics

### 5.1 ATP as Universal Currency

All economic activity is denominated in Adenosine Triphosphate (ATP), an intentional biological metaphor. ATP is the unit of account, medium of exchange, and store of value within the organism. Agents earn ATP through resource extraction and market rewards; they spend it on basal metabolism and replication.

### 5.2 Metabolic Cost

Every agent pays a basal metabolic cost of 0.15 ATP per epoch for the privilege of existing. This creates constant downward pressure on agent balances, ensuring that agents who fail to extract resources will eventually enter stasis and face elimination:

$$\text{balance}(t+1) = \text{balance}(t) - 0.15$$

Balance is clamped at zero (Invariant M-1); the metabolic tick returns the actual amount consumed, preventing negative balances.

### 5.3 Treasury and Redistribution

A central treasury implements five counter-cyclical mechanisms to prevent capital hoarding and maintain economic circulation:

1. **Market Skim** (5%): Applied to all problem-market rewards, funding the treasury reserve.
2. **Balance Decay** (2%/epoch): All agent ATP balances erode by 2% per epoch, creating velocity pressure that prevents indefinite hoarding.
3. **Wealth Tax** (1% above 100 ATP): Agents with balances exceeding 100 ATP pay a 1% marginal tax, with proceeds flowing to the treasury.
4. **Stipend Distribution**: Underrepresented roles (below the fair share of 20% of population) receive subsidies proportional to their deficit, funded from treasury reserves.
5. **Crisis Spending**: When population falls below half of carrying capacity, the treasury injects up to 2.0 ATP per agent from reserves.
6. **Overflow Redistribution**: If treasury reserves exceed 30% of total ATP supply, half the excess is distributed equally across all agents.

This layered approach ensures that ATP circulates rather than accumulating: the treasury collected 9,075 ATP and distributed 9,074 ATP over a 10,000-epoch simulation (Section 10.5), maintaining a mean reserve of only 0.76 ATP.

### 5.4 Ledger Integrity

The ATP ledger tracks all transactions with typed entries (ProofOfSolution, MetabolicCost, Replication, etc.) and computes total supply as the sum of all agent balances at query time (Invariant M-2), rather than maintaining a running counter. This audit-by-recomputation approach eliminates accumulation errors.

---

## 6. Selection and Reproduction

### 6.1 Dynamic Population Cap

The population ceiling is derived from environmental carrying capacity rather than being a fixed constant:

$$K_{\text{pop}} = \left\lfloor \frac{\text{total\_capacity}}{15} \right\rfloor \quad \text{clamped to } [10, 500]$$

This couples population dynamics directly to resource availability: catastrophes that reduce carrying capacity also reduce the allowable population, intensifying selection pressure precisely when resources are scarce.

### 6.2 Fitness-Based Culling

When population exceeds the dynamic cap, the selection engine eliminates agents with the lowest fitness scores. Agents younger than `MATURATION_EPOCHS` (10 epochs) are exempt from culling (Invariant S-2), preventing the premature elimination of agents who have not yet had time to demonstrate their fitness.

### 6.3 Stasis and Death

Agents whose ATP balance reaches zero enter stasis. Agents in stasis for more than `STASIS_TOLERANCE` (3 epochs) are marked for death. This implements a grace period — agents experiencing temporary resource scarcity can survive brief famines but cannot persist indefinitely without income.

### 6.4 Replication

Agents above a fitness threshold and with sufficient ATP can replicate, producing offspring with mutated genomes. Replication costs are deducted atomically from the parent's balance (Invariant M-3), and births per epoch are capped at `MAX_BIRTHS_PER_EPOCH` (3) to prevent population explosions.

### 6.5 Mutation

Mutation pressure is modulated by environmental stress:

$$p_{\text{mutation}} = p_{\text{base}} + \Delta p \cdot \sin(\text{season\_phase})$$

Higher environmental pressure (harsh seasons, catastrophes) increases mutation rates, driving faster adaptation when the environment demands it. This implements the ecological principle that environmental stress accelerates evolutionary change.

---

## 7. Provenance Model

### 7.1 Extending Literary Provenance to Software

The provenance model adapts the multi-layer architecture established in *Deterministic Literary Publishing* [3] for literary manuscripts. Where the literary protocol hashes 31 text blocks, 5 artifacts, 10 images, and 10 prompts across four Merkle trees, the software protocol hashes 39 source files across 7 crate-level Merkle trees. The cryptographic infrastructure is identical: SHA-256 hashing, binary Merkle tree construction with the odd-leaf-duplicate rule, and a composite edition root suitable for on-chain anchoring.

### 7.2 Evidence Chain

Provenance is established through six independent layers:

| Layer | Evidence | Record | Purpose |
|-------|----------|--------|---------|
| 1 | Local filesystem | OS metadata | First creation timestamps |
| 2 | Git commit history | 18 commits on GitHub | Continuous authorship timeline |
| 3 | Merkle trees | `dist/merkle.json` | Per-crate integrity proofs |
| 4 | SHA-256 edition root | `cd78b3be...d2cc3` | Content integrity fingerprint |
| 5 | ORCID | `0009-0008-8425-939X` | Author identity record |
| 6 | DOI | `10.5281/zenodo.18729652` | Permanent academic identifier |

Table 2. Six-layer evidence chain.

Each layer serves as a check on the others. A modified source file produces a different crate root, which changes the edition root, which mismatches any on-chain anchor. A forged git history is contradicted by the DOI registration timestamp. No single point of compromise can produce a consistent forgery across all six layers.

### 7.3 On-Chain Anchoring

The edition root can be anchored on Polygon mainnet via the same `LiteraryAnchor` contract used for the 2,500 Donkeys literary protocol:

| Property | Value |
|----------|-------|
| Contract | `0x97f456300817eaE3B40E235857b856dfFE8bba90` |
| Network | Polygon Mainnet (Chain ID 137) |
| Method | `anchorEdition(editionRoot, ipfsCID)` |
| Author Identity | `0xB9ffa688A8Bb332221030BbBE46bE5bF03323170` |
| Author Wallet | `0xC91668184736BF75C4ecE37473D694efb2A43978` |
| Estimated Cost | < 0.50 USD |

Table 3. On-chain anchoring configuration.

The author identity contract links the cryptographic wallet to the author's legal name, pen name, ORCID, and published bibliography, creating a unified identity graph queryable by any verifier.

---

## 8. Merkle Tree System

### 8.1 Construction

The system constructs seven independent Merkle trees — one per crate — using the following rules:

1. **Leaf computation**: $h_i = \text{SHA-256}(\text{content}_i)$ where content is the raw bytes of each `.rs` source file.

2. **Internal nodes**: $h_{\text{parent}} = \text{SHA-256}(h_{\text{left}} \| h_{\text{right}})$ where $\|$ is string concatenation of lowercase hex digests.

3. **Odd-leaf rule**: If a layer has an odd number of nodes, the last node is duplicated before pairing.

4. **Ordering**: Files within each crate are sorted alphabetically by path.

### 8.2 Crate Roots

Each crate's Merkle root is a 256-bit commitment to the integrity of all source files within that crate:

| Crate | Role | Leaves | Root (prefix) |
|-------|------|--------|---------------|
| `genesis-dna` | Genome and Identity | 7 | `e348c9a9...` |
| `metabolism` | Energy and Economics | 6 | `f739fd3a...` |
| `ecosystem` | Communication and Markets | 8 | `88b92989...` |
| `evolution` | Mutation and Selection | 5 | `fd7f0afb...` |
| `gateway` | Server and World Engine | 7 | `9ec9925d...` |
| `apostle` | Outbound Intelligence | 5 | `ab727d84...` |
| `tests` | Integration Tests | 1 | `6fa77826...` |

Table 4. Per-crate Merkle roots.

### 8.3 Edition Root

The edition root is the SHA-256 hash of the concatenation of all seven crate roots:

$$\text{editionRoot} = \text{SHA-256}(r_1 \| r_2 \| r_3 \| r_4 \| r_5 \| r_6 \| r_7)$$

**Edition Root**: `cd78b3be6a7f569a8a7a24d49a47ccd01af0fa5ab7c8a04baa6f7ee0367d2cc3`

This single 256-bit value is the commitment suitable for on-chain anchoring. From it, any individual source file can be verified via a Merkle inclusion proof consisting of sibling hashes along the path from leaf to crate root, then from crate root to edition root.

### 8.4 Verification Procedure

Any party can verify the source code integrity:

1. Clone the repository from `https://github.com/FTHTrading/AI`.
2. Run the Merkle generator: `powershell -ExecutionPolicy Bypass -File scripts/merkle.ps1`.
3. Compare the computed edition root against the published value (or on-chain anchor).
4. If roots match, the source code is authentic and unmodified.

For partial verification (e.g., a single crate), compute the crate's Merkle root from its source files and verify it matches the published crate root in `dist/merkle.json`.

---

## 9. Protocol Specification (SOP-1)

The Software Organism Protocol v1 (SOP-1) formalizes the requirements demonstrated by the Genesis Protocol reference implementation. It specifies five functional layers (Genome, Metabolism, Ecosystem, Evolution, Gateway), a state machine lifecycle, ecological model requirements, energy economics constraints, and provenance verification procedures.

### 9.1 State Machine

Every SOP-1 organism progresses through a lifecycle:

$$\text{GENESIS} \rightarrow \text{SPAWNING} \rightarrow \text{RUNNING} \rightarrow \text{STABLE} \rightarrow \text{ADAPTING} \rightarrow \text{RUNNING} \rightarrow \cdots$$

with a terminal `EXTINCTION` state reachable from any running state when the population reaches zero.

### 9.2 Relationship to LPS-1

SOP-1 extends the Literary Protocol Standard (LPS-1) established in *Deterministic Literary Publishing* [3]. Where LPS-1 specifies integrity guarantees for static literary content, SOP-1 extends those guarantees to executable systems. The key adaptation is replacing the four-tree literary model (manuscript, artifact, image, prompt) with a variable-tree software model (one tree per crate), while retaining the same hash construction, edition root computation, and on-chain anchoring methods.

### 9.3 Invariant Registry

The protocol defines 14 invariants across five domains. All invariants must hold at all times during system operation; violation of any invariant indicates a bug requiring correction before further operations. The complete invariant table is provided in Appendix A.

---

## 10. Evaluation

### 10.1 Test Coverage

The system is validated by 143 automated tests across all six crates:

| Crate | Tests | Coverage |
|-------|-------|----------|
| `genesis-dna` | Genome construction, trait validation, lineage tracking, role assignment | Core |
| `metabolism` | ATP accounting, ledger integrity, treasury operations, proof-of-work | Core |
| `ecosystem` | Mesh communication, problem market, publication gate, telemetry | Core |
| `evolution` | Mutation engine, selection pressure, gene transfer, fitness calculation | Core |
| `gateway` | World engine, epoch execution, server endpoints, persistence, security | Integration |
| `tests` | Multi-epoch load simulation, population dynamics, resource sustainability | End-to-end |
| **Total** | **143 tests, 0 failures** | |

Table 5. Test suite composition. All tests pass with `cargo test --workspace`.

### 10.2 Ecological Realism

The ecological model produces dynamics consistent with Lotka–Volterra predictions:

- **Population oscillation**: Agent populations exhibit boom-bust cycles synchronized with seasonal oscillation, with amplitude modulated by carrying capacity.
- **Resource recovery**: Depleted resource pools recover via logistic growth during low-extraction periods, consistent with the equation $dR/dt = rR(1 - R/K)$.
- **Catastrophe response**: Population declines during catastrophe events (~30–60% capacity reduction) followed by recovery when conditions normalize.
- **Niche partitioning**: Density-dependent foraging drives agents toward underexploited niches, producing stable coexistence at carrying capacity.
- **Competitive exclusion avoidance**: The cross-niche coefficient ($\alpha = 0.15$) is sufficiently low to permit multi-niche coexistence, consistent with ecological theory that $\alpha < 1$ prevents competitive exclusion [7].

### 10.3 Provenance Verification

The Merkle tree generator (`scripts/merkle.ps1`) was validated by:

1. Computing the edition root from the 39 source files.
2. Verifying each per-file SHA-256 hash against `dist/manifest.json`.
3. Reconstructing each crate Merkle root from individual file hashes.
4. Confirming the edition root `cd78b3be...d2cc3` matches the composite of all 7 crate roots.

The generator runs in under 3 seconds on commodity hardware (PowerShell 5.1, Windows 10).

### 10.4 Invariant Compliance

All 14 defined invariants (Appendix A) hold for the current edition. Key verifications:

- **E-1** (Logistic Regeneration): `ResourcePool::regenerate()` implements $R += r \cdot R \cdot (1 - R/K)$ with clamping.
- **M-1** (Non-Negative Balances): `metabolic_tick()` returns actual consumed amount, clamping at zero.
- **S-1** (Dynamic Population Cap): `run_epoch()` computes `total_capacity / 15`, clamped to [10, 500].
- **P-1** (Deterministic Edition Root): Independently recomputable from source files.

### 10.5 Empirical Simulation: 10,000-Epoch Run

To validate the ecological model beyond unit-level testing, we executed a single deterministic simulation of 10,000 epochs from a primordial state (20 agents, 50.0 ATP each). The full epoch-by-epoch dataset is available as `dist/simulation-10k.csv` in the source repository.

#### 10.5.1 Population Dynamics

| Metric | Bootstrap (0–100) | Growth (100–500) | Early Equilibrium (500–2000) | Mid Run (2000–5000) | Late Run (5000–10000) |
|--------|---:|---:|---:|---:|---:|
| Mean Population | 45.2 | 51.2 | 56.8 | 56.1 | 57.4 |
| Mean ATP Supply | 510.3 | 450.8 | 668.1 | 633.5 | 697.8 |
| Mean Fitness | 0.5224 | 0.5292 | 0.5644 | 0.5708 | 0.5623 |

Table 6. Phase-segmented population and economic metrics across 10,000 epochs.

The population converges from 20 primordial agents to an equilibrium band of 50–60 within the first 500 epochs. At steady state (epochs 5000–10000), mean population is 57.4 with standard deviation 2.11 — a coefficient of variation of 3.7%, indicating strong stability. The minimum population observed is 20 (epoch 0 only); the maximum is 60 (matching the dynamic carrying capacity ceiling for abundant resource conditions). Only 17 epochs across the entire run show population below 40, all occurring during the bootstrap phase.

#### 10.5.2 ATP Economics

| Metric | Value |
|--------|------:|
| Mean ATP per Agent (overall) | 11.72 |
| Mean ATP per Agent (equilibrium) | 12.15 |
| Total ATP Supply: min | 232.68 |
| Total ATP Supply: max | 1013.03 |
| Equilibrium ATP Supply: mean ± σ | 697.8 ± 109.4 |

Table 7. ATP supply dynamics across 10,000 epochs.

The initial ATP surplus (20 × 50.0 = 1000 ATP) declines rapidly as primordial grants are consumed by basal metabolism. The system finds economic equilibrium around 12.15 ATP per agent — roughly 80 epochs of survival budget at basal cost (0.15 ATP/epoch), providing a meaningful survival buffer without excessive hoarding.

**Treasury cycling.** The treasury collected 9,075.1 ATP and distributed 9,074.4 ATP over the run, with a mean reserve of only 0.76 ATP. This demonstrates that the redistribution mechanisms (decay, wealth tax, stipends, crisis spending, overflow redistribution) effectively prevent capital accumulation while maintaining a small emergency buffer. The maximum single-epoch distribution was 21.13 ATP (during a crisis event), and only 152 of 10,000 epochs (1.5%) had zero treasury distribution.

#### 10.5.3 Selection and Fitness

Mean population fitness rises from 0.5224 (bootstrap) to 0.5708 (mid-run), representing a 9.3% improvement through natural selection. Individual peak fitness reached 0.9824 at epoch 4,718, demonstrating that the mutation and selection engines can produce high-performing agents over time. The fitness plateau around 0.56–0.57 suggests a selection–mutation equilibrium: evolutionary gains from culling low-fitness agents are balanced by the introduction of variation through mutation.

#### 10.5.4 Demographic Turnover

| Metric | Value |
|--------|------:|
| Total Births | 902 |
| Total Deaths | 865 |
| Net Growth (20 → 57) | +37 |
| Epochs with Births | 674 (6.7%) |
| Epochs with Deaths | 760 (7.6%) |
| Max Births in One Epoch | 3 |
| Max Deaths in One Epoch | 4 |

Table 8. Demographic statistics across 10,000 epochs.

The birth-to-death ratio of 1.04:1 maintains slight positive pressure sufficient to sustain the population without runaway growth. The low per-epoch birth rate (6.7% of epochs) reflects the stringent replication requirements: agents must exceed both a fitness threshold and ATP cost (25.0 ATP). Deaths occur slightly more frequently (7.6% of epochs) but are typically single-agent events — the maximum of 4 deaths in a single epoch is rare and coincides with catastrophe-driven carrying capacity reduction.

#### 10.5.5 Environmental Dynamics

Seasonal distribution across 10,000 epochs: Autumn 33.0%, Winter 33.0%, Spring 17.0%, Summer 17.0%. The unequal distribution reflects the sinusoidal season cycle and epoch-to-season mapping. Catastrophe events occurred in 3,820 epochs (38.2%), consistent with the 5% base probability compounded with multi-epoch catastrophe duration (mean ~7 epochs per event).

#### 10.5.6 Role Distribution

At equilibrium (epochs 5000–10000), the five specialist roles maintain near-uniform representation:

| Role | Mean Count | Min | Max |
|------|---:|---:|---:|
| Optimizer | 11.6 | 9 | 15 |
| Strategist | 11.6 | 9 | 14 |
| Communicator | 11.5 | 9 | 14 |
| Archivist | 11.1 | 8 | 14 |
| Executor | 11.6 | 9 | 15 |

Table 9. Role distribution at equilibrium (epochs 5000–10000).

The near-uniform distribution validates the treasury stipend mechanism: underrepresented roles receive proportional subsidies, preventing niche monopolies. The slight deficit in Archivists (11.1 vs. 11.6 mean for other roles) suggests they face marginally higher competitive pressure but are not at risk of extinction.

---

## 11. Discussion

### 11.1 Ecological Model vs. Economic Models

Most multi-agent simulations use economic models (auctions, markets, mechanism design) for resource allocation. Our ecological approach offers different trade-offs: it produces emergent population dynamics that are difficult to achieve with economic mechanisms, but it provides weaker guarantees about individual agent optimality. The choice depends on whether the system's purpose is to model collective dynamics (where ecological models excel) or individual decision-making (where economic models are stronger).

### 11.2 Provenance Generalization

The successful extension of literary provenance to software provenance demonstrates that the cryptographic infrastructure — Merkle trees, edition roots, on-chain anchoring — is domain-agnostic. Any structured collection of files can be decomposed into category-level Merkle trees and composed into a single edition root. The literary model [3] uses four content categories (text, artifacts, images, prompts); the software model uses seven (one per crate). Additional categories can be added without modifying the core protocol.

### 11.3 Shared Infrastructure

Both Genesis Protocol and the 2,500 Donkeys literary protocol [3] share the same on-chain infrastructure: the `LiteraryAnchor` contract at `0x97f456300817eaE3B40E235857b856dfFE8bba90`, the `AuthorIdentity` contract at `0xB9ffa688A8Bb332221030BbBE46bE5bF03323170`, and the author wallet `0xC91668184736BF75C4ecE37473D694efb2A43978`. This demonstrates that a single set of smart contracts can serve as provenance infrastructure for an arbitrary number of works across different domains.

### 11.4 Limitations

**Treasury hoarding (v1.0 defect, corrected).** In the initial v1.0 release, five of six treasury mechanisms existed as implemented but uncalled code. Only the skim function was wired into the epoch loop, causing approximately 88% of ATP to accumulate in the treasury with no redistribution pathway. This was identified through external review, diagnosed as dead-code accumulation, and corrected by wiring all five mechanisms (decay, wealth tax, stipends, crisis spending, overflow redistribution) into `run_epoch()`. The 10,000-epoch empirical validation in Section 10.5 confirms the fix: the treasury now retains only 0.76 ATP on average versus 9,074 ATP distributed. This defect is disclosed here as a case study in the importance of integration testing for economic subsystems — unit tests confirmed each mechanism worked in isolation, but no test verified they were invoked in the main loop.

**Ecological simplification.** The model uses five discrete niches rather than a continuous niche space. Real ecosystems exhibit niche overlap, character displacement, and adaptive radiation that are not captured by discrete role assignments.

**Deterministic pseudo-randomness.** Stochastic events use a hash-based PRNG seeded deterministically, which enables reproducibility but means the specific event sequence is predetermined rather than genuinely random. This is a deliberate design choice favoring reproducibility over stochasticity. A consequence is that the 10,000-epoch results in Section 10.5 represent a single trajectory; ensemble statistics across multiple seeds would strengthen the empirical claims.

**Single-chain anchoring.** The current implementation prepares for but does not execute on-chain anchoring. Cross-chain redundancy (Bitcoin, Ethereum) would provide stronger durability guarantees.

**Key management.** The author's private key is a single point of failure for on-chain operations. Multi-signature schemes could mitigate this risk but are not currently implemented.

**Apostle crate maturity.** The `apostle` crate provides conversion tracking infrastructure but has not been exercised in production deployments. Its integration with the broader ecosystem remains at proof-of-concept stage.

---

## 12. Conclusion

We have presented Genesis Protocol, a six-crate Rust organism implementing Lotka–Volterra inspired resource competition with cryptographic provenance verification. The system demonstrates that biologically grounded ecological dynamics — logistic growth, seasonal oscillation, density-dependent foraging, stochastic perturbation — produce realistic population behavior in multi-agent simulations, and that the provenance model established for literary publishing generalizes to executable software without modification to its cryptographic infrastructure.

A 10,000-epoch empirical run validates the model: population converges to a stable equilibrium band (mean 57.4, CV 3.7%), ATP per agent stabilizes at 12.15 (an 80-epoch survival buffer), mean fitness improves 9.3% through selection, and the treasury cycles 9,074 of 9,075 collected ATP back into the economy — demonstrating effective counter-cyclical redistribution. The only significant defect discovered through review (treasury hoarding due to uncalled redistribution code) was corrected and is disclosed transparently in Section 11.4.

The organism comprises 39 source files totaling 10,015 lines of Rust, validated by 143 automated tests. Its integrity is committed through 7 crate-level Merkle trees composing into a single edition root (`cd78b3be...d2cc3`) anchorable on a production blockchain at a cost of less than $0.50 USD. The Software Organism Protocol (SOP-1) formalizes these guarantees as a reproducible specification for future implementations.

Every claim in this paper is independently verifiable. The source code is public on GitHub. The Merkle trees are recomputable from the source files. The invariants are testable against the running code. The on-chain infrastructure is source-verified on Polygonscan.

The model demonstrates that verifiable provenance for software organisms is technically feasible, economically accessible, and architecturally compatible with rigorous ecological simulation — providing a foundation for artificial life systems whose behavior and integrity can be audited from first principles.

---

## References

[1] C. G. Langton, "Artificial life," in *Artificial Life*, Santa Fe Institute Studies in the Sciences of Complexity, vol. 6, pp. 1–47, Addison-Wesley, 1989.

[2] T. S. Ray, "An approach to the synthesis of life," in *Artificial Life II*, Santa Fe Institute Studies in the Sciences of Complexity, vol. 10, pp. 371–408, Addison-Wesley, 1992.

[3] K. Burns, "Deterministic literary publishing: A multi-layer provenance model for verifiable manuscripts," Independent research, v1.0, Feb. 2026. DOI: 10.5281/zenodo.18646886.

[4] M. Wooldridge, *An Introduction to MultiAgent Systems*, 2nd ed., John Wiley & Sons, 2009.

[5] A. J. Lotka, "Contribution to the theory of periodic reactions," *Journal of Physical Chemistry*, vol. 14, no. 3, pp. 271–274, 1910.

[6] V. Volterra, "Variations and fluctuations of the number of individuals in animal species living together," *ICES Journal of Marine Science*, vol. 3, no. 1, pp. 3–51, 1928.

[7] P. Chesson, "Mechanisms of maintenance of species diversity," *Annual Review of Ecology and Systematics*, vol. 31, pp. 343–366, 2000.

[8] R. C. Merkle, "A digital signature based on a conventional encryption function," *Advances in Cryptology — CRYPTO '87*, Lecture Notes in Computer Science, vol. 293, pp. 369–378, Springer, 1988.

[9] S. Chacon and B. Straub, *Pro Git*, 2nd ed., Apress, 2014.

[10] J. Benet, "IPFS — Content addressed, versioned, P2P file system," arXiv:1407.3561, 2014.

[11] Reproducible Builds Project, "Reproducible builds," https://reproducible-builds.org, 2023.

---

## Appendix A: System Invariants

All 14 invariants must hold at all times during system operation.

### Ecology (E-1 through E-4)

| ID | Invariant | Formal Statement |
|----|-----------|-----------------|
| E-1 | Logistic Resource Regeneration | $R(t+1) = R(t) + r \cdot R(t) \cdot (1 - R(t)/K)$ |
| E-2 | Seasonal Modulation | $K_{\text{eff}} = K \cdot (1 + A \cdot \sin(2\pi \cdot \text{epoch} / T))$ |
| E-3 | Proportional Extraction | Winner-take-all dynamics are prohibited |
| E-4 | Density-Dependent Foraging | $\text{density} = 1/(1 + n_{\text{niche}} \cdot 0.1)$ |

### Metabolism (M-1 through M-3)

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| M-1 | Non-Negative Balances | `metabolic_tick()` clamps at zero |
| M-2 | Computed Supply | Sum of all balances at query time |
| M-3 | Atomic Replication Cost | Single deduction operation |

### Selection (S-1 through S-3)

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| S-1 | Dynamic Population Cap | `total_capacity / 15`, clamped [10, 500] |
| S-2 | Maturation Guard | Agents < 10 epochs exempt from culling |
| S-3 | Stasis Tolerance | 3-epoch grace period before death |

### Genome (G-1 through G-2)

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| G-1 | Cryptographic Primordial Diversity | SHA-256 derived seed genomes |
| G-2 | Environmentally-Responsive Mutation | Seasonal stress modulates mutation rate |

### Provenance (P-1 through P-2)

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| P-1 | Deterministic Edition Root | Merkle(crate_roots), independently recomputable |
| P-2 | Complete Manifest Coverage | All 39 files have SHA-256 entries |

Table A1. Complete invariant registry.

---

## Appendix B: Crate Hash Table

All SHA-256 Merkle roots as of edition v1.0 (git commit `889a560978939d7a40cdf2bebc8c539c0b83b435`).

| Crate | Merkle Root |
|-------|-------------|
| `genesis-dna` | `e348c9a9678af25a67e7d9c6cfe21702b588abf21415a316ce33b33d9f6f2197` |
| `metabolism` | `f739fd3aab927ce26847aa6a311ba5d7df64af1c451ce4df331361db4ec5e290` |
| `ecosystem` | `88b9298b76f6d4e207fb0f55303282ad7421111171a4fbcfd264f74f037e19dc` |
| `evolution` | `fd7f0afbd6ab83d59eff5624c5144f28e50be1727156e3fd904f5fe90e3ec775` |
| `gateway` | `9ec9925dc0f2df9d07d73603cdb8a99e0c07356e70a5bd9ef1446501f01732e8` |
| `apostle` | `ab727d8401a8275836a29309a344021e0414def4cd753b747cdfdf4926e87944` |
| `tests` | `6fa7782e6959b9ae5bafbf0651109bacf85b46f7070e95d10f39232aaf4926cd` |
| **Edition Root** | **`cd78b3be6a7f569a8a7a24d49a47ccd01af0fa5ab7c8a04baa6f7ee0367d2cc3`** |

Table B1. Per-crate Merkle roots and composite edition root.

**Source Repository:** https://github.com/FTHTrading/AI  
**Author Wallet:** `0xC91668184736BF75C4ecE37473D694efb2A43978`  
**Author Identity Contract:** `0xB9ffa688A8Bb332221030BbBE46bE5bF03323170`  
**LiteraryAnchor Contract:** `0x97f456300817eaE3B40E235857b856dfFE8bba90`

---

## Appendix C: How to Cite This Work

### BibTeX

```bibtex
@techreport{burns2026genesis,
  title     = {Genesis Protocol: A Lotka--Volterra Inspired Software Organism
               with Cryptographic Provenance},
  author    = {Burns, Kevan},
  year      = {2026},
  month     = {February},
  version   = {1.0},
  doi       = {10.5281/zenodo.18729652},
  url       = {https://doi.org/10.5281/zenodo.18729652},
  note      = {Independent research. Reference implementation: 39 Rust source
               files, 10,015 lines, 143 tests. Edition root:
               cd78b3be6a7f569a8a7a24d49a47ccd01af0fa5ab7c8a04baa6f7ee0367d2cc3.}
}
```

### APA (7th Edition)

Burns, K. (2026). *Genesis Protocol: A Lotka–Volterra inspired software organism with cryptographic provenance* (Version 1.0). Independent research. https://doi.org/10.5281/zenodo.18729652

### Chicago

Burns, Kevan. "Genesis Protocol: A Lotka–Volterra Inspired Software Organism with Cryptographic Provenance." Version 1.0. Independent research, February 2026. https://doi.org/10.5281/zenodo.18729652.

### IEEE

K. Burns, "Genesis Protocol: A Lotka–Volterra Inspired Software Organism with Cryptographic Provenance," Independent research, v1.0, Feb. 2026. DOI: 10.5281/zenodo.18729652. [Online]. Available: https://doi.org/10.5281/zenodo.18729652

---

Version 1.0 — February 21, 2026  
Kevan Burns — Independent Researcher — FTH Trading, Norcross, GA  
ORCID: 0009-0008-8425-939X  
Source code: https://github.com/FTHTrading/AI  
Companion work: *Deterministic Literary Publishing* (Burns, 2026) — https://github.com/FTHTrading/2500-donkeys
