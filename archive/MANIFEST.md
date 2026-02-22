# Genesis Protocol v1.0.0 — Frozen Archive Manifest

## Identity

| Field | Value |
|---|---|
| Version | v1.0.0 |
| Freeze Date | 2026-02-22 |
| Commit Hash | 1955dfa900296065308be5dcd232c580e9e8ef9a |
| Branch | master |
| Repository | https://github.com/FTHTrading/AI |
| Author | FTHTrading |

## System Metrics at Freeze

| Metric | Value |
|---|---|
| Crates | 13 |
| Tests | 349 passing, 0 failed, 7 ignored |
| Compiler Warnings | 0 |
| Experiments | 8 (4 foundational + 4-tier FTH reserve stress suite) |
| Total Worlds | 1,220 |
| Total Epochs | 610,000 |
| Civilization Collapses | 0 |
| Deterministic Seed | 20260222 |
| Runtime (full suite) | ~49 seconds (release build) |

## Contents

### Source Code
- 13 Rust crates (full workspace)
- Binary: `run_experiments` (experiment orchestrator)
- Binary: `genesis-protocol` (main gateway)

### Experiment Outputs
- `experiments/entropy_sweep/` — 200 worlds, 100,000 epochs
- `experiments/catastrophe_resilience/` — 140 worlds, 70,000 epochs
- `experiments/inequality_threshold/` — 160 worlds, 80,000 epochs
- `experiments/treasury_stability/` — 180 worlds, 90,000 epochs
- `experiments/fth_reserve_calm/` — 135 worlds, 67,500 epochs (shock=0.001)
- `experiments/fth_reserve_moderate/` — 135 worlds, 67,500 epochs (shock=0.005)
- `experiments/fth_reserve_stressed/` — 135 worlds, 67,500 epochs (shock=0.015)
- `experiments/fth_reserve_crisis/` — 135 worlds, 67,500 epochs (shock=0.030)

### Deliverables
- `deliverables/genesis-experiment-pack-v3/` — 27 files, SHA-256 verified
- `deliverables/genesis-experiment-pack-v2/` — Pack v2 (4 experiments)
- `deliverables/genesis-experiment-pack-v1/` — Pack v1 (3 experiments)

### Papers
- `papers/sravan-executive-brief.md` — Institutional decision instrument
- `papers/genesis-protocol-III-the-experimental-method.md` — Research methodology

## Key Findings at Freeze

### Treasury Stability (Experiment 4)
- Aggressive deployment (threshold 0.10) yields 2.1% higher fitness than hoarding (0.90)
- <1% Gini variation across all thresholds
- Zero collapses at any policy

### FTH Reserve Stress Suite (Experiments 5-8)
- Optimal threshold shifts +0.60 from calm (0.10) to crisis (0.70)
- Crossover detected at ~1.5% shock rate
- Deployment outperforms hoarding below crossover; hoarding outperforms above
- Fitness degradation calm→crisis: 3.9%
- Zero collapses across all 540 FTH worlds

## Cryptographic Architecture
- **State Chain**: SHA-256 — H(previous_state_hash ‖ epoch ‖ population_snapshot)
- **Genome Chain**: BLAKE3 — H(previous_genome_hash ‖ mutated_genomes)
- **Experiment Verification**: SHA-256 manifest hash per experiment
- **Deterministic Replay**: Any trial reproducible from seed + parameters

## Reproduction

```bash
git clone https://github.com/FTHTrading/AI.git
cd AI
git checkout 1955dfa900296065308be5dcd232c580e9e8ef9a
cargo build --release
cargo test --workspace  # Expected: 349 passed, 0 failed
cargo run --release --bin run_experiments  # ~49 seconds, 1,220 worlds
```

## Archive Integrity

This manifest accompanies a compressed archive. The archive SHA-256 hash and IPFS CID are recorded separately upon upload.

---

*Genesis Protocol is built and maintained by [FTHTrading](https://github.com/FTHTrading).*
*Frozen 2026-02-22 at commit 1955dfa.*
