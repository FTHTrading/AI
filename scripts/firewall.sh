#!/usr/bin/env bash
# Genesis Protocol — Firewall Setup (UFW)
#
# Usage: sudo bash scripts/firewall.sh
#
# Configures UFW with minimal attack surface:
#   - SSH (22)
#   - HTTP (80)  — Caddy redirect to HTTPS
#   - HTTPS (443) — Caddy reverse proxy
#   - All other inbound blocked
#   - Port 3000 NOT exposed (internal only, Caddy proxies to it)

set -euo pipefail

echo "=== Genesis Protocol — Firewall Setup ==="

# Reset to clean state
ufw --force reset

# Default deny inbound, allow outbound
ufw default deny incoming
ufw default allow outgoing

# SSH — required
ufw allow 22/tcp comment 'SSH'

# HTTP — Caddy needs this for ACME challenges + redirect
ufw allow 80/tcp comment 'HTTP (Caddy)'

# HTTPS — public traffic
ufw allow 443/tcp comment 'HTTPS (Caddy)'

# Rate limit SSH to slow brute force
ufw limit 22/tcp comment 'SSH rate limit'

# Enable
ufw --force enable

echo ""
echo "=== Firewall Active ==="
ufw status verbose
echo ""
echo "Port 3000 is NOT exposed externally."
echo "All public traffic goes through Caddy (443) → localhost:3000."
