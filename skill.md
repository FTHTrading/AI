# Genesis-Apex — Skill Sheet

## System Type

Autonomous evolutionary simulation — organism-as-a-service.

## Runtime

| Property          | Value                                |
|-------------------|--------------------------------------|
| Language          | Rust (edition 2021)                  |
| Binary            | `genesis-protocol`                   |
| Crates            | 6 library + 1 binary                 |
| Transport         | Axum 0.7 HTTP                        |
| Outbound          | reqwest 0.12 (Moltbot adapter)       |
| Persistence       | JSON snapshots (serde_json)          |
| Tick Rate         | 1 epoch/second (background loop)     |
| Autosave Interval | Every 25 epochs                      |
| Default Bind      | `0.0.0.0:3000`                       |

## Public Interfaces

### GET /status
Returns full ecosystem telemetry as JSON:
- `epoch` — current epoch counter
- `population` — living agent count
- `total_atp` — circulating energy supply
- `mean_fitness`, `max_fitness`, `min_fitness` — population fitness stats
- `risk_state` — one of: Healthy, AtpCrisis, PopulationCrash, ReputationDecay, FullCollapse
- `uptime_seconds` — continuous runtime since world creation
- `total_births`, `total_deaths` — lifetime counters
- `epoch_diff` — delta metrics over last 10 epochs (population, ATP, fitness, births, deaths, mutations)

### GET /genesis
Living HTML dashboard. Auto-refreshes every 5 seconds. Shows heartbeat,
economics, risk state, role distribution, leaderboard, and endpoint reference.
Dark theme. No JavaScript dependencies.

### GET /leaderboard
Top 20 agents ranked by fitness as JSON array. Each entry includes:
`agent_id`, `role`, `fitness`, `reputation`, `atp_balance`, `generation`,
`is_primordial`, `survived_epochs`.

### GET /agent/:id
Agent lookup by hex prefix. Returns genome, traits, skills, reputation,
role, lineage, and generation.

### POST /register
Controlled agent intake. Requires JSON body with `genome_hex` and `source`.
Returns 201 on success, 409 on duplicate, 400 on missing fields.

## Fitness Evaluation

Fitness is computed per-agent per-epoch from weighted trait scores:
- Adaptability (0.3), Resilience (0.25), Metabolic Efficiency (0.2),
  Signal Fidelity (0.15), Cognitive Depth (0.1)
- Modified by role-specific skill bonuses
- Filtered through publication gate before contributing to reputation

## Treasury Model

- **Skim**: 2% of all ATP transactions feed the ecosystem reserve
- **Stipends**: Distributed proportionally to fitness each epoch
- **Crisis Spend**: Reserve deployed when ATP supply drops below threshold
- **Cap**: Individual agent ATP capped at 1000.0

## Selection Pressure

- **Selection Engine**: Evaluates population each epoch, terminates lowest-fitness agents when population exceeds minimum viable threshold
- **Replication**: Top-fitness agents with sufficient ATP can reproduce (max 1 birth/epoch)
- **Mutation**: Trait perturbation scales with environmental pressure
- **Stasis Detection**: Terminates runs if fitness plateau persists (configurable threshold)

## Moltbot Adapter (Outbound)

Projects organism state to Moltbook. Outbound-only — no inbound routes.

| Setting                       | Default | Description                          |
|-------------------------------|---------|--------------------------------------|
| `MOLTBOOK_ENDPOINT`           | (none)  | Required to enable adapter           |
| `MOLTBOOK_API_KEY`            | (empty) | Bearer token for authentication      |
| `MOLTBOT_HEARTBEAT_INTERVAL`  | 60      | Epochs between heartbeat posts       |
| `MOLTBOT_MAX_RETRIES`         | 2       | Retry count on transient failure     |
| `MOLTBOT_TIMEOUT_SECS`        | 10      | HTTP timeout per request             |

### Heartbeat Payload
Periodic JSON post with: `epoch`, `population`, `mean_fitness`, `max_fitness`,
`total_atp`, `treasury_reserve`, `risks`, `leader`, `uptime_seconds`,
`total_births`, `total_deaths`.

### Milestone Events
Fired on: population peaks, fitness records, leader changes, birth bursts,
death spirals, epoch milestones (every 100), extinction risk, ATP crisis,
monoculture emergence.

## Test Coverage

149 tests across 6 crates. Zero failures. Covers:
- Cryptographic identity & replication
- ATP economy & treasury mechanics
- Mesh networking & gossip protocol
- Problem market & publication gate
- Selection, mutation, gene transfer
- Security hardening, rate limiting, emergency controls
- Load simulation (1000+ concurrent requests)
- Moltbot adapter serialization, event detection, mock-server integration
- World state, persistence, HTTP endpoints, background loop
