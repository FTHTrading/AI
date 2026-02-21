# Genesis Protocol — One-Shot VPS Ignition (Windows -> Hetzner)
#
# Usage:
#   .\scripts\ignite.ps1 -IP 203.0.113.10 -Domain genesis.example.com
#
# This script:
#   1. Creates a tarball of the committed source (no target/, no .env)
#   2. SCPs it to the VPS
#   3. SSHes in, unpacks, runs bootstrap.sh (already in tarball)
#   4. Tails the service logs so you see first epoch ticks
#
# Prerequisites:
#   - Hetzner CX21 provisioned with Ubuntu 22.04
#   - Domain A record pointing to VPS IP
#   - OpenSSH client available (Windows 10+ built-in)
#   - You can SSH as root@IP (password or key)

param(
    [Parameter(Mandatory=$true)]
    [string]$IP,

    [Parameter(Mandatory=$true)]
    [string]$Domain
)

$ErrorActionPreference = "Stop"
$sshOpts = "-o", "StrictHostKeyChecking=accept-new"

function Write-Step($msg) {
    $ts = Get-Date -Format "HH:mm:ss"
    Write-Host "`n[$ts] $msg" -ForegroundColor Green
}

function Write-Fail($msg) {
    Write-Host "[FAIL] $msg" -ForegroundColor Red
    exit 1
}

function Invoke-SSH($cmd) {
    ssh @sshOpts "root@$IP" $cmd
    if ($LASTEXITCODE -ne 0) { Write-Fail "SSH command failed: $cmd" }
}

# ─── Pre-flight ──────────────────────────────────────────
Push-Location $PSScriptRoot\..

Write-Step "Pre-flight: verifying local repo..."

if (-not (Test-Path ".git")) {
    Write-Fail "Not in a git repository. Run from genesis-protocol root."
}

$status = git status --porcelain
if ($status) {
    Write-Host "Warning: uncommitted changes detected. Tarball uses last commit only." -ForegroundColor Yellow
}

# ─── Step 1: Create tarball ──────────────────────────────
Write-Step "Creating source tarball from HEAD..."
$tarball = "genesis-deploy.tar"
git archive --format=tar --output=$tarball HEAD
if (-not (Test-Path $tarball)) {
    Write-Fail "Failed to create tarball"
}
$size = [math]::Round((Get-Item $tarball).Length / 1KB, 1)
Write-Host "  Tarball created: ${size} KB"

# ─── Step 2: SCP tarball to VPS ──────────────────────────
Write-Step "Uploading tarball to root@${IP}..."
scp @sshOpts $tarball "root@${IP}:/root/genesis-deploy.tar"
if ($LASTEXITCODE -ne 0) { Write-Fail "SCP failed" }
Write-Host "  Upload complete."

# ─── Step 3: Unpack tarball on VPS ───────────────────────
# The tarball already contains scripts/bootstrap.sh — no need to embed bash in PowerShell.
Write-Step "Unpacking tarball on VPS..."
Invoke-SSH "mkdir -p /tmp/genesis-src; tar -xf /root/genesis-deploy.tar -C /tmp/genesis-src"

# ─── Step 4: Execute bootstrap on VPS ────────────────────
Write-Step "Executing bootstrap on VPS (this takes 5-10 minutes)..."
Write-Host "  Building Rust from source on first run — be patient." -ForegroundColor Yellow
Write-Host ""
Invoke-SSH "chmod +x /tmp/genesis-src/scripts/bootstrap.sh; bash /tmp/genesis-src/scripts/bootstrap.sh $Domain"

# ─── Step 5: Tail logs ──────────────────────────────────
Write-Step "Tailing Genesis logs (Ctrl+C to stop)..."
Write-Host ""
ssh @sshOpts "root@$IP" "journalctl -u genesis -f -n 30"

# ─── Cleanup ─────────────────────────────────────────────
Pop-Location
Remove-Item $tarball -ErrorAction SilentlyContinue
