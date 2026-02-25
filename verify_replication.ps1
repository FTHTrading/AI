#!/usr/bin/env pwsh
# verify_replication.ps1 — Genesis Protocol Replication Verifier
#
# Compares locally-generated experiment results against the canonical
# hash registry in replication_status.json.
#
# Usage:
#   ./verify_replication.ps1                        # Verify all experiments
#   ./verify_replication.ps1 -Experiment "s4_full_attack"  # Verify one
#   ./verify_replication.ps1 -Submit -Username "your_name" # Generate submission
#
# Requirements: Experiments must already be run (cargo test or cargo run).

param(
    [string]$Experiment = "",
    [string]$ExperimentsDir = "",
    [switch]$Submit,
    [string]$Username = "",
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

# ── Locate project root ─────────────────────────────────────────────────────
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = if (Test-Path "$scriptDir\..\Cargo.toml") {
    Resolve-Path "$scriptDir\.."
} elseif (Test-Path "$scriptDir\Cargo.toml") {
    $scriptDir
} else {
    Get-Location
}

$registryPath = Join-Path $projectRoot "replication_status.json"
if (-not (Test-Path $registryPath)) {
    Write-Error "Cannot find replication_status.json at: $registryPath"
    exit 1
}

$experimentsPath = if ($ExperimentsDir) { $ExperimentsDir } else { Join-Path $projectRoot "experiments" }
if (-not (Test-Path $experimentsPath)) {
    Write-Error "Cannot find experiments directory at: $experimentsPath"
    exit 1
}

# ── Load canonical registry ──────────────────────────────────────────────────
$registry = Get-Content $registryPath -Raw | ConvertFrom-Json
$canonical = @{}
foreach ($entry in $registry.canonical_experiments.hash_registry) {
    $canonical[$entry.experiment] = $entry
}

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════╗"
Write-Host "║        GENESIS PROTOCOL REPLICATION VERIFIER            ║"
Write-Host "║                                                          ║"
Write-Host "║  Canonical: $($canonical.Count) experiments, $($registry.canonical_experiments.total_worlds) worlds  ║"
Write-Host "╚══════════════════════════════════════════════════════════╝"
Write-Host ""

# ── Gather local manifests ───────────────────────────────────────────────────
$localManifests = @{}
$manifestFiles = Get-ChildItem $experimentsPath -Filter "*manifest*" -Recurse
foreach ($mf in $manifestFiles) {
    $dir = Split-Path $mf.FullName -Parent | Split-Path -Leaf
    $data = Get-Content $mf.FullName -Raw | ConvertFrom-Json
    $localManifests[$dir] = $data
}

# ── Filter if specific experiment requested ──────────────────────────────────
$toVerify = if ($Experiment) {
    @($Experiment)
} else {
    $canonical.Keys | Sort-Object
}

# ── Verify ───────────────────────────────────────────────────────────────────
$matches = 0
$mismatches = 0
$missing = 0
$results = @()

foreach ($expName in $toVerify) {
    $canonEntry = $canonical[$expName]
    if (-not $canonEntry) {
        Write-Host "  [?] $expName — not in canonical registry" -ForegroundColor Yellow
        continue
    }

    if (-not $localManifests.ContainsKey($expName)) {
        Write-Host "  [MISSING] $expName — no local manifest found" -ForegroundColor DarkGray
        $missing++
        $results += [ordered]@{
            experiment = $expName
            status = "missing"
            canonical_hash = $canonEntry.result_hash
            local_hash = $null
            match = $null
        }
        continue
    }

    $local = $localManifests[$expName]
    $localHash = $local.result_hash
    $canonHash = $canonEntry.result_hash

    if ($localHash -eq $canonHash) {
        Write-Host "  [MATCH]   $expName" -ForegroundColor Green
        if ($Verbose) {
            Write-Host "            hash: $($canonHash.Substring(0, 32))..." -ForegroundColor DarkGreen
            Write-Host "            worlds: $($local.total_worlds) | seed: $($local.config.base_seed)" -ForegroundColor DarkGreen
        }
        $matches++
        $results += [ordered]@{
            experiment = $expName
            status = "match"
            canonical_hash = $canonHash
            local_hash = $localHash
            match = $true
            worlds = $local.total_worlds
            seed = $local.config.base_seed
            duration_ms = $local.duration_ms
        }
    } else {
        Write-Host "  [MISMATCH] $expName" -ForegroundColor Red
        Write-Host "    canonical: $canonHash" -ForegroundColor DarkRed
        Write-Host "    local:     $localHash" -ForegroundColor DarkRed
        $mismatches++
        $results += [ordered]@{
            experiment = $expName
            status = "mismatch"
            canonical_hash = $canonHash
            local_hash = $localHash
            match = $false
            worlds = $local.total_worlds
            seed = $local.config.base_seed
        }
    }
}

# ── Summary ──────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════"
Write-Host "  VERIFICATION SUMMARY"
Write-Host "  Matches:    $matches" -ForegroundColor Green
Write-Host "  Mismatches: $mismatches" -ForegroundColor $(if ($mismatches -gt 0) { "Red" } else { "Green" })
Write-Host "  Missing:    $missing" -ForegroundColor $(if ($missing -gt 0) { "Yellow" } else { "Green" })
Write-Host "  Total:      $($matches + $mismatches + $missing) / $($canonical.Count)"
Write-Host "═══════════════════════════════════════════════════════════"

# ── Collect system info ──────────────────────────────────────────────────────
$rustVersion = try { (rustc --version 2>&1) -replace '\s+', ' ' } catch { "unknown" }
$osInfo = "$([System.Runtime.InteropServices.RuntimeInformation]::OSDescription)"
$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture

# ── Generate submission if requested ─────────────────────────────────────────
if ($Submit) {
    if (-not $Username) {
        $Username = Read-Host "Enter your username/handle"
    }

    $submission = [ordered]@{
        schema = "genesis-replication-submission-v1"
        username = $Username
        timestamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        system = [ordered]@{
            os = $osInfo
            architecture = "$arch"
            rust_version = "$rustVersion"
        }
        summary = [ordered]@{
            experiments_verified = $matches + $mismatches
            hash_matches = $matches
            hash_mismatches = $mismatches
            missing = $missing
        }
        results = $results
    }

    $outPath = Join-Path $projectRoot "replication_submission_$($Username)_$(Get-Date -Format 'yyyyMMdd').json"
    $submission | ConvertTo-Json -Depth 10 | Out-File $outPath -Encoding utf8
    Write-Host ""
    Write-Host "  Submission saved: $outPath" -ForegroundColor Cyan
    Write-Host "  Post this file as a GitHub Issue or Moltbook comment." -ForegroundColor Cyan
}

Write-Host ""
Write-Host "  System: $osInfo ($arch)"
Write-Host "  Rust:   $rustVersion"
Write-Host ""
