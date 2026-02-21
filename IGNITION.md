# GENESIS VPS IGNITION — FULL AUTOMATION FLOW

**Target:** Hetzner CX21 · Ubuntu 22.04 LTS · Native binary + systemd · Caddy TLS

**Prerequisites:**
- Hetzner VPS provisioned
- Domain A record pointing to VPS IP
- Repo pushed with `scripts/deploy.sh`, `scripts/genesis.service`, `scripts/Caddyfile`, `scripts/firewall.sh`, `scripts/validate.sh`

---

## Phase 0 — Local Pre-Flight (Before SSH)

From your local machine:

```bash
nslookup YOUR_DOMAIN
```

Must resolve to Hetzner IP.

```bash
git status
git push
```

Ensure all deployment scripts are committed and pushed.

---

## Phase 1 — VPS Provisioning

```bash
ssh root@YOUR_IP
```

```bash
apt update && apt upgrade -y
adduser genesis
usermod -aG sudo genesis
su - genesis
```

---

## Phase 2 — Clone + Deploy

```bash
git clone YOUR_REPO_URL ~/genesis-protocol
cd ~/genesis-protocol
bash scripts/deploy.sh YOUR_DOMAIN
```

`deploy.sh` handles: Rust install, release build, Caddy install + config, systemd service, firewall, endpoint verification.

No manual steps.

---

## Phase 3 — Service Verification

```bash
journalctl -u genesis -f
```

Expected output:

```
Genesis Gateway listening on 0.0.0.0:3000
Epoch tick epoch=...
```

Verify endpoints:

```bash
curl http://localhost:3000/status
curl https://YOUR_DOMAIN/status
```

Both must return JSON with no null fields.

---

## Phase 4 — Hour 0 Validation

```bash
GENESIS_DOMAIN=YOUR_DOMAIN GENESIS_PROTO=https bash scripts/validate.sh hour0
```

Confirm:
- TLS valid
- Shield active
- No null telemetry
- Rate limiter responding
- No memory anomalies

---

## Phase 5 — Enable Adapter (Hour 1)

```bash
nano .env
```

Set:

```
MOLTBOOK_ENDPOINT=https://REAL_ENDPOINT
MOLTBOOK_API_KEY=YOUR_KEY
MOLTBOT_HEARTBEAT_INTERVAL=300
```

```bash
sudo systemctl restart genesis
journalctl -u genesis -f
```

Expected:

```
Heartbeat posted total_sent=1
Moltbot adapter alive snapshots=60 heartbeats=1 milestones=0
```

No retry storms. No 401s. No timeouts.

---

## Phase 6 — 5-Hour Burn Timeline

### Hour 0–1
- Service stable
- TLS stable
- Firewall intact

### Hour 1–2
- First heartbeat confirmed
- Adapter alive logs visible
- Run: `bash scripts/validate.sh hour1`

### Hour 2–3
- Memory steady, no CPU spikes

```bash
ps -o rss,vsz,pid -p $(pidof genesis-protocol)
sudo lsof -p $(pidof genesis-protocol) | wc -l
```

- Run: `bash scripts/validate.sh hour2`

### Hour 4
```bash
GENESIS_DOMAIN=YOUR_DOMAIN GENESIS_PROTO=https bash scripts/validate.sh stress
```

### Hour 5
```bash
GENESIS_DOMAIN=YOUR_DOMAIN GENESIS_PROTO=https bash scripts/validate.sh hour5
```

If clean → public announcement allowed.

---

## Post-Burn Hardening

```bash
sudo apt install fail2ban -y
sudo systemctl enable fail2ban
```

Optional:
- Add Cloudflare proxy
- Lock SSH to key-only auth (`PasswordAuthentication no` in `/etc/ssh/sshd_config`)
- Reduce heartbeat interval from 300s to 60s

---

## Emergency Commands

```bash
# Stop the organism
sudo systemctl stop genesis

# Restart
sudo systemctl restart genesis

# Check status
sudo systemctl status genesis

# Last 100 log lines
journalctl -u genesis -n 100 --no-pager

# Memory/CPU snapshot
ps -o rss,vsz,%cpu,%mem,pid -p $(pidof genesis-protocol)

# Open file descriptors
sudo lsof -p $(pidof genesis-protocol) | wc -l
```

---

## Silent Failure Watch Pattern

If adapter alive logs continue but `heartbeats_sent` stops incrementing:
Moltbook is returning non-success responses. Check:

```bash
journalctl -u genesis --since "5 minutes ago" | grep -i "heartbeat\|error\|retry"
```

---

No improvisation. No feature creep. No mid-flight architecture changes. Just ignition.
