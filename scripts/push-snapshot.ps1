# push-snapshot.ps1
# Watches docs/system-state.json and auto-commits + pushes it to GitHub
# so the GitHub Pages dashboard stays fresh.
#
# Usage: Run this in a separate terminal while the server is running.
#   cd C:\Users\Kevan\genesis-protocol
#   powershell -ExecutionPolicy Bypass -File scripts\push-snapshot.ps1

param(
    [int]$IntervalSeconds = 60  # How often to check for changes
)

$RepoRoot  = Split-Path -Parent $PSScriptRoot
$SnapFile  = Join-Path $RepoRoot "docs\system-state.json"
$LastHash  = ""

Set-Location $RepoRoot

Write-Host "Genesis Protocol — snapshot pusher"
Write-Host "Watching: $SnapFile"
Write-Host "Interval: every $IntervalSeconds seconds"
Write-Host "Press Ctrl+C to stop."
Write-Host ""

while ($true) {
    Start-Sleep -Seconds $IntervalSeconds

    if (-not (Test-Path $SnapFile)) {
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] system-state.json not found yet — server may not have autosaved." -ForegroundColor Yellow
        continue
    }

    # Hash current file to detect changes
    $Hash = (Get-FileHash $SnapFile -Algorithm MD5).Hash

    if ($Hash -eq $LastHash) {
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] No change." -ForegroundColor DarkGray
        continue
    }

    $LastHash = $Hash

    # Get epoch from the file for the commit message
    try {
        $snap = Get-Content $SnapFile -Raw | ConvertFrom-Json
        $Epoch = $snap.epoch
        $Pop   = $snap.population
        $Msg   = "chore: snapshot epoch $Epoch pop=$Pop [skip ci]"
    } catch {
        $Msg   = "chore: update system-state snapshot [skip ci]"
    }

    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Changed — committing: $Msg" -ForegroundColor Cyan

    git add docs/system-state.json 2>&1 | Out-Null

    $CommitOut = git commit -m $Msg 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  commit skipped (nothing staged)" -ForegroundColor DarkGray
        continue
    }

    $PushOut = git push 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  pushed OK" -ForegroundColor Green
    } else {
        Write-Host "  push failed: $PushOut" -ForegroundColor Red
    }
}
