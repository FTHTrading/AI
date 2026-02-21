# merkle.ps1 - Genesis Protocol Provenance Generator
# Builds per-crate Merkle trees, manifest, and edition root.
# No external dependencies. PowerShell 5.1+ compatible.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts/merkle.ps1
#
# Output:
#   dist/merkle.json       - Merkle trees + edition root
#   dist/manifest.json     - Per-file SHA-256 manifest
#   dist/provenance.json   - Full provenance record

param(
    [string]$Root = (Split-Path -Parent $PSScriptRoot)
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# --- Helpers -----------------------------------------------------------------

function Get-SHA256 {
    param([string]$FilePath)
    $hash = Get-FileHash -Path $FilePath -Algorithm SHA256
    return $hash.Hash.ToLower()
}

function Get-SHA256String {
    param([string]$Text)
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $hash = $sha.ComputeHash($bytes)
    return ($hash | ForEach-Object { $_.ToString("x2") }) -join ""
}

function Build-MerkleTree {
    param([string[]]$Leaves)

    if ($Leaves.Count -eq 0) { return @{ root = ""; leaves = @(); levels = @() } }
    if ($Leaves.Count -eq 1) { return @{ root = $Leaves[0]; leaves = $Leaves; levels = @(,$Leaves) } }

    $levels = @()
    $current = $Leaves
    $levels += ,($current.Clone())

    while ($current.Count -gt 1) {
        $next = @()
        for ($i = 0; $i -lt $current.Count; $i += 2) {
            if ($i + 1 -lt $current.Count) {
                $combined = $current[$i] + $current[$i + 1]
            } else {
                $combined = $current[$i] + $current[$i]
            }
            $next += Get-SHA256String -Text $combined
        }
        $current = $next
        $levels += ,($current.Clone())
    }

    return @{
        root   = $current[0]
        leaves = $Leaves
        levels = $levels
    }
}

# --- Safe git wrappers (PS 5.1 compatible) -----------------------------------

function Get-GitValue {
    param([string]$GitArgs, [string]$Default = "unknown")
    try {
        $argArray = $GitArgs.Split(" ")
        $result = & git -C $Root @argArray 2>$null
        if ($result) { return $result.Trim() } else { return $Default }
    } catch {
        return $Default
    }
}

# --- Crate Discovery ---------------------------------------------------------

$distDir = Join-Path $Root "dist"

if (-not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir -Force | Out-Null
}

$crates = @(
    @{ name = "genesis-dna";  path = "crates/genesis-dna/src";  role = "Genome and Identity" }
    @{ name = "metabolism";   path = "crates/metabolism/src";    role = "Energy and Economics" }
    @{ name = "ecosystem";    path = "crates/ecosystem/src";    role = "Communication and Markets" }
    @{ name = "evolution";    path = "crates/evolution/src";     role = "Mutation and Selection" }
    @{ name = "apostle";      path = "crates/apostle/src";      role = "Outbound Intelligence" }
    @{ name = "gateway";      path = "crates/gateway/src";      role = "Server and World Engine" }
)

Write-Host ""
Write-Host "  Genesis Protocol - Provenance Generator" -ForegroundColor Cyan
Write-Host "  ========================================" -ForegroundColor DarkCyan
Write-Host ""

# --- Per-Crate Merkle Trees --------------------------------------------------

$manifest = @{}
$crateTrees = @{}
$crateRoots = @()
$totalFiles = 0
$totalLines = 0

foreach ($crate in $crates) {
    $srcPath = Join-Path $Root $crate.path
    $files = Get-ChildItem -Path $srcPath -Filter "*.rs" | Sort-Object Name

    $leaves = @()
    $fileEntries = @()

    foreach ($file in $files) {
        $hash = Get-SHA256 -FilePath $file.FullName
        $relPath = $file.FullName.Replace("$Root\", "").Replace("\", "/")
        $lineCount = (Get-Content $file.FullName).Count

        $leaves += $hash
        $fileEntries += @{
            file  = $relPath
            hash  = $hash
            lines = $lineCount
        }

        $manifest[$relPath] = @{
            sha256 = $hash
            lines  = $lineCount
            crate  = $crate.name
        }

        $totalFiles++
        $totalLines += $lineCount
    }

    $tree = Build-MerkleTree -Leaves $leaves
    $crateRoots += $tree.root

    $crateTrees[$crate.name] = @{
        role       = $crate.role
        root       = $tree.root
        leaf_count = $leaves.Count
        files      = $fileEntries
    }

    Write-Host "  [$($crate.name)]" -ForegroundColor Green -NoNewline
    Write-Host " $($leaves.Count) files -> root: $($tree.root.Substring(0,16))..." -ForegroundColor DarkGray
}

# --- Test Files --------------------------------------------------------------

$testFiles = Get-ChildItem -Path (Join-Path $Root "crates") -Recurse -Filter "*.rs" |
    Where-Object { $_.FullName -match "tests" } | Sort-Object Name

$testLeaves = @()
$testEntries = @()
foreach ($file in $testFiles) {
    $hash = Get-SHA256 -FilePath $file.FullName
    $relPath = $file.FullName.Replace("$Root\", "").Replace("\", "/")
    $lineCount = (Get-Content $file.FullName).Count

    $testLeaves += $hash
    $testEntries += @{ file = $relPath; hash = $hash; lines = $lineCount }
    $manifest[$relPath] = @{ sha256 = $hash; lines = $lineCount; crate = "tests" }
    $totalFiles++
    $totalLines += $lineCount
}

$testTree = Build-MerkleTree -Leaves $testLeaves
$crateRoots += $testTree.root

Write-Host "  [tests]" -ForegroundColor Green -NoNewline
Write-Host " $($testLeaves.Count) files -> root: $($testTree.root.Substring(0,16))..." -ForegroundColor DarkGray

# --- Edition Root ------------------------------------------------------------

$editionTree = Build-MerkleTree -Leaves $crateRoots
$editionRoot = $editionTree.root

Write-Host ""
Write-Host "  Edition Root: " -ForegroundColor Yellow -NoNewline
Write-Host $editionRoot -ForegroundColor White
Write-Host "  Files: $totalFiles | Lines: $totalLines" -ForegroundColor DarkGray
Write-Host ""

# --- Git Metadata ------------------------------------------------------------

$gitCommit   = Get-GitValue -GitArgs "rev-parse HEAD"
$gitBranch   = Get-GitValue -GitArgs "rev-parse --abbrev-ref HEAD"
$gitRemote   = Get-GitValue -GitArgs "remote get-url origin"
$commitCount = Get-GitValue -GitArgs "rev-list --count HEAD" -Default "0"

# --- Write Merkle JSON -------------------------------------------------------

$crateRootsMap = @{}
foreach ($crate in $crates) {
    $crateRootsMap[$crate.name] = $crateTrees[$crate.name].root
}
$crateRootsMap["tests"] = $testTree.root

$merkleOutput = @{
    version      = "1.0"
    generated    = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    edition_root = $editionRoot
    crate_roots  = $crateRootsMap
    crates       = $crateTrees
    tests        = @{
        root       = $testTree.root
        leaf_count = $testLeaves.Count
        files      = $testEntries
    }
    git          = @{
        commit  = $gitCommit
        branch  = $gitBranch
        remote  = $gitRemote
        commits = [int]$commitCount
    }
}

$merkleJson = $merkleOutput | ConvertTo-Json -Depth 10
[System.IO.File]::WriteAllText((Join-Path $distDir "merkle.json"), $merkleJson)

# --- Write Manifest JSON -----------------------------------------------------

$manifestOutput = @{
    version     = "1.0"
    generated   = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    total_files = $totalFiles
    total_lines = $totalLines
    files       = $manifest
}

$manifestJson = $manifestOutput | ConvertTo-Json -Depth 10
[System.IO.File]::WriteAllText((Join-Path $distDir "manifest.json"), $manifestJson)

# --- Write Provenance JSON ---------------------------------------------------

$provenance = @{
    protocol     = "Genesis Protocol"
    standard     = "SOP-1 (Software Organism Protocol v1)"
    version      = "1.0"
    generated    = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    author       = @{
        name   = "Kevan Burns"
        alias  = "Kidd James"
        orcid  = "0009-0008-8425-939X"
    }
    repository   = @{
        url     = "https://github.com/FTHTrading/AI"
        branch  = $gitBranch
        commit  = $gitCommit
        commits = [int]$commitCount
    }
    integrity    = @{
        edition_root = $editionRoot
        crate_roots  = $crateRootsMap
        total_files  = $totalFiles
        total_lines  = $totalLines
    }
    evidence_chain = @(
        @{ layer = 1; evidence = "Local filesystem";     purpose = "First creation timestamps";      record = "OS metadata" }
        @{ layer = 2; evidence = "Git commit history";    purpose = "Continuous authorship timeline"; record = "GitHub ($commitCount commits)" }
        @{ layer = 3; evidence = "Merkle trees";          purpose = "Per-crate integrity proofs";     record = "dist/merkle.json" }
        @{ layer = 4; evidence = "SHA-256 edition root";  purpose = "Content integrity fingerprint";  record = $editionRoot }
        @{ layer = 5; evidence = "ORCID";                 purpose = "Author identity record";         record = "0009-0008-8425-939X" }
        @{ layer = 6; evidence = "DOI";                   purpose = "Permanent academic identifier";  record = "10.5281/zenodo.18646886" }
    )
    on_chain_anchoring = @{
        description = "Edition root can be anchored on Polygon via the same LiteraryAnchor contract used for 2500-donkeys"
        contract    = "0x97f456300817eaE3B40E235857b856dfFE8bba90"
        method      = "anchorEdition(editionRoot, ipfsCID)"
        network     = "Polygon Mainnet (Chain ID 137)"
        cost        = "Under 0.50 USD"
        status      = "Ready for anchoring"
    }
    related_works = @(
        @{
            title    = "The 2500 Donkeys: Deterministic Literary Publishing Protocol"
            repo     = "https://github.com/FTHTrading/2500-donkeys"
            doi      = "10.5281/zenodo.18646886"
            relation = "Shared author identity, provenance methodology, and on-chain infrastructure"
        }
    )
    system_invariants = @(
        @{ id = "E-1"; domain = "Ecology";    invariant = "Resource pools regenerate via logistic growth" }
        @{ id = "E-2"; domain = "Ecology";    invariant = "Seasonal modulation follows sinusoidal cycle with configurable amplitude" }
        @{ id = "E-3"; domain = "Ecology";    invariant = "Resource extraction is proportional to fitness, never winner-take-all" }
        @{ id = "E-4"; domain = "Ecology";    invariant = "Density-dependent foraging: extraction decreases with niche crowding" }
        @{ id = "M-1"; domain = "Metabolism"; invariant = "ATP balance cannot go negative: metabolic tick clamps at zero" }
        @{ id = "M-2"; domain = "Metabolism"; invariant = "Total ATP supply is sum of all agent balances (computed, not tracked)" }
        @{ id = "M-3"; domain = "Metabolism"; invariant = "Replication costs are deducted atomically from parent balance" }
        @{ id = "S-1"; domain = "Selection";  invariant = "Population cap is dynamic: total_capacity / 15, clamped [10, 500]" }
        @{ id = "S-2"; domain = "Selection";  invariant = "Selection pressure respects maturation period before culling" }
        @{ id = "S-3"; domain = "Selection";  invariant = "Stasis tolerance prevents premature extinction of viable agents" }
        @{ id = "G-1"; domain = "Genome";     invariant = "Primordial genomes are SHA-256 derived for genuine diversity" }
        @{ id = "G-2"; domain = "Genome";     invariant = "Mutation pressure is modulated by seasonal environmental stress" }
        @{ id = "P-1"; domain = "Provenance"; invariant = "Edition root = Merkle(crate_roots), deterministically recomputable" }
        @{ id = "P-2"; domain = "Provenance"; invariant = "All source files have SHA-256 entries in manifest.json" }
    )
}

$provenanceJson = $provenance | ConvertTo-Json -Depth 10
[System.IO.File]::WriteAllText((Join-Path $distDir "provenance.json"), $provenanceJson)

Write-Host "  Output:" -ForegroundColor Cyan
Write-Host "    dist/merkle.json     - Merkle trees + edition root" -ForegroundColor DarkGray
Write-Host "    dist/manifest.json   - Per-file SHA-256 manifest" -ForegroundColor DarkGray
Write-Host "    dist/provenance.json - Full provenance record" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  Provenance chain complete. Edition root ready for on-chain anchoring." -ForegroundColor Green
Write-Host ""
