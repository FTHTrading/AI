# Genesis Protocol — Bundle Manifest

**Bundle:** Genesis-Protocol-v1.0.0-Experimental-Engine
**Purpose:** IP Priority Evidence — Frozen Snapshot
**Created:** 2026-02-22
**Author:** Kevan Burns (ORCID 0009-0008-8425-939X)
**Commit:** 1955dfa900296065308be5dcd232c580e9e8ef9a

---

## Contents

```
Genesis-Protocol-v1.0.0-Experimental-Engine/
│
├── SYSTEM_STATE.md              # Engine state at freeze (stats, findings, publication record)
├── BUNDLE_MANIFEST.md           # This file — contents listing
├── README.md                    # Project README (from repo root)
│
├── crate-snapshot/
│   ├── CRATE_MANIFEST.md        # 13-crate architecture description
│   ├── Cargo.toml               # Workspace root manifest
│   ├── Cargo.lock               # Locked dependency graph
│   ├── .zenodo.json             # Zenodo metadata (DOI linkage)
│   └── CITATION.cff             # Citation metadata
│
├── experiments/
│   ├── entropy_sweep/           # 200 worlds — entropy vs. inequality
│   │   ├── entropy_sweep_data.csv
│   │   ├── entropy_sweep_manifest.json
│   │   └── entropy_sweep_report.txt
│   ├── catastrophe_resilience/  # 140 worlds — shock vs. fitness
│   │   ├── catastrophe_resilience_data.csv
│   │   ├── catastrophe_resilience_manifest.json
│   │   └── catastrophe_resilience_report.txt
│   ├── inequality_threshold/    # 160 worlds — Gini vs. redistribution
│   │   ├── inequality_threshold_data.csv
│   │   ├── inequality_threshold_manifest.json
│   │   └── inequality_threshold_report.txt
│   ├── treasury_stability/      # 180 worlds — deployment timing
│   │   ├── treasury_stability_data.csv
│   │   ├── treasury_stability_manifest.json
│   │   └── treasury_stability_report.txt
│   ├── fth_reserve_calm/        # Baseline — no shock
│   │   ├── fth_reserve_calm_data.csv
│   │   ├── fth_reserve_calm_manifest.json
│   │   └── fth_reserve_calm_report.txt
│   ├── fth_reserve_moderate/    # Moderate redemption pressure
│   │   ├── fth_reserve_moderate_data.csv
│   │   ├── fth_reserve_moderate_manifest.json
│   │   └── fth_reserve_moderate_report.txt
│   ├── fth_reserve_stressed/    # Sustained liquidity drain
│   │   ├── fth_reserve_stressed_data.csv
│   │   ├── fth_reserve_stressed_manifest.json
│   │   └── fth_reserve_stressed_report.txt
│   └── fth_reserve_crisis/      # Acute crisis + compounding shock
│       ├── fth_reserve_crisis_data.csv
│       ├── fth_reserve_crisis_manifest.json
│       └── fth_reserve_crisis_report.txt
│
├── deliverables/
│   └── genesis-experiment-pack-v3/  # Experiment Pack v3 (archival copy)
│
├── papers/
│   ├── sravan-executive-brief.md
│   ├── genesis-protocol-paper.md    # (if present)
│   └── experimental-method.md       # (if present)
│
└── integrity/
    ├── sha256sums.txt           # SHA-256 of every file in this bundle
    └── archive-hash.txt         # SHA-256 of the .tar.gz archive
```

---

## What Is NOT Included (Trade Secret)

- Source code (`crates/*/src/**`) — withheld as trade secret
- Build artifacts and binaries
- Private keys and wallet files
- Internal configuration secrets

## What IS Included (Proof of Possession)

- All experimental outputs (data, manifests, reports)
- Workspace build metadata (Cargo.toml, Cargo.lock)
- Citation and DOI metadata
- Complete crate architecture documentation
- System state with verified test counts
- SHA-256 integrity chain

---

## Legal Notice

This bundle constitutes evidence of prior art and inventorship.
It is timestamped via IPFS Content Identifier (CID) and git commit SHA.
Contents are the intellectual property of Kevan Burns / FTH Trading Inc.
All rights reserved.

---

*Genesis Protocol v1.0.0 — Bundle Manifest*
