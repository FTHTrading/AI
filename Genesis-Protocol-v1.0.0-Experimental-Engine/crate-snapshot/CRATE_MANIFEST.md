# Genesis Protocol — Crate Manifest

**Bundle Version:** v1.0.0
**Commit:** 1955dfa900296065308be5dcd232c580e9e8ef9a
**Workspace Type:** Rust (Cargo workspace)
**Edition:** 2021

---

## Workspace Layout

```
genesis-protocol/
├── Cargo.toml          (workspace root)
├── Cargo.lock          (locked dependency graph)
├── crates/
│   ├── apostle/
│   ├── ecosystem/
│   ├── evolution/
│   ├── gateway/
│   ├── genesis-anchor/
│   ├── genesis-dna/
│   ├── genesis-econometrics/
│   ├── genesis-experiment/
│   ├── genesis-federation/
│   ├── genesis-homeostasis/
│   ├── genesis-multiverse/
│   ├── genesis-replay/
│   └── metabolism/
├── experiments/        (8 completed experiments)
├── deliverables/       (Pack v3)
└── papers/             (3 technical papers)
```

---

## Crate Descriptions

### 1. `apostle`
**Domain:** Agent lifecycle & behavioral rules
**Tests:** 9
Agent definition, birth/death cycles, trait expression, decision-making behavior.
Each agent ("apostle") is a self-contained actor with heritable traits and metabolic needs.

### 2. `ecosystem`
**Domain:** World environment & resource dynamics
**Tests:** 28
Resource pools, spatial structure, environmental shocks, carrying capacity.
The ecosystem provides the substrate in which agents live, compete, and die.

### 3. `evolution`
**Domain:** Evolutionary selection & reproduction
**Tests:** 19
Fitness evaluation, selection pressure, mutation, crossover, generational advancement.
Drives adaptation: agents that survive contribute traits to succeeding cohorts.

### 4. `gateway`
**Domain:** External interface & API layer
**Tests:** 6
Experiment launch, configuration ingestion, output serialization.
The boundary between the simulation engine and the outside world.

### 5. `genesis-anchor`
**Domain:** Dual-chain cryptographic anchoring
**Tests:** 22
XRPL memo anchoring, Ethereum event-log anchoring, SHA-256 digest binding.
Ensures every simulation run is tied to an immutable on-chain record.

### 6. `genesis-dna`
**Domain:** Agent trait encoding & inheritance
**Tests:** 13
Genetic representation, trait vectors, heritability coefficients, mutation rates.
The "genome" layer — how traits propagate across generations.

### 7. `genesis-econometrics`
**Domain:** Statistical analysis & policy extraction
**Tests:** 20
Gini coefficient, survival rate, ATP distribution, treasury efficiency metrics.
Post-simulation analytics that extract policy-relevant signals from raw epoch data.

### 8. `genesis-experiment`
**Domain:** Experiment orchestration & parameter sweeps
**Tests:** 47
Multi-world runs, parameter grids, manifest generation, CSV output.
The experiment engine that coordinates large-scale sweeps across parameter space.

### 9. `genesis-federation`
**Domain:** Multi-world coordination
**Tests:** 7
Cross-world signaling, federation topology, shared state channels.
Allows multiple parallel worlds to exchange signals during simulation.

### 10. `genesis-homeostasis`
**Domain:** Treasury & redistribution balance
**Tests:** 19
Treasury accumulation, redistribution triggers, balance thresholds.
Models how pooled resources are deployed back into the ecosystem.

### 11. `genesis-multiverse`
**Domain:** Fork, diverge, merge architecture
**Tests:** 37
World forking, divergence tracking, convergence/merge logic, variant comparison.
The multiverse layer: run many branching realities, then compare outcomes.

### 12. `genesis-replay`
**Domain:** Deterministic replay & verification
**Tests:** 7
Bit-exact re-execution, state hash verification, divergence detection.
Guarantees any simulation can be replayed to produce identical results.

### 13. `metabolism`
**Domain:** ATP economy & thermodynamic engine
**Tests:** 105
Energy units (ATP), metabolic cost/gain, thermodynamic constraints, resource flow.
The largest crate — models the economy as a thermodynamic energy system.

---

## Novel Integration

These 13 crates form a single integrated system. The combination of:

- **Agent-based simulation** (apostle, ecosystem, evolution)
- **Deterministic replay** (genesis-replay)
- **Cryptographic anchoring** (genesis-anchor)
- **Adaptive cortex** (genesis-homeostasis)
- **Multiverse fork/merge** (genesis-multiverse)
- **Policy extraction** (genesis-econometrics)

…as one unified engine has no known precedent in published systems.

---

## Note

This manifest covers the crate architecture only.
Source code is NOT included in this frozen bundle (selective trade-secret protection).
Cargo.toml and Cargo.lock are included to prove dependency structure and build reproducibility.

---

*Genesis Protocol v1.0.0 — Crate Manifest*
