// Server — Axum HTTP Gateway (Hardened)
//
// Exposes Genesis Protocol as REST endpoints:
//   POST /register    — controlled external agent entry
//   GET  /agent/:id   — read-only agent info
//   GET  /status      — ecosystem telemetry (JSON)
//   GET  /leaderboard — top agents ranked by fitness
//   GET  /genesis     — human-readable HTML dashboard
//   GET  /stream      — SSE real-time state stream
//   GET  /observatory — Three.js "Circle of Life" frontend
//
// Defense layers (via Shield module):
//   - Per-IP rate limiting (read / write split)
//   - Emergency mode controls (lockdown, intake disable, treasury freeze)
//   - Security headers on every response
//   - Request body size limits
//   - Input validation (length, charset, injection blocking)
//   - No panics in handlers — all unwraps eliminated
//
// No direct ledger, mutation, or replication manipulation.
// All evolutionary flows pass through World::run_epoch().

use std::collections::HashMap;
use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, State, DefaultBodyLimit},
    http::StatusCode,
    middleware,
    response::{
        Html, IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::Serialize;
use tokio_stream::StreamExt as _;
use tower_http::cors::{CorsLayer, Any};

use crate::shield::{
    self, EmergencyControls, SharedControls,
    rate_limit_middleware, security_headers_middleware, emergency_middleware,
};
use crate::world::{RegistrationRequest, SharedWorld, EpochDiff};

/// Status response for GET /status.
#[derive(Serialize)]
pub struct StatusResponse {
    pub epoch: u64,
    pub population: usize,
    pub avg_fitness: f64,
    pub total_atp: f64,
    pub role_distribution: HashMap<String, usize>,
    pub treasury_balance: f64,
    pub treasury_collected: f64,
    pub treasury_distributed: f64,
    pub market_solved: u64,
    pub market_rewarded: f64,
    pub risks: Vec<String>,
    pub uptime_seconds: i64,
    pub total_births: u64,
    pub total_deaths: u64,
    pub epoch_diff: EpochDiff,
}

/// Agent info response for GET /agent/:id.
#[derive(Serialize)]
pub struct AgentResponse {
    pub agent_id: String,
    pub role: String,
    pub fitness: f64,
    pub reputation: f64,
    pub atp_balance: f64,
    pub generation: u64,
    pub is_primordial: bool,
    pub skills: SkillsResponse,
}

#[derive(Serialize)]
pub struct SkillsResponse {
    pub compute: f64,
    pub optimization: f64,
    pub communication: f64,
    pub cooperation: f64,
}

/// Error response body.
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ── SSE stream payload ──────────────────────────────────────────────

/// A single agent's state for the SSE stream.
#[derive(Serialize, Clone)]
pub struct SseAgent {
    pub id: String,
    pub role: String,
    pub fitness: f64,
    pub reputation: f64,
    pub atp: f64,
    pub generation: u64,
    pub is_primordial: bool,
}

/// Full organism state pushed once per SSE tick.
#[derive(Serialize, Clone)]
pub struct SseFrame {
    pub epoch: u64,
    pub population: usize,
    pub avg_fitness: f64,
    pub total_atp: f64,
    pub treasury_reserve: f64,
    pub treasury_collected: f64,
    pub treasury_distributed: f64,
    pub total_births: u64,
    pub total_deaths: u64,
    pub roles: HashMap<String, usize>,
    pub risks: Vec<String>,
    pub agents: Vec<SseAgent>,
    pub epoch_diff: EpochDiff,
    pub uptime_seconds: i64,
}

/// Build the Axum router with all endpoints and defense layers.
pub fn build_router(world: SharedWorld) -> Router {
    build_router_with_controls(world, default_controls())
}

/// Build router with explicit emergency controls (for testing / override).
pub fn build_router_with_controls(world: SharedWorld, controls: SharedControls) -> Router {
    let read_rl = shield::read_limiter();
    let write_rl = shield::write_limiter();

    // Read-only routes with read rate limiter
    let read_routes = Router::new()
        .route("/status", get(get_status))
        .route("/agent/:id", get(get_agent))
        .route("/leaderboard", get(get_leaderboard))
        .route("/genesis", get(get_genesis_dashboard))
        .route_layer(middleware::from_fn_with_state(
            read_rl.clone(),
            rate_limit_middleware,
        ))
        .with_state(world.clone());

    // Mutation routes with strict write rate limiter
    let write_routes = Router::new()
        .route("/register", post(post_register))
        .route_layer(middleware::from_fn_with_state(
            write_rl.clone(),
            rate_limit_middleware,
        ))
        .with_state(world.clone());

    // SSE + frontend routes (no rate limiting — long-lived connections)
    let stream_routes = Router::new()
        .route("/stream", get(get_sse_stream))
        .route("/observatory", get(get_observatory))
        .with_state(world);

    // CORS layer: allow the observatory to connect from any origin
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(read_routes)
        .merge(write_routes)
        .merge(stream_routes)
        // Emergency controls applied to all routes
        .route_layer(middleware::from_fn_with_state(
            controls,
            emergency_middleware,
        ))
        // Security headers on every response
        .route_layer(middleware::from_fn(security_headers_middleware))
        // Request body size limit (32 KB)
        .layer(DefaultBodyLimit::max(shield::MAX_BODY_SIZE))
        // CORS
        .layer(cors)
}

/// Default emergency controls (from env vars or Normal mode).
pub fn default_controls() -> SharedControls {
    std::sync::Arc::new(std::sync::Mutex::new(EmergencyControls::from_env()))
}

/// Start the Axum HTTP server. This blocks until shutdown.
pub async fn start_server(world: SharedWorld, bind_addr: &str) {
    let controls = default_controls();
    let ctrl = controls.lock().unwrap();
    tracing::info!("Shield active — mode: {:?}, intake_disabled: {}, treasury_frozen: {}",
        ctrl.mode, ctrl.intake_disabled, ctrl.treasury_frozen);
    drop(ctrl);

    let app = build_router_with_controls(world, controls);

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("Failed to bind server address");

    tracing::info!("Genesis Gateway listening on {}", bind_addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

/// GET /status — ecosystem telemetry snapshot.
///
/// Minimises mutex hold time: grabs a snapshot of needed values
/// and drops the lock before serializing.
async fn get_status(
    State(world): State<SharedWorld>,
) -> impl IntoResponse {
    let w = match world.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let status = w.telemetry();

    let mut role_distribution = HashMap::new();
    for agent in &w.agents {
        *role_distribution.entry(agent.role.label().to_string()).or_insert(0usize) += 1;
    }

    let avg_fitness = if w.agents.is_empty() {
        0.0
    } else {
        w.agents.iter().map(|a| a.fitness()).sum::<f64>() / w.agents.len() as f64
    };

    let risk_labels: Vec<String> = status.risks.iter().map(|r| {
        use ecosystem::telemetry::RiskState;
        match r {
            RiskState::Stable => "STABLE".to_string(),
            RiskState::MonocultureEmerging => "MONOCULTURE".to_string(),
            RiskState::ATPConcentrationHigh => "ATP_CONCENTRATION".to_string(),
            RiskState::ReputationDecay => "REPUTATION_DECAY".to_string(),
            RiskState::PopulationCrashRisk => "POPULATION_CRASH".to_string(),
        }
    }).collect();

    // Build response while still holding lock, but this is a fast copy
    let resp = StatusResponse {
        epoch: w.epoch,
        population: w.agents.len(),
        avg_fitness,
        total_atp: w.ledger.total_supply(),
        role_distribution,
        treasury_balance: w.treasury.reserve,
        treasury_collected: w.treasury.total_collected,
        treasury_distributed: w.treasury.total_distributed,
        market_solved: w.problem_market.total_solved,
        market_rewarded: w.problem_market.total_rewarded,
        risks: risk_labels,
        uptime_seconds: w.uptime_seconds(),
        total_births: w.total_births,
        total_deaths: w.total_deaths,
        epoch_diff: w.epoch_diff(10),
    };

    // Drop lock explicitly before serialization
    drop(w);

    (StatusCode::OK, Json(resp))
}

/// GET /agent/:id — read-only agent info by genome hex prefix.
///
/// No unwraps — returns 500 on serialization failure instead of panic.
async fn get_agent(
    State(world): State<SharedWorld>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Validate ID input: hex prefix only, max 64 chars
    if id.len() > 64 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        let err = ErrorResponse {
            error: "Invalid agent ID format (expected hex prefix, max 64 chars)".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!(err))).into_response();
    }

    let w = match world.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    match w.find_agent_by_hex(&id) {
        Some(agent) => {
            let atp = w.agent_atp(agent);
            let resp = AgentResponse {
                agent_id: agent.genome_hex()[..16].to_string(),
                role: agent.role.label().to_string(),
                fitness: agent.fitness(),
                reputation: agent.reputation.score,
                atp_balance: atp,
                generation: agent.generation,
                is_primordial: agent.is_primordial,
                skills: SkillsResponse {
                    compute: agent.skills.compute,
                    optimization: agent.skills.optimization,
                    communication: agent.skills.communication,
                    cooperation: agent.skills.cooperation,
                },
            };
            match serde_json::to_value(&resp) {
                Ok(val) => (StatusCode::OK, Json(val)).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "Internal serialization error"
                }))).into_response(),
            }
        }
        None => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Agent not found"
            }))).into_response()
        }
    }
}

/// POST /register — controlled external agent entry.
///
/// Validates input through Shield layer before touching world state.
/// No unwraps — all error paths return structured JSON.
async fn post_register(
    State(world): State<SharedWorld>,
    Json(req): Json<RegistrationRequest>,
) -> impl IntoResponse {
    // Shield-layer validation: length, charset, injection blocking
    if let Err(msg) = shield::validate_registration(&req.external_id, &req.public_key) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": msg
        }))).into_response();
    }

    let mut w = match world.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    match w.register_external(&req) {
        Ok(result) => {
            match serde_json::to_value(&result) {
                Ok(val) => (StatusCode::CREATED, Json(val)).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "Internal serialization error"
                }))).into_response(),
            }
        }
        Err(e) => {
            (StatusCode::CONFLICT, Json(serde_json::json!({
                "error": e.message
            }))).into_response()
        }
    }
}

/// GET /leaderboard — top agents ranked by fitness.
///
/// Snapshot + drop pattern: grab data, release lock, then serialize.
async fn get_leaderboard(
    State(world): State<SharedWorld>,
) -> impl IntoResponse {
    let entries = {
        let w = match world.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        w.leaderboard(20)
    }; // lock released here

    (StatusCode::OK, Json(entries))
}

/// GET /genesis — human-readable HTML dashboard.
///
/// Grabs a snapshot of all needed values under the lock, drops
/// the lock, then renders the full HTML template outside the lock.
async fn get_genesis_dashboard(
    State(world): State<SharedWorld>,
) -> impl IntoResponse {
    // --- Snapshot phase: hold lock, copy scalars/strings ---
    let (
        epoch, uptime, pop, avg_fitness, total_atp,
        treasury_reserve, treasury_collected, treasury_distributed,
        market_solved, market_rewarded, total_births, total_deaths,
        diff, role_dist, leaders, risk_html, started,
    ) = {
        let w = match world.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let uptime = w.uptime_seconds();
        let avg = if w.agents.is_empty() {
            0.0
        } else {
            w.agents.iter().map(|a| a.fitness()).sum::<f64>() / w.agents.len() as f64
        };
        let diff = w.epoch_diff(10);

        let mut rd = HashMap::new();
        for agent in &w.agents {
            *rd.entry(agent.role.label().to_string()).or_insert(0usize) += 1;
        }
        let leaders = w.leaderboard(10);
        let status = w.telemetry();

        let risk_html: String = status.risks.iter().map(|r| {
            use ecosystem::telemetry::RiskState;
            let (label, color) = match r {
                RiskState::Stable => ("STABLE", "#4CAF50"),
                RiskState::MonocultureEmerging => ("MONOCULTURE", "#FF9800"),
                RiskState::ATPConcentrationHigh => ("ATP CONCENTRATION", "#FF5722"),
                RiskState::ReputationDecay => ("REPUTATION DECAY", "#9C27B0"),
                RiskState::PopulationCrashRisk => ("POPULATION CRASH", "#F44336"),
            };
            format!("<span class=\"risk\" style=\"background:{}\">{}</span> ", color, label)
        }).collect();

        let started = w.started_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();

        (
            w.epoch, uptime, w.agents.len(), avg, w.ledger.total_supply(),
            w.treasury.reserve, w.treasury.total_collected, w.treasury.total_distributed,
            w.problem_market.total_solved, w.problem_market.total_rewarded,
            w.total_births, w.total_deaths,
            diff, rd, leaders, risk_html, started,
        )
    }; // lock released here

    // --- Render phase: no lock held, pure formatting ---
    let days = uptime / 86400;
    let hours = (uptime % 86400) / 3600;
    let mins = (uptime % 3600) / 60;
    let secs = uptime % 60;
    let uptime_str = format!("{}d {}h {}m {}s", days, hours, mins, secs);

    let role_rows: String = role_dist.iter().map(|(role, count)| {
        format!(
            "<tr><td>{}</td><td><div class=\"bar\" style=\"width:{}px\"></div> {}</td></tr>",
            role,
            count * 20,
            count,
        )
    }).collect::<Vec<_>>().join("\n");

    let leader_rows: String = leaders.iter().enumerate().map(|(i, e)| {
        let badge = if e.is_primordial { " 🧬" } else { "" };
        format!(
            "<tr><td>{}</td><td><code>{}</code>{}</td><td>{}</td><td>{:.4}</td><td>{:.2}</td><td>{:.1}</td><td>{}</td></tr>",
            i + 1, e.agent_id, badge, e.role, e.fitness, e.reputation, e.atp_balance, e.generation
        )
    }).collect::<Vec<_>>().join("\n");

    let pop_arrow = if diff.population_delta > 0 { "▲" }
        else if diff.population_delta < 0 { "▼" }
        else { "—" };

    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="refresh" content="5">
<title>Genesis Protocol — Live Organism</title>
<style>
  * {{ margin:0; padding:0; box-sizing:border-box; }}
  body {{ background:#0a0a0a; color:#e0e0e0; font-family:'Courier New',monospace; padding:24px; }}
  h1 {{ color:#00ff88; font-size:1.6em; margin-bottom:4px; }}
  .tagline {{ color:#888; margin-bottom:24px; font-size:0.9em; }}
  .grid {{ display:grid; grid-template-columns:1fr 1fr; gap:20px; margin-bottom:24px; }}
  .card {{ background:#111; border:1px solid #222; border-radius:8px; padding:16px; }}
  .card h2 {{ color:#00ff88; font-size:1em; margin-bottom:10px; border-bottom:1px solid #222; padding-bottom:6px; }}
  .metric {{ display:flex; justify-content:space-between; padding:4px 0; }}
  .metric .label {{ color:#888; }}
  .metric .value {{ color:#fff; font-weight:bold; }}
  .epoch {{ font-size:2.4em; color:#00ff88; font-weight:bold; }}
  .uptime {{ color:#888; font-size:0.85em; }}
  .risk {{ display:inline-block; padding:2px 8px; border-radius:3px; font-size:0.75em; color:#fff; font-weight:bold; }}
  table {{ width:100%; border-collapse:collapse; font-size:0.85em; }}
  th {{ text-align:left; color:#00ff88; padding:6px 8px; border-bottom:1px solid #333; }}
  td {{ padding:5px 8px; border-bottom:1px solid #1a1a1a; }}
  code {{ color:#00ff88; }}
  .bar {{ display:inline-block; height:14px; background:#00ff88; border-radius:2px; vertical-align:middle; }}
  .delta {{ font-size:0.8em; }}
  .delta.positive {{ color:#4CAF50; }}
  .delta.negative {{ color:#F44336; }}
  .delta.neutral {{ color:#666; }}
  .footer {{ margin-top:24px; color:#444; font-size:0.75em; text-align:center; }}
  @media(max-width:700px) {{ .grid {{ grid-template-columns:1fr; }} }}
</style>
</head>
<body>
<h1>🧬 GENESIS PROTOCOL</h1>
<div class="tagline">A sovereign digital organism evolving under real economic pressure.</div>

<div class="grid">
  <div class="card">
    <h2>HEARTBEAT</h2>
    <div class="epoch">{epoch}</div>
    <div class="uptime">uptime: {uptime}</div>
    <div style="margin-top:10px">
      <div class="metric"><span class="label">Population</span><span class="value">{pop} {pop_arrow}</span></div>
      <div class="metric"><span class="label">Avg Fitness</span><span class="value">{fitness:.5}</span></div>
      <div class="metric"><span class="label">ATP Supply</span><span class="value">{atp:.1}</span></div>
    </div>
  </div>
  <div class="card">
    <h2>ECONOMICS</h2>
    <div class="metric"><span class="label">Treasury Reserve</span><span class="value">{treasury:.2}</span></div>
    <div class="metric"><span class="label">Total Collected</span><span class="value">{collected:.2}</span></div>
    <div class="metric"><span class="label">Total Distributed</span><span class="value">{distributed:.2}</span></div>
    <div class="metric"><span class="label">Market Problems Solved</span><span class="value">{solved}</span></div>
    <div class="metric"><span class="label">Market ATP Rewarded</span><span class="value">{rewarded:.1}</span></div>
    <div class="metric"><span class="label">Total Births / Deaths</span><span class="value">{births} / {deaths}</span></div>
  </div>
  <div class="card">
    <h2>RISK STATE</h2>
    <div style="margin:8px 0">{risks}</div>
    <h2 style="margin-top:12px">DELTA (last 10 epochs)</h2>
    <div class="metric"><span class="label">Pop Δ</span><span class="value delta {pop_class}">{pop_delta:+}</span></div>
    <div class="metric"><span class="label">ATP Δ</span><span class="value delta {atp_class}">{atp_delta:+.1}</span></div>
    <div class="metric"><span class="label">Fitness Δ</span><span class="value delta {fit_class}">{fit_delta:+.5}</span></div>
  </div>
  <div class="card">
    <h2>ROLE DISTRIBUTION</h2>
    <table>
      <tr><th>Role</th><th>Count</th></tr>
      {roles}
    </table>
  </div>
</div>

<div class="card" style="margin-bottom:20px">
  <h2>🏆 SURVIVAL LEADERBOARD (Top 10)</h2>
  <table>
    <tr><th>#</th><th>Agent</th><th>Role</th><th>Fitness</th><th>Rep</th><th>ATP</th><th>Gen</th></tr>
    {leaders}
  </table>
</div>

<div class="card">
  <h2>ENDPOINTS</h2>
  <div class="metric"><span class="label">GET /status</span><span class="value">Telemetry JSON</span></div>
  <div class="metric"><span class="label">GET /agent/:id</span><span class="value">Agent lookup</span></div>
  <div class="metric"><span class="label">GET /leaderboard</span><span class="value">Fitness rankings</span></div>
  <div class="metric"><span class="label">POST /register</span><span class="value">Phase 2+ (controlled)</span></div>
</div>

<div class="footer">
  Genesis Protocol v0.1.0 · Epoch tick: 1s · Auto-refresh: 5s · Started: {started}
</div>
</body>
</html>"#,
        epoch = epoch,
        uptime = uptime_str,
        pop = pop,
        pop_arrow = pop_arrow,
        fitness = avg_fitness,
        atp = total_atp,
        treasury = treasury_reserve,
        collected = treasury_collected,
        distributed = treasury_distributed,
        solved = market_solved,
        rewarded = market_rewarded,
        births = total_births,
        deaths = total_deaths,
        risks = risk_html,
        pop_delta = diff.population_delta,
        atp_delta = diff.atp_delta,
        fit_delta = diff.fitness_delta,
        pop_class = if diff.population_delta > 0 { "positive" } else if diff.population_delta < 0 { "negative" } else { "neutral" },
        atp_class = if diff.atp_delta > 0.0 { "positive" } else if diff.atp_delta < 0.0 { "negative" } else { "neutral" },
        fit_class = if diff.fitness_delta > 0.0 { "positive" } else if diff.fitness_delta < 0.0 { "negative" } else { "neutral" },
        roles = role_rows,
        leaders = leader_rows,
        started = started,
    );

    Html(html)
}

// ── SSE Real-time Stream ────────────────────────────────────────────

/// GET /stream — Server-Sent Events, one JSON frame per second.
///
/// Streams the full organism state for the frontend observatory.
/// No rate limiting (single long-lived connection per client).
async fn get_sse_stream(
    State(world): State<SharedWorld>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::wrappers::IntervalStream::new(
        tokio::time::interval(Duration::from_secs(1)),
    )
    .map(move |_| {
        let frame = snapshot_sse_frame(&world);
        let json = serde_json::to_string(&frame).unwrap_or_default();
        Ok(Event::default().data(json))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Build an SseFrame from the current World state.
/// Acquires and releases the mutex quickly.
fn snapshot_sse_frame(world: &SharedWorld) -> SseFrame {
    let w = match world.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let status = w.telemetry();

    let mut roles = HashMap::new();
    for agent in &w.agents {
        *roles.entry(agent.role.label().to_string()).or_insert(0usize) += 1;
    }

    let avg_fitness = if w.agents.is_empty() {
        0.0
    } else {
        w.agents.iter().map(|a| a.fitness()).sum::<f64>() / w.agents.len() as f64
    };

    let risks: Vec<String> = status.risks.iter().map(|r| {
        use ecosystem::telemetry::RiskState;
        match r {
            RiskState::Stable => "STABLE".to_string(),
            RiskState::MonocultureEmerging => "MONOCULTURE".to_string(),
            RiskState::ATPConcentrationHigh => "ATP_CONCENTRATION".to_string(),
            RiskState::ReputationDecay => "REPUTATION_DECAY".to_string(),
            RiskState::PopulationCrashRisk => "POPULATION_CRASH".to_string(),
        }
    }).collect();

    let agents: Vec<SseAgent> = w.agents.iter().map(|a| {
        SseAgent {
            id: a.genome_hex()[..16].to_string(),
            role: a.role.label().to_string(),
            fitness: a.fitness(),
            reputation: a.reputation.score,
            atp: w.agent_atp(a),
            generation: a.generation,
            is_primordial: a.is_primordial,
        }
    }).collect();

    let diff = w.epoch_diff(10);
    let uptime = w.uptime_seconds();

    SseFrame {
        epoch: w.epoch,
        population: w.agents.len(),
        avg_fitness,
        total_atp: w.ledger.total_supply(),
        treasury_reserve: w.treasury.reserve,
        treasury_collected: w.treasury.total_collected,
        treasury_distributed: w.treasury.total_distributed,
        total_births: w.total_births,
        total_deaths: w.total_deaths,
        roles,
        risks,
        agents,
        epoch_diff: diff,
        uptime_seconds: uptime,
    }
}

// ── Observatory Frontend ────────────────────────────────────────────

/// GET /observatory — serves the Three.js "Circle of Life" frontend.
async fn get_observatory() -> impl IntoResponse {
    Html(include_str!("observatory.html"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt; // for oneshot

    fn test_world() -> SharedWorld {
        Arc::new(Mutex::new(World::new()))
    }

    #[tokio::test]
    async fn test_get_status() {
        let world = test_world();
        let app = build_router(world);

        let req = Request::builder()
            .uri("/status")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["population"], 20);
        assert!(json["epoch"].as_u64().unwrap() == 0);
    }

    #[tokio::test]
    async fn test_get_agent_found() {
        let world = test_world();
        let first_hex = {
            let w = world.lock().unwrap();
            w.agents[0].genome_hex()[..8].to_string()
        };

        let app = build_router(world);
        let req = Request::builder()
            .uri(format!("/agent/{}", first_hex))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["fitness"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_get_agent_not_found() {
        let world = test_world();
        let app = build_router(world);

        let req = Request::builder()
            .uri("/agent/deadbeef99999999")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_post_register() {
        let world = test_world();
        let app = build_router(world);

        let body = serde_json::json!({
            "external_id": "moltbook:test-agent",
            "public_key": "pk_test_abc123"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(!json["agent_id"].as_str().unwrap().is_empty());
        assert_eq!(json["initial_atp"], 5.0);
    }

    #[tokio::test]
    async fn test_post_register_duplicate() {
        let world = test_world();

        let body = serde_json::json!({
            "external_id": "moltbook:dup-test",
            "public_key": "pk_test_dup"
        });

        // First registration
        let app = build_router(world.clone());
        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Second registration (duplicate)
        let app = build_router(world);
        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_post_register_empty_fields() {
        let world = test_world();
        let app = build_router(world);

        let body = serde_json::json!({
            "external_id": "",
            "public_key": "pk_test"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_leaderboard() {
        let world = test_world();
        let app = build_router(world);

        let req = Request::builder()
            .uri("/leaderboard")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 20); // All 20 agents in a fresh world
        // Verify sorted by fitness descending
        for window in arr.windows(2) {
            let a = window[0]["fitness"].as_f64().unwrap();
            let b = window[1]["fitness"].as_f64().unwrap();
            assert!(a >= b);
        }
    }

    #[tokio::test]
    async fn test_get_genesis_dashboard() {
        let world = test_world();
        let app = build_router(world);

        let req = Request::builder()
            .uri("/genesis")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("GENESIS PROTOCOL"));
        assert!(html.contains("HEARTBEAT"));
        assert!(html.contains("SURVIVAL LEADERBOARD"));
    }

    #[tokio::test]
    async fn test_status_includes_uptime_and_diff() {
        let world = test_world();
        // Run a few epochs to populate history
        {
            let mut w = world.lock().unwrap();
            for _ in 0..5 {
                w.run_epoch();
            }
        }
        let app = build_router(world);

        let req = Request::builder()
            .uri("/status")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["uptime_seconds"].as_i64().is_some());
        assert!(json["total_births"].as_u64().is_some());
        assert!(json["total_deaths"].as_u64().is_some());
        assert!(json["epoch_diff"]["window"].as_u64().is_some());
    }

    // ===== Security integration tests =====

    #[tokio::test]
    async fn test_security_headers_present() {
        let world = test_world();
        let app = build_router(world);

        let req = Request::builder()
            .uri("/status")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.headers().get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(resp.headers().get("x-frame-options").unwrap(), "DENY");
        assert_eq!(resp.headers().get("x-xss-protection").unwrap(), "1; mode=block");
        assert_eq!(resp.headers().get("referrer-policy").unwrap(), "no-referrer");
        assert!(resp.headers().get("strict-transport-security").is_some());
        assert!(resp.headers().get("content-security-policy").is_some());
        assert_eq!(resp.headers().get("server").unwrap(), "Genesis");
    }

    #[tokio::test]
    async fn test_emergency_lockdown_blocks_writes() {
        use crate::shield::EmergencyControls;

        let world = test_world();
        let controls = Arc::new(Mutex::new(EmergencyControls {
            mode: crate::shield::GatewayMode::Lockdown,
            intake_disabled: false,
            treasury_frozen: false,
        }));
        let app = build_router_with_controls(world, controls);

        // POST should be blocked
        let body = serde_json::json!({
            "external_id": "lockdown-test",
            "public_key": "pk_lockdown"
        });
        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_emergency_lockdown_allows_reads() {
        use crate::shield::EmergencyControls;

        let world = test_world();
        let controls = Arc::new(Mutex::new(EmergencyControls {
            mode: crate::shield::GatewayMode::Lockdown,
            intake_disabled: false,
            treasury_frozen: false,
        }));
        let app = build_router_with_controls(world, controls);

        // GET should still work
        let req = Request::builder()
            .uri("/status")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_emergency_shutdown_blocks_all() {
        use crate::shield::EmergencyControls;

        let world = test_world();
        let controls = Arc::new(Mutex::new(EmergencyControls {
            mode: crate::shield::GatewayMode::Shutdown,
            intake_disabled: false,
            treasury_frozen: false,
        }));
        let app = build_router_with_controls(world, controls);

        let req = Request::builder()
            .uri("/status")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_intake_disabled_blocks_register() {
        use crate::shield::EmergencyControls;

        let world = test_world();
        let controls = Arc::new(Mutex::new(EmergencyControls {
            mode: crate::shield::GatewayMode::Normal,
            intake_disabled: true,
            treasury_frozen: false,
        }));
        let app = build_router_with_controls(world, controls);

        let body = serde_json::json!({
            "external_id": "intake-test",
            "public_key": "pk_intake"
        });
        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_agent_id_rejects_non_hex() {
        let world = test_world();
        let app = build_router(world);

        // "zzzz" is not valid hex
        let req = Request::builder()
            .uri("/agent/zzzz_not_hex")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_agent_id_rejects_oversized() {
        let world = test_world();
        let app = build_router(world);

        let long_hex = "a".repeat(128);
        let req = Request::builder()
            .uri(format!("/agent/{}", long_hex))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_register_rejects_injection() {
        let world = test_world();
        let app = build_router(world);

        let body = serde_json::json!({
            "external_id": "<script>alert('xss')</script>",
            "public_key": "pk_normal"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_register_rejects_oversized_field() {
        let world = test_world();
        let app = build_router(world);

        let body = serde_json::json!({
            "external_id": "x".repeat(500),
            "public_key": "pk_normal"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_agent_error_does_not_leak_id() {
        let world = test_world();
        let app = build_router(world);

        let req = Request::builder()
            .uri("/agent/deadbeef")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        // The response must NOT echo back the requested ID
        assert!(!text.contains("deadbeef"));
    }

    // ===== SSE + Observatory tests =====

    #[tokio::test]
    async fn test_observatory_returns_html() {
        let world = test_world();
        let app = build_router(world);

        let req = Request::builder()
            .uri("/observatory")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("Genesis Protocol"));
        assert!(text.contains("three.js"));
    }

    #[tokio::test]
    async fn test_sse_stream_returns_200() {
        let world = test_world();
        let app = build_router(world);

        let req = Request::builder()
            .uri("/stream")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        // SSE content type
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("text/event-stream"), "expected SSE content type, got: {}", ct);
    }

    #[test]
    fn test_snapshot_sse_frame() {
        let world = test_world();
        let frame = snapshot_sse_frame(&world);
        assert_eq!(frame.population, 20); // primordial agents
        assert!(frame.total_atp > 0.0);
        assert!(!frame.agents.is_empty());
        assert_eq!(frame.agents.len(), 20);
        // Every agent should have an id and role
        for agent in &frame.agents {
            assert!(!agent.id.is_empty());
            assert!(!agent.role.is_empty());
        }
        // Should serialize cleanly
        let json = serde_json::to_string(&frame).unwrap();
        assert!(json.contains("epoch"));
        assert!(json.contains("treasury_reserve"));
    }
}
