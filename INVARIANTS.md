# System Invariants — Genesis Protocol

**Protocol:** SOP-1 (Software Organism Protocol v1)  
**Version:** 1.0  
**Date:** February 21, 2026

All 14 invariants must hold at all times during system operation. Violation of any invariant indicates a bug.

---

## Ecology (E-1 through E-4)

### E-1: Logistic Resource Regeneration
Resource pools regenerate via logistic growth: `R(t+1) = R(t) + r * R(t) * (1 - R(t)/K)`.  
**Verification:** Inspect `ResourcePool::regenerate()` in `crates/gateway/src/world.rs`.

### E-2: Seasonal Modulation
Seasonal modulation follows a sinusoidal cycle with configurable amplitude and period.  
Effective capacity: `K_eff = K * (1 + A * sin(2π * epoch / T))`.  
**Verification:** Inspect `Environment::tick()` in `crates/gateway/src/world.rs`.

### E-3: Proportional Extraction
Resource extraction is proportional to agent fitness and niche skill. Winner-take-all dynamics are prohibited.  
**Verification:** Inspect `run_epoch()` Step 2 (Resource Extraction) in `crates/gateway/src/world.rs`.

### E-4: Density-Dependent Foraging
Extraction per agent decreases with niche crowding via factor `1 / (1 + n_niche * α)`.  
Cross-niche competition coefficient β further modulates extraction.  
**Verification:** Inspect density_factor and cross_penalty computations in `run_epoch()`.

---

## Metabolism (M-1 through M-3)

### M-1: Non-Negative Balances
ATP balance cannot go negative. The metabolic tick clamps deductions at zero and returns actual amount consumed.  
**Verification:** Inspect `metabolic_tick()` in `crates/metabolism/src/atp.rs`.

### M-2: Computed Supply
Total ATP supply is the sum of all agent balances at query time. No running counter is maintained.  
**Verification:** Inspect `total_supply()` in `crates/metabolism/src/ledger.rs`.

### M-3: Atomic Replication Cost
Replication costs are checked and deducted in a single operation. Insufficient balance prevents replication.  
**Verification:** Inspect replication block in `run_epoch()` in `crates/gateway/src/world.rs`.

---

## Selection (S-1 through S-3)

### S-1: Dynamic Population Cap
Population cap is derived from total resource capacity: `K_pop = total_capacity / 15`, clamped to `[10, 500]`.  
**Verification:** Inspect dynamic_cap computation in `run_epoch()`.

### S-2: Maturation Guard
Selection pressure respects a maturation period. Agents younger than `MATURATION_EPOCHS` are exempt from culling.  
**Verification:** Inspect maturation check in replication block of `run_epoch()`.

### S-3: Stasis Tolerance
Agents in stasis (zero ATP) are given a tolerance period (`STASIS_TOLERANCE` epochs) before death.  
**Verification:** Inspect `SelectionEngine::new()` in `crates/evolution/src/selection.rs`.

---

## Genome (G-1 through G-2)

### G-1: Cryptographic Primordial Diversity
Primordial genomes are derived from SHA-256 hashes of unique seed strings, ensuring genuine genetic diversity.  
**Verification:** Inspect `spawn_primordials()` in `crates/gateway/src/world.rs`.

### G-2: Environmentally-Responsive Mutation
Mutation pressure is modulated by seasonal stress. Higher environmental pressure increases mutation rate.  
**Verification:** Inspect mutation pressure computation in `run_epoch()`.

---

## Provenance (P-1 through P-2)

### P-1: Deterministic Edition Root
Edition root = Merkle(crate_roots), deterministically recomputable from source files.  
**Verification:** Run `powershell -ExecutionPolicy Bypass -File scripts/merkle.ps1` and compare `dist/merkle.json`.

### P-2: Complete Manifest Coverage
All source files have SHA-256 entries in `dist/manifest.json`. No source file may exist without a corresponding hash.  
**Verification:** Compare `dist/manifest.json` file list against `crates/**/src/*.rs`.
