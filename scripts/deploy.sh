#!/usr/bin/env bash
# Genesis Protocol — VPS Deployment Script
#
# Run this on a fresh Ubuntu 22.04 Hetzner VPS as the 'genesis' user.
#
# Prerequisites:
#   - You are logged in as 'genesis' (non-root, sudo access)
#   - Your domain DNS A record points to this server's IP
#   - The genesis-protocol repo is cloned to ~/genesis-protocol
#
# Usage:
#   bash scripts/deploy.sh YOUR_DOMAIN
#
# Example:
#   bash scripts/deploy.sh genesis.example.com

set -euo pipefail

DOMAIN="${1:-}"

if [ -z "$DOMAIN" ]; then
    echo "Usage: bash scripts/deploy.sh YOUR_DOMAIN"
    echo "Example: bash scripts/deploy.sh genesis.example.com"
    exit 1
fi

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

step() { echo -e "\n${GREEN}[$(date +%H:%M:%S)]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; exit 1; }

# ─── Phase 1: System Packages ───────────────────────────
step "Updating system packages..."
sudo apt update -qq && sudo apt upgrade -y -qq

step "Installing build dependencies..."
sudo apt install -y -qq build-essential pkg-config libssl-dev curl git

# ─── Phase 2: Rust ──────────────────────────────────────
if command -v rustc &>/dev/null; then
    step "Rust already installed: $(rustc --version)"
else
    step "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    step "Rust installed: $(rustc --version)"
fi

# ─── Phase 3: Build Release Binary ──────────────────────
step "Building release binary..."
cd ~/genesis-protocol
cargo build --release 2>&1

if [ ! -f target/release/genesis-protocol ]; then
    fail "Release binary not found after build"
fi
step "Binary built: $(ls -lh target/release/genesis-protocol | awk '{print $5}')"

# ─── Phase 4: Environment ───────────────────────────────
if [ ! -f .env ]; then
    step "Creating .env from template (adapter disabled)..."
    cp .env.example .env
else
    step ".env already exists — preserving"
fi

# ─── Phase 5: Caddy ─────────────────────────────────────
if command -v caddy &>/dev/null; then
    step "Caddy already installed: $(caddy version)"
else
    step "Installing Caddy..."
    sudo apt install -y -qq debian-keyring debian-archive-keyring apt-transport-https
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg 2>/dev/null
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list >/dev/null
    sudo apt update -qq
    sudo apt install caddy -y -qq
    step "Caddy installed: $(caddy version)"
fi

step "Configuring Caddy for ${DOMAIN}..."
sudo mkdir -p /var/log/caddy
sed "s/YOUR_DOMAIN/${DOMAIN}/g" scripts/Caddyfile | sudo tee /etc/caddy/Caddyfile >/dev/null
sudo systemctl reload caddy || sudo systemctl restart caddy

# ─── Phase 6: Firewall ──────────────────────────────────
step "Configuring firewall..."
sudo bash scripts/firewall.sh

# ─── Phase 7: Systemd Service ───────────────────────────
step "Installing systemd service..."
sudo cp scripts/genesis.service /etc/systemd/system/genesis.service
sudo systemctl daemon-reload
sudo systemctl enable genesis
sudo systemctl start genesis

# ─── Phase 8: Verify ────────────────────────────────────
step "Waiting for Genesis to start..."
sleep 5

if systemctl is-active --quiet genesis; then
    step "Genesis service is RUNNING"
else
    fail "Genesis service failed to start. Check: journalctl -u genesis -n 50"
fi

# Check local endpoint
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 http://localhost:3000/status 2>/dev/null || echo "000")
if [ "$HTTP_STATUS" = "200" ]; then
    step "Local /status endpoint: 200 OK"
else
    fail "Local /status returned ${HTTP_STATUS}"
fi

# Check public endpoint (may take a moment for TLS)
sleep 3
HTTPS_STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 15 "https://${DOMAIN}/status" 2>/dev/null || echo "000")
if [ "$HTTPS_STATUS" = "200" ]; then
    step "Public https://${DOMAIN}/status: 200 OK — TLS active"
else
    echo "  Note: Public HTTPS returned ${HTTPS_STATUS} — TLS may still be provisioning."
    echo "  Wait 60 seconds and try: curl https://${DOMAIN}/status"
fi

# ─── Done ────────────────────────────────────────────────
echo ""
echo "══════════════════════════════════════════════════════════"
echo "  GENESIS PROTOCOL — DEPLOYED"
echo "══════════════════════════════════════════════════════════"
echo ""
echo "  Domain:    https://${DOMAIN}"
echo "  Service:   sudo systemctl status genesis"
echo "  Logs:      journalctl -u genesis -f"
echo "  Validate:  GENESIS_DOMAIN=${DOMAIN} GENESIS_PROTO=https bash scripts/validate.sh hour0"
echo ""
echo "  Adapter is DISABLED. To enable (Hour 1):"
echo "    1. Edit .env — set MOLTBOOK_ENDPOINT and MOLTBOOK_API_KEY"
echo "    2. sudo systemctl restart genesis"
echo "    3. journalctl -u genesis -f   (watch for heartbeat logs)"
echo ""
