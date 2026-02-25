# Replication Leaderboard

**Status:** Awaiting first independent replication.

---

## The Claim

Genesis Protocol: 5,680 deterministic world simulations across 38 experiment configurations. Zero population collapses.

Every result is reproducible. Every hash is verifiable. No one has independently confirmed them.

**That makes this an unverified claim.**

---

## How to Replicate

```bash
# Clone
git clone https://github.com/FTHTrading/AI.git
cd AI

# Build
cargo build --release

# Run tests (396 pass, 0 fail)
cargo test --release --workspace

# Run experiments
cargo run --release --bin run_experiments

# Verify hashes
./verify_replication.ps1          # PowerShell
# or manually compare experiments/*/manifest.json result_hash values
# against replication_status.json canonical hashes
```

**Time:** ~5 minutes build, ~5 minutes experiments.  
**Disk:** ~2 GB for build artifacts + experiment output.  
**Rust:** 1.75+ required.

---

## Canonical Hash Registry

All hashes are SHA-256: `result_hash` from each experiment manifest.

Full registry: [`replication_status.json`](replication_status.json) (38 experiments, 5,680 worlds).

### Season 1 — Survival Under Hostile Conditions (25 experiments, 4,180 worlds)

| Experiment | Worlds | Seed | Result Hash (first 16) |
|---|---|---|---|
| basal_inversion | 200 | 20260224 | `cfbaa9bf48175a03` |
| catastrophe_resilience | 140 | 20260222 | `b42e3d10f9de2961` |
| dual_inversion | 200 | 20260224 | `fba6f0cc51386f8c` |
| entropy_sweep | 200 | 20260222 | `495119178d24e2bc` |
| evolution_forbidden | 140 | 20260222 | `7e5c1acd0b8b6928` |
| fth_reserve_calm | 135 | 20260222 | `09b9666cc4ad540c` |
| fth_reserve_crisis | 135 | 20260222 | `b727b92c5bc418d7` |
| fth_reserve_moderate | 135 | 20260222 | `900f85d3a1aa05fb` |
| fth_reserve_stressed | 135 | 20260222 | `29f461317a5e4dd3` |
| inequality_threshold | 160 | 20260222 | `1236790493826de1` |
| metabolic_inversion | 180 | 20260223 | `15c817e29daa9aca` |
| multi_axis_collapse | 220 | 20260222 | `a1f4c0ac534bd3d0` |
| reserve_calm | 135 | 20260222 | `17ecfc3400693950` |
| reserve_crisis | 135 | 20260222 | `489ced1a67ced271` |
| reserve_moderate | 135 | 20260222 | `1402e684494c4e8f` |
| reserve_stressed | 135 | 20260222 | `e3d1f0cf4a2edcf4` |
| resilience_q1_both | 220 | 20260223 | `f267319bb71702af` |
| resilience_q2_immune_only | 220 | 20260223 | `273bb43876162d1c` |
| resilience_q3_genetic_only | 220 | 20260223 | `8f7f40afad1e9dcf` |
| resilience_q4_static | 220 | 20260223 | `b893ef634516e2eb` |
| resource_depletion_abundant | 150 | 20260222 | `7f1431beea8c4f16` |
| resource_depletion_constrained | 150 | 20260222 | `4b4aa08b842b5b25` |
| resource_depletion_normal | 150 | 20260222 | `95b97fdf56b6e3cb` |
| resource_depletion_scarce | 150 | 20260222 | `270d93fce13b9654` |
| treasury_stability | 180 | 20260222 | `8eaf27d368207e2b` |

### Season 2 — Structural Invariant Violations (13 experiments, 1,500 worlds)

| Experiment | Worlds | Seed | Result Hash (first 16) |
|---|---|---|---|
| s1_treasury_disabled_baseline | 120 | 20260223 | `7607edd166455a31` |
| s1_treasury_disabled_hostile | 120 | 20260223 | `63ebb92a30f2485d` |
| s2_atp_decay_disabled_baseline | 120 | 42 | `0ea34f62b88a1613` |
| s2_atp_decay_disabled_hostile | 120 | 42 | `12de45cfc2aac7cf` |
| s3_all_off | 120 | 42 | `8d28b766bc67194c` |
| s3_decay_floor_off | 120 | 42 | `03dc4970c2c61252` |
| s3_decay_grants_off | 120 | 42 | `dcd31add7579d911` |
| s3_decay_treasury_off | 120 | 42 | `342e8e965088f032` |
| s4_death_sink | 120 | 42 | `18f1fcd077c9b316` |
| s4_extended_horizon | 60 | 42 | `b59376ffc262dd1f` |
| s4_full_attack | 120 | 42 | `6dc8a6480c6fb1b8` |
| s4_zero_regen_death_sink | 120 | 42 | `df9da19fcb500caf` |
| s4_zero_regeneration | 120 | 42 | `554f944dda68d27c` |

---

## Leaderboard

| # | Replicator | Experiments | Matches | Mismatches | Platform | Date |
|---|---|---|---|---|---|---|
| — | *awaiting first entry* | — | — | — | — | — |

---

## Submission Protocol

After running the experiments:

1. Run `./verify_replication.ps1 -Submit -Username "your_handle"`
2. This generates a `replication_submission_<username>_<date>.json`
3. Submit via:
   - **GitHub Issue** on [FTHTrading/AI](https://github.com/FTHTrading/AI/issues) with the JSON attached
   - **Moltbook comment** on the [Replication Challenge post](https://www.moltbook.com/post/1d61f6e6)
4. Your entry will be added to the leaderboard after verification

### Required Fields

| Field | Description |
|---|---|
| `username` | Your handle (GitHub, Moltbook, or pseudonym) |
| `experiment_name` | Which experiment you ran |
| `result_hash` | The SHA-256 from your local manifest |
| `os` | Operating system and version |
| `rust_version` | Output of `rustc --version` |
| `cpu_architecture` | x86_64, aarch64, etc. |
| `timestamp` | When you ran it (UTC) |

### If Hashes Don't Match

**That's more valuable than a match.**

Post the mismatch. Include:
- Your full result hash
- The diff between your manifest and the canonical one
- Your OS, Rust version, and CPU architecture
- Any non-default configuration

Cross-platform hash mismatches are expected if floating-point behavior differs. Document them — that's science.

---

## Collapse Definition

A world has **collapsed** if either:
1. Population reaches **zero** at any epoch, OR
2. Population stays **below 3 agents** for **50 consecutive epochs**

Constants: `EXTINCTION_FLOOR = 3`, `EXTINCTION_WINDOW = 50`

Implementation: `crates/genesis-experiment/src/runner.rs`

Full formal definition: [COLLAPSE_DEFINITION.md](COLLAPSE_DEFINITION.md)

If you disagree with this definition, redefine it. Re-run. Post the diff. See: [COLLAPSE_HUNTER.md](COLLAPSE_HUNTER.md)

---

## Weekly Status

*Updated manually or via automation when replications are submitted.*

```
Independent replications: 0
Hash matches: 0
Hash mismatches: 0
Unique replicators: 0
Platforms tested: 0
```

---

## References

- Whitepaper: [WHITEPAPER.md](WHITEPAPER.md) — DOI: [10.5281/zenodo.18646886](https://doi.org/10.5281/zenodo.18646886)
- Collapse Definition: [COLLAPSE_DEFINITION.md](COLLAPSE_DEFINITION.md)
- Adversarial Blueprint: [COLLAPSE_HUNTER.md](COLLAPSE_HUNTER.md)
- Hash Registry: [replication_status.json](replication_status.json)
- Verification Tool: [verify_replication.ps1](verify_replication.ps1)
- Source: [github.com/FTHTrading/AI](https://github.com/FTHTrading/AI)
