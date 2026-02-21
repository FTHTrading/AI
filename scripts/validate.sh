#!/usr/bin/env bash
# Genesis Protocol — Deployment Validation Script
#
# Usage:
#   ./scripts/validate.sh [hour0|hour1|hour2|hour4|hour5|stress]
#
# Run at each phase of the 5-hour soft launch to verify operational integrity.

set -euo pipefail

DOMAIN="${GENESIS_DOMAIN:-localhost:3000}"
PROTO="${GENESIS_PROTO:-http}"
BASE="${PROTO}://${DOMAIN}"
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

pass() { echo -e "  ${GREEN}✓${NC} $1"; }
fail() { echo -e "  ${RED}✗${NC} $1"; FAILURES=$((FAILURES + 1)); }
warn() { echo -e "  ${YELLOW}⚠${NC} $1"; }
FAILURES=0

# ─── Helper ───────────────────────────────
check_endpoint() {
    local path=$1
    local expect_status=${2:-200}
    local status
    status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "${BASE}${path}" 2>/dev/null || echo "000")
    if [ "$status" = "$expect_status" ]; then
        pass "$path returned $status"
    else
        fail "$path returned $status (expected $expect_status)"
    fi
}

check_json() {
    local path=$1
    local field=$2
    local value
    value=$(curl -s --max-time 10 "${BASE}${path}" 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin).get('$field',''))" 2>/dev/null || echo "")
    if [ -n "$value" ]; then
        pass "$path.$field = $value"
    else
        fail "$path.$field is empty or endpoint unavailable"
    fi
}

# ─── Hour 0: Deploy (Adapter OFF) ────────
hour0() {
    echo ""
    echo "═══════════════════════════════════════════"
    echo " Hour 0 — Deploy Verification (Adapter OFF)"
    echo "═══════════════════════════════════════════"
    echo ""

    echo "Endpoints:"
    check_endpoint "/status"
    check_endpoint "/leaderboard"
    check_endpoint "/genesis"
    check_endpoint "/agent/000000" 404
    echo ""

    echo "Telemetry:"
    check_json "/status" "epoch"
    check_json "/status" "population"
    check_json "/status" "uptime_seconds"
    echo ""

    echo "Rate Limiter:"
    local rate_limited=false
    for i in $(seq 1 60); do
        status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 2 "${BASE}/status" 2>/dev/null || echo "000")
        if [ "$status" = "429" ]; then
            rate_limited=true
            pass "Rate limiter triggered after $i requests"
            break
        fi
    done
    if [ "$rate_limited" = false ]; then
        warn "Rate limiter did not trigger in 60 sequential requests (may need concurrent test)"
    fi
    echo ""

    echo "Container Health:"
    if command -v docker &>/dev/null; then
        local health
        health=$(docker inspect --format='{{.State.Health.Status}}' genesis-protocol 2>/dev/null || echo "unknown")
        if [ "$health" = "healthy" ]; then
            pass "Container health: $health"
        else
            warn "Container health: $health"
        fi

        local mem cpu
        read -r cpu mem <<< $(docker stats genesis-protocol --no-stream --format "{{.CPUPerc}} {{.MemUsage}}" 2>/dev/null || echo "? ?")
        pass "CPU: $cpu | Memory: $mem"
    else
        warn "Docker not available — skipping container checks"
    fi
    echo ""

    # Check no secret leaks in recent logs
    echo "Log Safety:"
    if command -v docker &>/dev/null; then
        local key_leak
        key_leak=$(docker logs genesis-protocol 2>&1 | grep -i "api_key\|bearer\|secret\|password" | head -1 || true)
        if [ -z "$key_leak" ]; then
            pass "No credential patterns in logs"
        else
            fail "Possible credential leak in logs: $key_leak"
        fi
    fi
    echo ""
}

# ─── Hour 1: Adapter Enabled ──────────────
hour1() {
    echo ""
    echo "═══════════════════════════════════════════"
    echo " Hour 1 — Adapter Verification"
    echo "═══════════════════════════════════════════"
    echo ""

    echo "Endpoints (still responding):"
    check_endpoint "/status"
    check_endpoint "/leaderboard"
    echo ""

    echo "Adapter Liveness (from logs):"
    if command -v docker &>/dev/null; then
        local adapter_started
        adapter_started=$(docker logs genesis-protocol 2>&1 | grep -c "Moltbot adapter started" || echo "0")
        if [ "$adapter_started" -gt 0 ]; then
            pass "Adapter started successfully"
        else
            fail "No adapter start message in logs"
        fi

        local heartbeat_count
        heartbeat_count=$(docker logs genesis-protocol 2>&1 | grep -c "Heartbeat posted" || echo "0")
        if [ "$heartbeat_count" -gt 0 ]; then
            pass "Heartbeats posted: $heartbeat_count"
        else
            fail "No heartbeats posted yet"
        fi

        local last_heartbeat
        last_heartbeat=$(docker logs genesis-protocol 2>&1 | grep "Heartbeat posted" | tail -1 || true)
        if [ -n "$last_heartbeat" ]; then
            pass "Last heartbeat: $last_heartbeat"
        fi

        local adapter_alive
        adapter_alive=$(docker logs genesis-protocol 2>&1 | grep -c "Moltbot adapter alive" || echo "0")
        if [ "$adapter_alive" -gt 0 ]; then
            pass "Adapter liveness pings: $adapter_alive"
        else
            warn "No liveness pings yet (emitted every ~60 epochs)"
        fi

        local errors
        errors=$(docker logs genesis-protocol 2>&1 | grep -c "Moltbot post failed\|Moltbot post rejected" || echo "0")
        if [ "$errors" = "0" ]; then
            pass "No post failures in logs"
        else
            warn "Post failures detected: $errors"
        fi

        local panics
        panics=$(docker logs genesis-protocol 2>&1 | grep -c "panic\|PANIC" || echo "0")
        if [ "$panics" = "0" ]; then
            pass "No panic traces in logs"
        else
            fail "Panic detected in logs!"
        fi
    else
        warn "Docker not available — check logs manually"
    fi
    echo ""
}

# ─── Hour 2-3: Stability ─────────────────
hour2() {
    echo ""
    echo "═══════════════════════════════════════════"
    echo " Hour 2-3 — Stability Check"
    echo "═══════════════════════════════════════════"
    echo ""

    echo "Endpoints:"
    check_endpoint "/status"
    check_json "/status" "epoch"
    check_json "/status" "uptime_seconds"
    echo ""

    echo "Resource Usage:"
    if command -v docker &>/dev/null; then
        local mem cpu
        read -r cpu mem <<< $(docker stats genesis-protocol --no-stream --format "{{.CPUPerc}} {{.MemUsage}}" 2>/dev/null || echo "? ?")
        pass "CPU: $cpu | Memory: $mem"

        # File descriptor count
        local pid
        pid=$(docker inspect --format='{{.State.Pid}}' genesis-protocol 2>/dev/null || echo "")
        if [ -n "$pid" ] && [ -d "/proc/$pid/fd" ]; then
            local fd_count
            fd_count=$(ls -1 /proc/$pid/fd 2>/dev/null | wc -l || echo "?")
            pass "Open file descriptors: $fd_count"
        else
            warn "Cannot read file descriptors (non-Linux host or pid unavailable)"
        fi
    fi
    echo ""

    echo "Adapter Continuity:"
    if command -v docker &>/dev/null; then
        local heartbeat_count
        heartbeat_count=$(docker logs genesis-protocol 2>&1 | grep -c "Heartbeat posted" || echo "0")
        pass "Total heartbeats posted: $heartbeat_count"

        local alive_count
        alive_count=$(docker logs genesis-protocol 2>&1 | grep -c "Moltbot adapter alive" || echo "0")
        pass "Liveness pings: $alive_count"

        local retry_storms
        retry_storms=$(docker logs genesis-protocol 2>&1 | grep "Moltbot post failed" | wc -l || echo "0")
        if [ "$retry_storms" -lt 5 ]; then
            pass "No retry storm ($retry_storms failures)"
        else
            warn "Elevated failures: $retry_storms"
        fi
    fi
    echo ""

    echo "Persistence:"
    if command -v docker &>/dev/null; then
        docker exec genesis-protocol ls -la /home/genesis/data/world_state.json 2>/dev/null && pass "world_state.json present" || warn "world_state.json not found"
    fi
    echo ""
}

# ─── Hour 4: Edge Stress ─────────────────
hour4() {
    echo ""
    echo "═══════════════════════════════════════════"
    echo " Hour 4 — Edge Reality Check"
    echo "═══════════════════════════════════════════"
    echo ""

    echo "Pre-stress baseline:"
    check_json "/status" "epoch"
    echo ""

    if command -v ab &>/dev/null; then
        echo "Stress: 200 concurrent /status..."
        ab -n 1000 -c 200 -q "${BASE}/status" 2>/dev/null | grep -E "Requests per second|Failed requests|Non-2xx" || warn "ab not available"
        echo ""

        echo "Stress: 50 concurrent /leaderboard..."
        ab -n 500 -c 50 -q "${BASE}/leaderboard" 2>/dev/null | grep -E "Requests per second|Failed requests|Non-2xx" || warn "ab not available"
        echo ""
    else
        warn "Apache Bench (ab) not installed — install with: apt-get install apache2-utils"
        echo "  Alternative: curl -s '${BASE}/status' for manual spot check"
    fi

    echo "Post-stress:"
    check_endpoint "/status"
    check_json "/status" "epoch"
    echo ""

    echo "Adapter survived stress?"
    if command -v docker &>/dev/null; then
        local heartbeat_count
        heartbeat_count=$(docker logs genesis-protocol 2>&1 | grep -c "Heartbeat posted" || echo "0")
        pass "Heartbeats still posting: $heartbeat_count"
    fi
    echo ""
}

# ─── Hour 5: Final Audit ─────────────────
hour5() {
    echo ""
    echo "═══════════════════════════════════════════"
    echo " Hour 5 — Final Audit"
    echo "═══════════════════════════════════════════"
    echo ""

    echo "Endpoints:"
    check_endpoint "/status"
    check_endpoint "/leaderboard"
    check_endpoint "/genesis"
    echo ""

    echo "Telemetry Snapshot:"
    check_json "/status" "epoch"
    check_json "/status" "population"
    check_json "/status" "uptime_seconds"
    check_json "/status" "total_births"
    check_json "/status" "total_deaths"
    echo ""

    echo "Resource Usage:"
    if command -v docker &>/dev/null; then
        local mem cpu
        read -r cpu mem <<< $(docker stats genesis-protocol --no-stream --format "{{.CPUPerc}} {{.MemUsage}}" 2>/dev/null || echo "? ?")
        pass "CPU: $cpu | Memory: $mem"
    fi
    echo ""

    echo "Adapter Summary:"
    if command -v docker &>/dev/null; then
        local heartbeat_count alive_count milestone_count failure_count
        heartbeat_count=$(docker logs genesis-protocol 2>&1 | grep -c "Heartbeat posted" || echo "0")
        alive_count=$(docker logs genesis-protocol 2>&1 | grep -c "Moltbot adapter alive" || echo "0")
        milestone_count=$(docker logs genesis-protocol 2>&1 | grep -c "Milestone posted" || echo "0")
        failure_count=$(docker logs genesis-protocol 2>&1 | grep -c "Moltbot post failed\|Moltbot post rejected" || echo "0")
        pass "Heartbeats: $heartbeat_count | Milestones: $milestone_count | Liveness: $alive_count | Failures: $failure_count"
    fi
    echo ""

    echo "Log Safety:"
    if command -v docker &>/dev/null; then
        local log_size
        log_size=$(docker logs genesis-protocol 2>&1 | wc -c || echo "?")
        pass "Total log size: $((log_size / 1024)) KB"

        local panics
        panics=$(docker logs genesis-protocol 2>&1 | grep -c "panic\|PANIC" || echo "0")
        if [ "$panics" = "0" ]; then
            pass "Zero panics"
        else
            fail "$panics panic traces found!"
        fi
    fi
    echo ""

    echo "Persistence:"
    if command -v docker &>/dev/null; then
        docker exec genesis-protocol ls -la /home/genesis/data/world_state.json 2>/dev/null && pass "world_state.json intact" || fail "world_state.json missing"
    fi
    echo ""

    echo "═══════════════════════════════════════════"
    if [ $FAILURES -eq 0 ]; then
        echo -e " ${GREEN}ALL CHECKS PASSED${NC} — organism is breathing clean"
    else
        echo -e " ${RED}$FAILURES CHECK(S) FAILED${NC} — review before announcing"
    fi
    echo "═══════════════════════════════════════════"
    echo ""
}

# ─── Quick stress (standalone) ────────────
stress() {
    echo ""
    echo "═══════════════════════════════════════════"
    echo " Quick Stress Test"
    echo "═══════════════════════════════════════════"
    echo ""

    if ! command -v ab &>/dev/null; then
        echo "Apache Bench required: apt-get install apache2-utils"
        exit 1
    fi

    echo "2000 requests, 100 concurrent to /status..."
    ab -n 2000 -c 100 "${BASE}/status" 2>/dev/null | grep -E "Requests per second|Failed requests|Non-2xx|Time taken"
    echo ""

    echo "Post-stress health:"
    check_endpoint "/status"
    echo ""
}

# ─── Main ─────────────────────────────────
case "${1:-}" in
    hour0) hour0 ;;
    hour1) hour1 ;;
    hour2) hour2 ;;
    hour4) hour4 ;;
    hour5) hour5 ;;
    stress) stress ;;
    *)
        echo "Genesis Protocol — Deployment Validator"
        echo ""
        echo "Usage: $0 <phase>"
        echo ""
        echo "  hour0   Deploy verification (adapter OFF)"
        echo "  hour1   Adapter first heartbeat check"
        echo "  hour2   Stability window (hours 2-3)"
        echo "  hour4   Edge reality check with stress"
        echo "  hour5   Final audit + pre-announcement baseline"
        echo "  stress  Quick standalone stress test"
        echo ""
        echo "Environment:"
        echo "  GENESIS_DOMAIN  Target domain (default: localhost:3000)"
        echo "  GENESIS_PROTO   Protocol (default: http)"
        echo ""
        echo "Example:"
        echo "  GENESIS_DOMAIN=genesis.yourdomain.com GENESIS_PROTO=https $0 hour0"
        ;;
esac

exit $FAILURES
