#!/usr/bin/env bash
# Genesis Protocol — VPS Bootstrap (run as root)
#
# This script provisions a fresh Ubuntu 22.04 VPS from zero.
# It creates the genesis user, unpacks the source, and hands off to deploy.sh.
#
# Prerequisites:
#   - /root/genesis-deploy.tar exists (created by git archive on local machine)
#   - Domain A record pointing to this server's IP
#
# Usage:
#   bash scripts/bootstrap.sh YOUR_DOMAIN
#
# If running manually after SCP:
#   scp genesis-deploy.tar root@YOUR_IP:/root/
#   ssh root@YOUR_IP
#   tar -xf /root/genesis-deploy.tar -C /tmp/genesis-src
#   bash /tmp/genesis-src/scripts/bootstrap.sh YOUR_DOMAIN

set -euo pipefail

DOMAIN="${1:-}"

if [ -z "$DOMAIN" ]; then
    echo "Usage: bash scripts/bootstrap.sh YOUR_DOMAIN"
    exit 1
fi

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'
step() { echo -e "\n${GREEN}[$(date +%H:%M:%S)]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; exit 1; }

# ── Verify root ──────────────────────────────────────────
if [ "$(id -u)" -ne 0 ]; then
    fail "Must run as root"
fi

# ── System update ────────────────────────────────────────
step "Updating system packages..."
apt update -qq && apt upgrade -y -qq
apt install -y -qq build-essential pkg-config libssl-dev curl git

# ── Create genesis user ─────────────────────────────────
if id "genesis" &>/dev/null; then
    step "User 'genesis' already exists"
else
    step "Creating user 'genesis'..."
    adduser --disabled-password --gecos "Genesis Protocol" genesis
    usermod -aG sudo genesis
    echo "genesis ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/genesis
    chmod 440 /etc/sudoers.d/genesis
fi

# ── Unpack source ────────────────────────────────────────
step "Unpacking source to /home/genesis/genesis-protocol..."

# Resolve source: tarball, current dir, or script's parent dir
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT_PARENT="$(dirname "$SCRIPT_DIR")"

if [ -f /root/genesis-deploy.tar ]; then
    mkdir -p /home/genesis/genesis-protocol
    tar -xf /root/genesis-deploy.tar -C /home/genesis/genesis-protocol
elif [ -f "${SCRIPT_PARENT}/Cargo.toml" ]; then
    step "Running from unpacked tree — copying to /home/genesis/genesis-protocol..."
    mkdir -p /home/genesis/genesis-protocol
    cp -r "${SCRIPT_PARENT}/." /home/genesis/genesis-protocol/
elif [ -f ./Cargo.toml ]; then
    step "Running from source tree — copying to /home/genesis/genesis-protocol..."
    mkdir -p /home/genesis/genesis-protocol
    cp -r . /home/genesis/genesis-protocol/
else
    fail "No source found. Place genesis-deploy.tar in /root/ or run from the source directory."
fi

chown -R genesis:genesis /home/genesis/genesis-protocol

# ── Run deploy.sh as genesis user ────────────────────────
step "Handing off to deploy.sh as genesis user..."
su - genesis -c "cd ~/genesis-protocol && bash scripts/deploy.sh ${DOMAIN}"

# ── Post-deploy verification ────────────────────────────
step "── Post-Deploy Verification ──"

sleep 3

# Service
if systemctl is-active --quiet genesis; then
    step "Genesis service: RUNNING"
else
    fail "Genesis service not running. Check: journalctl -u genesis -n 50"
fi

# Firewall
UFW_STATUS=$(ufw status | head -1)
step "Firewall: $UFW_STATUS"

# Caddy
if systemctl is-active --quiet caddy; then
    step "Caddy: RUNNING"
else
    echo "  Warning: Caddy not active yet"
fi

# Local endpoint
LOCAL_STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 http://localhost:3000/status 2>/dev/null || echo "000")
step "Local /status: HTTP $LOCAL_STATUS"

# Fetch full status payload
if [ "$LOCAL_STATUS" = "200" ]; then
    step "Status payload:"
    curl -s http://localhost:3000/status 2>/dev/null | python3 -m json.tool 2>/dev/null || curl -s http://localhost:3000/status
fi

# Public endpoint
sleep 5
PUBLIC_STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 15 "https://${DOMAIN}/status" 2>/dev/null || echo "000")
if [ "$PUBLIC_STATUS" = "200" ]; then
    step "Public https://${DOMAIN}/status: 200 OK — TLS ACTIVE"
else
    step "Public HTTPS: $PUBLIC_STATUS — TLS may still be provisioning"
    echo "  Try again in 60 seconds: curl https://${DOMAIN}/status"
fi

# ── Summary ──────────────────────────────────────────────
echo ""
echo "══════════════════════════════════════════════════════════"
echo "  GENESIS PROTOCOL — IGNITION COMPLETE"
echo "══════════════════════════════════════════════════════════"
echo ""
echo "  Domain:     https://${DOMAIN}"
echo "  Service:    systemctl status genesis"
echo "  Logs:       journalctl -u genesis -f"
echo ""
echo "  ── Validation Commands ──"
echo "  Hour 0:  GENESIS_DOMAIN=${DOMAIN} GENESIS_PROTO=https bash scripts/validate.sh hour0"
echo "  Hour 1:  Edit .env (set MOLTBOOK_API_KEY + MOLTBOOK_SUBMOLT), then: sudo systemctl restart genesis"
echo "  Hour 2:  GENESIS_DOMAIN=${DOMAIN} GENESIS_PROTO=https bash scripts/validate.sh hour2"
echo "  Stress:  GENESIS_DOMAIN=${DOMAIN} GENESIS_PROTO=https bash scripts/validate.sh stress"
echo "  Hour 5:  GENESIS_DOMAIN=${DOMAIN} GENESIS_PROTO=https bash scripts/validate.sh hour5"
echo ""
echo "  ── Emergency ──"
echo "  Stop:     sudo systemctl stop genesis"
echo "  Restart:  sudo systemctl restart genesis"
echo "  Logs:     journalctl -u genesis -n 100 --no-pager"
echo ""
