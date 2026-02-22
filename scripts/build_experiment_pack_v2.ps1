## Genesis Experiment Pack v2 — Build Script
## Deletes and rebuilds the deliverable bundle from verified repo artifacts.
## Fails if any input file is missing.

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not (Test-Path (Join-Path $root "Cargo.toml"))) {
    $root = (Get-Location).Path
}
$packDir = Join-Path $root "deliverables\genesis-experiment-pack-v2"

# --- Input manifest: every file that must exist ---
$inputs = @(
    "papers\sravan-executive-brief.md",
    "experiments\entropy_sweep\entropy_sweep_manifest.json",
    "experiments\entropy_sweep\entropy_sweep_data.csv",
    "experiments\entropy_sweep\entropy_sweep_report.txt",
    "experiments\catastrophe_resilience\catastrophe_resilience_manifest.json",
    "experiments\catastrophe_resilience\catastrophe_resilience_data.csv",
    "experiments\catastrophe_resilience\catastrophe_resilience_report.txt",
    "experiments\inequality_threshold\inequality_threshold_manifest.json",
    "experiments\inequality_threshold\inequality_threshold_data.csv",
    "experiments\inequality_threshold\inequality_threshold_report.txt",
    "experiments\treasury_stability\treasury_stability_manifest.json",
    "experiments\treasury_stability\treasury_stability_data.csv",
    "experiments\treasury_stability\treasury_stability_report.txt"
)

Write-Host "Verifying inputs..." -ForegroundColor Cyan
foreach ($f in $inputs) {
    $full = Join-Path $root $f
    if (-not (Test-Path $full)) {
        Write-Error "MISSING INPUT: $f"
        exit 1
    }
}
Write-Host "All $($inputs.Count) inputs verified." -ForegroundColor Green

# --- Clean and rebuild ---
if (Test-Path $packDir) {
    Remove-Item $packDir -Recurse -Force
    Write-Host "Cleaned previous pack." -ForegroundColor Yellow
}

# Create directory structure
$dirs = @(
    $packDir,
    (Join-Path $packDir "02_EXPERIMENTS\entropy_sweep"),
    (Join-Path $packDir "02_EXPERIMENTS\catastrophe_resilience"),
    (Join-Path $packDir "02_EXPERIMENTS\inequality_threshold"),
    (Join-Path $packDir "02_EXPERIMENTS\treasury_stability"),
    (Join-Path $packDir "03_INTEGRITY")
)
foreach ($d in $dirs) {
    New-Item -ItemType Directory -Path $d -Force | Out-Null
}

# --- Copy files ---
Write-Host "Assembling pack..." -ForegroundColor Cyan

# Executive brief
Copy-Item (Join-Path $root "papers\sravan-executive-brief.md") `
          (Join-Path $packDir "01_SRAVAN_EXECUTIVE_BRIEF.md")

# Experiment outputs
$experiments = @("entropy_sweep", "catastrophe_resilience", "inequality_threshold", "treasury_stability")
foreach ($exp in $experiments) {
    $srcDir = Join-Path $root "experiments\$exp"
    $dstDir = Join-Path $packDir "02_EXPERIMENTS\$exp"
    Copy-Item (Join-Path $srcDir "${exp}_manifest.json") $dstDir
    Copy-Item (Join-Path $srcDir "${exp}_data.csv")      $dstDir
    Copy-Item (Join-Path $srcDir "${exp}_report.txt")     $dstDir
}

# Reproduction guide and license notes
$guideSource = Join-Path $root "deliverables\00_README_REPRODUCE_v2.md"
$licenseSource = Join-Path $root "deliverables\04_LICENSE_NOTES.md"

if (Test-Path $guideSource) {
    Copy-Item $guideSource (Join-Path $packDir "00_README_REPRODUCE.md")
} else {
    Write-Error "MISSING: deliverables\00_README_REPRODUCE_v2.md — create it first."
    exit 1
}

if (Test-Path $licenseSource) {
    Copy-Item $licenseSource (Join-Path $packDir "04_LICENSE_NOTES.md")
} else {
    Write-Error "MISSING: deliverables\04_LICENSE_NOTES.md — create it first."
    exit 1
}

# --- Generate SHA-256 integrity file ---
Write-Host "Computing SHA-256 hashes..." -ForegroundColor Cyan

$hashFile = Join-Path $packDir "03_INTEGRITY\sha256sums.txt"
$allFiles = Get-ChildItem $packDir -Recurse -File |
    Where-Object { $_.FullName -ne $hashFile } |
    Sort-Object FullName

$lines = @()
foreach ($file in $allFiles) {
    $hash = (Get-FileHash -Path $file.FullName -Algorithm SHA256).Hash
    $rel = $file.FullName.Substring($packDir.Length + 1).Replace("\", "/")
    $lines += "$hash  $rel"
}
$lines | Out-File -FilePath $hashFile -Encoding UTF8

Write-Host ""
Write-Host "=== Genesis Experiment Pack v2 ===" -ForegroundColor Green
Write-Host "Location: $packDir" -ForegroundColor Green
Write-Host "Files:    $($allFiles.Count + 1) (including sha256sums.txt)" -ForegroundColor Green
Write-Host ""

foreach ($line in $lines) {
    Write-Host "  $line" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "Pack built successfully." -ForegroundColor Green
