# Genesis Experiment Pack v1 — Reproduction Guide

## What This Pack Contains

| Item | Description |
|---|---|
| `01_SRAVAN_EXECUTIVE_BRIEF.md` | Decision-grade summary of platform capabilities and experimental results |
| `02_EXPERIMENTS/` | Three flagship experiment outputs (manifest + data + report per experiment) |
| `03_INTEGRITY/sha256sums.txt` | SHA-256 hash of every file in this pack |
| `04_LICENSE_NOTES.md` | Licensing and attribution |

### Experiments Included

| Experiment | Worlds | Epochs | Independent Variable |
|---|---|---|---|
| Entropy Sweep | 200 | 100,000 | Metabolic cost of existence |
| Catastrophe Resilience | 140 | 70,000 | Catastrophe probability per epoch |
| Inequality Threshold | 160 | 80,000 | Gini threshold for wealth tax activation |

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

Runtime: approximately 24 seconds on a modern machine (release build).

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
```

---

## How to Verify Integrity

### Step 1: Compare File Hashes

On Windows PowerShell:

```powershell
Get-FileHash -Algorithm SHA256 -Path "02_EXPERIMENTS\entropy_sweep\entropy_sweep_data.csv"
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

Expected: 339 tests passing, 0 failures.

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
