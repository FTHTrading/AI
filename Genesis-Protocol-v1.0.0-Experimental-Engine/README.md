# Genesis Protocol

Deterministic evolutionary economics engine. 13 crates. 349 tests. Pure Rust.

Digital organisms are born with cryptographic genomes, metabolize scarce resources, mutate under selective pressure, and die when they can't pay their entropy cost. Institutional mechanisms — treasury, taxation, catastrophe events, adaptive regulation — emerge as endogenous feedback loops, not exogenous parameters. Every state transition is deterministically replayable and cryptographically verifiable.

This is not a simulation wrapper. It is a research platform for conducting reproducible macroeconomic experiments on populations of autonomous agents operating under real scarcity constraints.

---

## What Has Been Proven

Eight controlled experiments across five research domains. 1,220 independent world simulations. 610,000 total economic epochs. Zero civilization collapses.

| Experiment | Worlds | Epochs | Key Finding |
|---|---|---|---|
| **Entropy Sweep** | 200 | 100,000 | 10× increase in metabolic cost produces only 4.9% Gini increase. No collapses. |
| **Catastrophe Resilience** | 140 | 70,000 | 0–3% catastrophe probability per epoch → zero collapses. Deaths scale linearly (0 → 12.4). Population declines only 4.6%. |
| **Inequality Threshold** | 160 | 80,000 | Varying wealth tax threshold from 0.20 to 0.90 produces 31.6% Gini increase. Population stability and mean fitness remain invariant. |
| **Treasury Stability** | 180 | 90,000 | Sweeping reserve deployment threshold 0.10–0.90: aggressive deployment yields 2.1% higher fitness, <1% Gini variation. Zero collapses at any policy. |
| **FTH Reserve Stress** (4 tiers) | 540 | 270,000 | Optimal treasury threshold shifts +0.60 (0.10→0.70) from calm to crisis. Crossover detected: deployment outperforms hoarding until shock rate exceeds 1.5%. |

All results are deterministically reproducible from seed `20260222`, verified via SHA-256 result hashing, and exported as CSV datasets for independent analysis.

Full methodology: [papers/genesis-protocol-III-the-experimental-method.md](papers/genesis-protocol-III-the-experimental-method.md)

---

## Architecture

Thirteen crates. One organism.

| Layer | Crate | Role |
|---|---|---|
| **Identity** | `genesis-dna` | Cryptographic genome, trait expression, phenotype derivation |
| **Economics** | `metabolism` | ATP energy ledger, treasury, metabolic decay |
| **Econometrics** | `genesis-econometrics` | Gini coefficient, wealth distribution analysis |
| **Evolution** | `evolution` | Mutation, natural selection, horizontal gene transfer |
| **Population** | `ecosystem` | Social mesh, problem markets, carrying capacity, telemetry |
| **Regulation** | `genesis-homeostasis` | Adaptive Cortex — real-time parameter modulation from population signals |
| **Multiverse** | `genesis-multiverse` | Parallel world instantiation, parameter sweep, cross-world aggregation |
| **Experiments** | `genesis-experiment` | Experiment engine, trial runner, statistical reporting |
| **Cryptography** | `genesis-anchor` | Dual-chain anchoring — SHA-256 state chain + BLAKE3 genome chain |
| **Replay** | `genesis-replay` | Deterministic replay from any checkpoint |
| **Federation** | `genesis-federation` | Cross-instance communication protocol |
| **Gateway** | `gateway` | HTTP API, SSE event stream, observatory dashboard |
| **Recruitment** | `apostle` | Outbound integration and agent recruitment |

### Dual-Chain Cryptographic Architecture

Every tick, two independent hash chains advance:

- **State Chain** (SHA-256): `H(previous_state_hash ‖ epoch ‖ population_snapshot)` — anchors macroeconomic state
- **Genome Chain** (BLAKE3): `H(previous_genome_hash ‖ mutated_genomes)` — anchors evolutionary lineage

Divergence between chains is detectable. Replay integrity is verifiable to any depth.

---

## Reproduce an Experiment

```bash
# Clone and build
git clone https://github.com/FTHTrading/AI.git
cd AI
cargo build --release

# Run all eight experiments (~49 seconds)
cargo run --release --bin run_experiments

# Results appear in experiments/
#   entropy_sweep/          — 200 worlds, data + manifest + report
#   catastrophe_resilience/ — 140 worlds, data + manifest + report
#   inequality_threshold/   — 160 worlds, data + manifest + report
#   treasury_stability/     — 180 worlds, data + manifest + report
#   fth_reserve_calm/       — 135 worlds, shock=0.001
#   fth_reserve_moderate/   — 135 worlds, shock=0.005
#   fth_reserve_stressed/   — 135 worlds, shock=0.015
#   fth_reserve_crisis/     — 135 worlds, shock=0.030

# Verify the full test suite
cargo test --workspace
# Expected: 349 passed, 0 failed
```

Each experiment exports:
- **CSV data** — per-trial metrics for independent analysis
- **JSON manifest** — parameters, seed, SHA-256 result hash
- **Text report** — human-readable statistical summary

---

## Current Metrics

| Metric | Value |
|---|---|
| Crates | 13 |
| Tests | 349 passing, 0 failed, 7 ignored |
| Compiler warnings | 0 |
| Experiment worlds | 1,220 |
| Total epochs simulated | 610,000 |
| Civilization collapses | 0 |
| Deterministic seed | `20260222` |
| Result verification | SHA-256 manifest hash per experiment |

---

## Deliverables

**[Genesis Experiment Pack v3](deliverables/genesis-experiment-pack-v3/)** — Portable bundle containing the executive brief, all eight experiment outputs (manifests, CSV datasets, reports), reproduction instructions, and SHA-256 integrity verification. Self-contained and independently verifiable. Includes the 4-tier FTH Reserve Stress Suite (540 worlds, 270,000 epochs, crossover analysis).

Rebuild the pack:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_experiment_pack_v3.ps1
```

[Pack v2](deliverables/genesis-experiment-pack-v2/) (4 experiments, 680 worlds) and [Pack v1](deliverables/genesis-experiment-pack-v1/) (3 experiments, 500 worlds) remain available.

---

![Rust](https://img.shields.io/badge/rust-edition%202021-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

---

*Built by [FTHTrading](https://github.com/FTHTrading)*
