# Genesis Experiment Pack v3 — Reproduction Guide

## What This Pack Contains

| Item | Description |
|---|---|
| `01_SRAVAN_EXECUTIVE_BRIEF.md` | Decision-grade summary of platform capabilities and experimental results |
| `02_EXPERIMENTS/` | Eight experiment outputs (manifest + data + report per experiment) |
| `03_INTEGRITY/sha256sums.txt` | SHA-256 hash of every file in this pack |
| `04_LICENSE_NOTES.md` | Licensing and attribution |

### Experiments Included

| Experiment | Worlds | Epochs | Independent Variable |
|---|---|---|---|
| Entropy Sweep | 200 | 100,000 | Metabolic cost of existence |
| Catastrophe Resilience | 140 | 70,000 | Catastrophe probability per epoch |
| Inequality Threshold | 160 | 80,000 | Gini threshold for wealth tax activation |
| Treasury Stability | 180 | 90,000 | Treasury overflow threshold (reserve deployment policy) |
| **FTH Reserve Calm** | **135** | **67,500** | **Treasury threshold under shock=0.001** |
| **FTH Reserve Moderate** | **135** | **67,500** | **Treasury threshold under shock=0.005** |
| **FTH Reserve Stressed** | **135** | **67,500** | **Treasury threshold under shock=0.015** |
| **FTH Reserve Crisis** | **135** | **67,500** | **Treasury threshold under shock=0.030** |

**Total: 1,220 worlds, 610,000 epochs, zero collapses.**

---

## What's New in v3

The **FTH Reserve Stress Suite** applies the Genesis engine to a multi-tier domain question:

> **How should treasury deployment policy adapt as external shock frequency increases?**

Four shock tiers sweep `treasury_overflow_threshold` from 0.10 to 0.90 under different catastrophe base probabilities (0.001 calm → 0.030 crisis). Each tier runs 135 worlds (9 steps × 15 runs × 500 epochs). Key findings:

| Shock Tier | Catastrophe Prob | Optimal Threshold | Mean Fitness | Collapse Rate |
|---|---|---|---|---|
| Calm | 0.001 | 0.10 (deploy) | 0.5458 | 0% |
| Moderate | 0.005 | 0.30 | 0.5485 | 0% |
| Stressed | 0.015 | 0.60 | 0.5575 | 0% |
| Crisis | 0.030 | 0.70 (hoard) | 0.5705 | 0% |

### Cross-Tier Synthesis

- **Policy shift:** Optimal threshold moves +0.60 from calm (0.10) to crisis (0.70)
- **Crossover detected:** Deployment outperforms hoarding until shock rate exceeds ~1.5% per epoch
- **Fitness degradation:** Only 3.9% fitness decline from calm to crisis — the system adapts
- **Zero collapses:** All 540 FTH worlds survive even at 30× baseline shock frequency

**Conclusion:** Deploy aggressively in stable markets. Shift to conservative reserves as external shock frequency increases. The crossover point is quantifiable — this is the type of non-obvious result that justifies simulation infrastructure.

---

## How to Reproduce

### Prerequisites

- Rust toolchain (edition 2021)
- Windows, macOS, or Linux

### Build and Run

```bash
git clone https://github.com/FTHTrading/AI.git
cd AI
cargo build --release --bin run_experiments
```

On Windows:

```powershell
.\target\release\run_experiments.exe
```

On Linux/macOS:

```bash
./target/release/run_experiments
```

Runtime: approximately 49 seconds on a modern machine (release build).

### Output Location

Results appear in the repository root under `experiments/`:

```
experiments/
  entropy_sweep/
    entropy_sweep_manifest.json
    entropy_sweep_data.csv
    entropy_sweep_report.txt
  catastrophe_resilience/
    catastrophe_resilience_manifest.json
    catastrophe_resilience_data.csv
    catastrophe_resilience_report.txt
  inequality_threshold/
    inequality_threshold_manifest.json
    inequality_threshold_data.csv
    inequality_threshold_report.txt
  treasury_stability/
    treasury_stability_manifest.json
    treasury_stability_data.csv
    treasury_stability_report.txt
  fth_reserve_calm/
    fth_reserve_calm_manifest.json
    fth_reserve_calm_data.csv
    fth_reserve_calm_report.txt
  fth_reserve_moderate/
    fth_reserve_moderate_manifest.json
    fth_reserve_moderate_data.csv
    fth_reserve_moderate_report.txt
  fth_reserve_stressed/
    fth_reserve_stressed_manifest.json
    fth_reserve_stressed_data.csv
    fth_reserve_stressed_report.txt
  fth_reserve_crisis/
    fth_reserve_crisis_manifest.json
    fth_reserve_crisis_data.csv
    fth_reserve_crisis_report.txt
```

---

## How to Verify Integrity

### Step 1: Compare File Hashes

On Windows PowerShell:

```powershell
Get-FileHash -Algorithm SHA256 -Path "02_EXPERIMENTS\fth_reserve_crisis\fth_reserve_crisis_data.csv"
```

Compare the output hash against the corresponding line in `03_INTEGRITY/sha256sums.txt`.

### Step 2: Verify Experiment Reproducibility

Each manifest JSON contains:

- `base_seed`: The deterministic seed used (all experiments use `20260222`)
- `result_hash`: SHA-256 hash of the aggregated experiment output

To verify: run the experiments from the same seed and confirm the result hash matches the manifest.

### Step 3: Full Test Suite

```bash
cargo test --workspace
```

Expected: 349 tests passing, 0 failures.

---

## How to Replay Individual Trials

Each experiment manifest contains the seed derivation formula:

```
trial_seed = base_seed + (step_index × 1000) + run_index
```

Any individual trial can be replayed exactly by constructing its seed and running a single-world simulation with the corresponding parameters. The `genesis-replay` crate provides deterministic replay verification from any checkpoint.

---

## Deterministic Guarantee

All simulations use deterministic random number generation seeded from `20260222`. Given the same seed, parameters, and code version, the engine produces byte-identical outputs. This is verified via SHA-256 manifest hashing at the experiment level and dual-chain cryptographic anchoring (SHA-256 + BLAKE3) at the per-epoch level.
