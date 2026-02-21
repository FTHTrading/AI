# SOP-1: Software Organism Protocol v1

**Status:** Draft  
**Version:** 1.0  
**Author:** Kevan Burns (ORCID: 0009-0008-8425-939X)  
**Date:** February 21, 2026  
**Reference Implementation:** [Genesis Protocol](https://github.com/FTHTrading/AI)

---

## Abstract

SOP-1 defines a reproducible pattern for constructing self-sustaining software organisms with cryptographic provenance. It specifies the ecological model, energy economics, selection pressure, and integrity verification requirements that any conforming implementation must satisfy. The protocol extends the provenance methodology established in [LPS-1](https://github.com/FTHTrading/2500-donkeys/blob/main/LPS-1.md) (Literary Protocol Standard) to executable software systems.

---

## I. Protocol Layers

SOP-1 conforming systems must implement five functional layers:

| Layer | Name | Responsibility | Genesis Protocol Crate |
|-------|------|---------------|----------------------|
| 1 | **Genome** | Agent identity, traits, fitness metrics | `genesis-dna` |
| 2 | **Metabolism** | Energy accounting, proof-of-work, treasury | `metabolism` |
| 3 | **Ecosystem** | Communication mesh, problem markets, telemetry | `ecosystem` |
| 4 | **Evolution** | Mutation engine, selection pressure, gene transfer | `evolution` |
| 5 | **Gateway** | World state, epoch engine, external interfaces | `gateway` |

An optional sixth layer (**Apostle**) handles outbound intelligence — converting system state into external communications.

---

## II. State Machine

Every SOP-1 organism progresses through a linear lifecycle:

```
GENESIS → SPAWNING → RUNNING → STABLE → ADAPTING → RUNNING → ...
                                  ↓
                              EXTINCTION
```

| State | Entry Condition | Exit Condition |
|-------|----------------|----------------|
| GENESIS | First epoch | Primordial agents spawned |
| SPAWNING | Primordials created | All agents registered in ledger |
| RUNNING | Epoch loop active | Population stable for N epochs |
| STABLE | Mean fitness > threshold | Environmental perturbation |
| ADAPTING | Catastrophe or boom event | Population re-stabilizes |
| EXTINCTION | Population reaches zero | Terminal (organism dies) |

---

## III. Ecological Model Requirements

### III.1 Resource Pools

Each agent role (niche) MUST have an independent resource pool with:
- **Capacity** (K): Maximum resource level
- **Regeneration rate** (r): Per-epoch growth coefficient
- **Logistic growth**: `R(t+1) = R(t) + r * R(t) * (1 - R(t)/K)`

### III.2 Seasonal Oscillation

The environment MUST exhibit periodic variation:
- **Modulation**: Sinusoidal with configurable period and amplitude
- **Effective capacity**: `K_eff = K * (1 + A * sin(2π * epoch / period))`
- Where A is amplitude (recommended: 0.20-0.30) and period is cycle length

### III.3 Stochastic Events

The system MUST include random perturbation:
- **Catastrophe**: Low-probability event (1-3% per epoch) that reduces resources
- **Boom**: Low-probability event (3-7% per epoch) that increases resources
- Event severity MUST be bounded and time-limited

### III.4 Resource Extraction

Extraction MUST be proportional, never winner-take-all:
- Each agent extracts based on: `fitness * niche_skill * pool_availability * density_factor`
- Density factor MUST decrease with niche crowding: `1 / (1 + n_niche * α)`
- Cross-niche competition coefficient (β) reduces extraction when population is dense

### III.5 Population Dynamics

- Population cap MUST be dynamic, derived from total resource capacity
- Carrying capacity formula: `K_pop = total_capacity / c`, clamped to `[min, max]`
- Births MUST be rate-limited per epoch
- Maturation period MUST prevent immediate reproduction

---

## IV. Energy Economics Requirements

### IV.1 ATP (Adenosine Triphosphate) Accounting

| Parameter | Constraint | Reference Range |
|-----------|-----------|-----------------|
| Basal metabolic cost | Fixed per epoch | 0.10-0.25 ATP |
| Replication cost | Fixed, achievable in 20-50 epochs | 15-50 ATP |
| Primordial grant | Sufficient for 100+ epochs survival | 30-80 ATP |
| Child grant | Minimal starter, not self-sufficient | 5-15 ATP |

### IV.2 Invariants

- **M-1**: ATP balance MUST NOT go negative — all deductions clamp at zero
- **M-2**: Total supply MUST equal sum of all agent balances (computed, never tracked separately)
- **M-3**: Replication cost MUST be deducted atomically from parent

---

## V. Selection Pressure Requirements

### V.1 Natural Selection

- Bottom N% of population by fitness MAY be culled each epoch
- Stasis (zero-balance) agents MUST be given a tolerance period before death
- Recommended stasis tolerance: 5-12 epochs

### V.2 Reproduction

- Fitness threshold MUST be met before replication is allowed
- Maturation period MUST elapse before an agent can reproduce
- Child genome MUST be derived from parent with mutation
- Maximum births per epoch MUST be bounded

---

## VI. Genome Requirements

### VI.1 Diversity

- Primordial genomes MUST be cryptographically derived (SHA-256 or equivalent)
- Genomes MUST NOT be generated from trivially predictable seeds
- Initial population MUST exhibit measurable trait variance

### VI.2 Mutation

- Mutation rate MUST be environmentally responsive
- Higher seasonal stress SHOULD increase mutation pressure
- Mutations MUST preserve genome validity

---

## VII. Provenance Requirements

### VII.1 Integrity Verification

SOP-1 systems MUST provide:

| Layer | Mechanism | Verification Command |
|-------|-----------|---------------------|
| 1 | Per-source-file SHA-256 hashes | `scripts/merkle.ps1` |
| 2 | Per-crate Merkle tree roots | Rebuild from source |
| 3 | Edition root (Merkle of crate roots) | `dist/merkle.json` |
| 4 | Git commit history | `git log` |
| 5 | On-chain anchoring (optional) | Polygonscan |

### VII.2 Merkle Tree Structure

```
┌──────────────────────────────────────────────────────┐
│                    EDITION ROOT                       │
│   Merkle(genesis-dna ‖ metabolism ‖ ecosystem ‖      │
│          evolution ‖ apostle ‖ gateway ‖ tests)      │
├──────────┬──────────┬──────────┬──────────┬──────────┤
│ genesis  │ metab    │ eco      │ evol     │ gateway  │
│ -dna     │ olism    │ system   │ ution    │          │
│ Root     │ Root     │ Root     │ Root     │ Root     │
├──────────┼──────────┼──────────┼──────────┼──────────┤
│ genome ██│ atp   ██ │ mesh  ██ │ mut   ██ │ world ██ │
│ traits ██│ ledger██ │ msgs  ██ │ sel   ██ │ server██ │
│ roles  ██│ proof ██ │ mkt   ██ │ gene  ██ │ shield██ │
│ ...    ██│ ...   ██ │ ...   ██ │ ...   ██ │ ...   ██ │
└──────────┴──────────┴──────────┴──────────┴──────────┘
```

### VII.3 On-Chain Anchoring

The edition root MAY be anchored on Polygon mainnet via:
- **LiteraryAnchor** contract: `anchorEdition(editionRoot, ipfsCID)`
- Contract address: `0x97f456300817eaE3B40E235857b856dfFE8bba90`
- This binds the software state to an immutable on-chain record
- Same contract infrastructure used by 2500-donkeys (DOI: 10.5281/zenodo.18646886)

### VII.4 Academic Identity

SOP-1 systems SHOULD provide:
- `CITATION.cff` — Machine-readable citation metadata
- `.zenodo.json` — DOI registration metadata
- ORCID binding — Author identity linked to academic record

---

## VIII. System Invariants

All 14 invariants MUST hold at all times:

| ID | Domain | Invariant |
|----|--------|-----------|
| E-1 | Ecology | Resource pools regenerate via logistic growth |
| E-2 | Ecology | Seasonal modulation follows sinusoidal cycle with configurable amplitude |
| E-3 | Ecology | Resource extraction is proportional to fitness, never winner-take-all |
| E-4 | Ecology | Density-dependent foraging: extraction decreases with niche crowding |
| M-1 | Metabolism | ATP balance cannot go negative: metabolic tick clamps at zero |
| M-2 | Metabolism | Total ATP supply is sum of all agent balances (computed, not tracked) |
| M-3 | Metabolism | Replication costs are deducted atomically from parent balance |
| S-1 | Selection | Population cap is dynamic: total_capacity / 15, clamped [10, 500] |
| S-2 | Selection | Selection pressure respects maturation period before culling |
| S-3 | Selection | Stasis tolerance prevents premature extinction of viable agents |
| G-1 | Genome | Primordial genomes are SHA-256 derived for genuine diversity |
| G-2 | Genome | Mutation pressure is modulated by seasonal environmental stress |
| P-1 | Provenance | Edition root = Merkle(crate_roots), deterministically recomputable |
| P-2 | Provenance | All source files have SHA-256 entries in manifest.json |

---

## IX. Verification

Any third party can verify a SOP-1 system:

```powershell
# 1. Clone the repository
git clone https://github.com/FTHTrading/AI.git
cd AI

# 2. Rebuild Merkle trees from source
powershell -ExecutionPolicy Bypass -File scripts/merkle.ps1

# 3. Compare edition root against dist/provenance.json

# 4. Run the test suite (143+ tests)
cargo test --workspace

# 5. (Optional) Verify on-chain state via Polygonscan
```

---

## X. Relationship to LPS-1

| Aspect | LPS-1 (Literary) | SOP-1 (Software Organism) |
|--------|-------------------|---------------------------|
| Content type | Markdown manuscripts | Rust source code |
| Merkle granularity | Per-chapter blocks | Per-crate source files |
| Hash algorithm | SHA-256 | SHA-256 |
| On-chain contract | LiteraryAnchor | LiteraryAnchor (shared) |
| State machine | DRAFT → PUBLISHED | GENESIS → RUNNING |
| Verification | `npm run lps:verify` | `scripts/merkle.ps1` |
| Academic identity | ORCID + DOI | ORCID + DOI (shared) |

Both protocols share:
- The same author identity (ORCID: 0009-0008-8425-939X)
- The same on-chain infrastructure (Polygon LiteraryAnchor)
- The same Merkle tree methodology
- The same provenance evidence chain structure

---

## XI. License

- **Protocol specification (SOP-1):** CC-BY-4.0
- **Reference implementation (Genesis Protocol):** MIT
- **Research paper:** CC-BY-4.0

---

*"Systems that can prove their own integrity don't need trust."*

Built with Rust. Verified with SHA-256. Anchored on Polygon.  
DOI: 10.5281/zenodo.18646886 · ORCID: 0009-0008-8425-939X
