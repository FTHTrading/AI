# moltbook_operator.ps1 — Hardened Moltbook Operator Runtime
# Deterministic. Idempotent. State-aware. Explicit about invariants.
#
# Usage:
#   .\moltbook_operator.ps1 -Action reply -PostId "xxx" -ParentCommentId "yyy" -Content "text"
#   .\moltbook_operator.ps1 -Action post -Title "title" -Content "body" [-SubmoltName "aiagents"]
#   .\moltbook_operator.ps1 -Action batch -BatchFile "replies.json"
#   .\moltbook_operator.ps1 -Action status
#
# All operations are:
#   1. Lock-guarded (single operator at a time)
#   2. Ledger-tracked (persistent reply registry)
#   3. Dedup-checked (never posts identical content twice)
#   4. Rate-limited (3s minimum between API calls)
#   5. Logged (append-only operator log)

param(
    [Parameter(Mandatory)][ValidateSet("reply", "post", "batch", "status")][string]$Action,
    [string]$PostId,
    [string]$ParentCommentId,
    [string]$Title,
    [string]$Content,
    [string]$SubmoltName = "aiagents",
    [string]$BatchFile,
    [string]$ApiKey = "moltbook_sk_fJpvzkSGYnqw6_3YaFnFHry-hgKr_aGI"
)

$ErrorActionPreference = "Stop"
$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

# ── Import Modules ────────────────────────────────────────────────────────────
$modulesDir = Join-Path $ScriptRoot "modules"
Import-Module (Join-Path $modulesDir "Moltbook.Logging.psm1") -Force
Import-Module (Join-Path $modulesDir "Moltbook.Lock.psm1") -Force
Import-Module (Join-Path $modulesDir "Moltbook.Ledger.psm1") -Force
Import-Module (Join-Path $modulesDir "Moltbook.Api.psm1") -Force

# ── Initialize ────────────────────────────────────────────────────────────────
$logPath = Join-Path $ScriptRoot "moltbook_operator.log"
$lockPath = Join-Path $ScriptRoot ".moltbook_lock"
$ledgerPath = Join-Path $ScriptRoot "moltbook_reply_ledger.json"

Initialize-MoltbookLog -Path $logPath
Initialize-MoltbookLock -Path $lockPath
Initialize-MoltbookLedger -Path $ledgerPath
Initialize-MoltbookApi -ApiKey $ApiKey

Write-MoltbookLog -Level INFO -Message "Operator started: Action=$Action PID=$PID"

# ── Status Action ─────────────────────────────────────────────────────────────
if ($Action -eq "status") {
    $stats = Get-LedgerStats
    $lockActive = Test-MoltbookLock
    Write-Host "=== Moltbook Operator Status ===" -ForegroundColor Cyan
    Write-Host "  Ledger:  $($stats.replies) replies across $($stats.posts) posts" -ForegroundColor White
    Write-Host "  Lock:    $(if ($lockActive) { 'ACTIVE (another operator running)' } else { 'Free' })" -ForegroundColor $(if ($lockActive) { "Yellow" } else { "Green" })
    Write-Host "  Log:     $logPath" -ForegroundColor White
    Write-Host "  Ledger:  $ledgerPath" -ForegroundColor White
    exit 0
}

# ── Acquire Lock ──────────────────────────────────────────────────────────────
if (Test-MoltbookLock) {
    Write-MoltbookLog -Level LOCK -Message "Lock already held. Aborting to prevent concurrent operation."
    Write-Host "ERROR: Another operator instance is running. Aborting." -ForegroundColor Red
    exit 1
}

Enter-MoltbookLock
Write-MoltbookLog -Level LOCK -Message "Lock acquired."

try {
    # ── Reply Action ──────────────────────────────────────────────────────────
    if ($Action -eq "reply") {
        if (-not $PostId -or -not $ParentCommentId -or -not $Content) {
            Write-MoltbookLog -Level ERROR -Message "Reply requires -PostId, -ParentCommentId, -Content"
            throw "Missing required parameters for reply action."
        }

        $result = Submit-MoltbookIdempotentComment -PostId $PostId -ParentCommentId $ParentCommentId -Content $Content

        if ($result.skipped) {
            Write-Host "SKIPPED: $($result.reason)" -ForegroundColor Yellow
        } elseif ($result.success) {
            Write-Host "POSTED: $($result.comment.id)" -ForegroundColor Green
        } else {
            Write-Host "FAILED: $($result.message)" -ForegroundColor Red
            exit 1
        }
    }

    # ── Post Action ───────────────────────────────────────────────────────────
    elseif ($Action -eq "post") {
        if (-not $Title -or -not $Content) {
            Write-MoltbookLog -Level ERROR -Message "Post requires -Title, -Content"
            throw "Missing required parameters for post action."
        }

        $result = Submit-MoltbookPost -Title $Title -Content $Content -SubmoltName $SubmoltName

        if ($result.success) {
            Write-Host "POSTED: $($result.post.id) '$Title'" -ForegroundColor Green
        } else {
            Write-Host "FAILED: $($result.message)" -ForegroundColor Red
            exit 1
        }
    }

    # ── Batch Action ──────────────────────────────────────────────────────────
    elseif ($Action -eq "batch") {
        if (-not $BatchFile -or -not (Test-Path $BatchFile)) {
            Write-MoltbookLog -Level ERROR -Message "Batch requires -BatchFile pointing to a valid JSON file"
            throw "Missing or invalid batch file."
        }

        $batch = Get-Content $BatchFile -Raw | ConvertFrom-Json
        $posted = 0; $skipped = 0; $failed = 0

        Write-MoltbookLog -Level INFO -Message "Batch started: $($batch.Count) items from $BatchFile"

        foreach ($item in $batch) {
            Write-Host "`n--- $($item.label) ---" -ForegroundColor Cyan

            if ($item.action -eq "reply" -or ($item.post_id -and $item.parent_comment_id)) {
                $result = Submit-MoltbookIdempotentComment -PostId $item.post_id -ParentCommentId $item.parent_comment_id -Content $item.content

                if ($result.skipped) {
                    Write-Host "  SKIPPED: $($result.reason)" -ForegroundColor Yellow
                    $skipped++
                } elseif ($result.success) {
                    Write-Host "  POSTED" -ForegroundColor Green
                    $posted++
                } else {
                    Write-Host "  FAILED: $($result.message)" -ForegroundColor Red
                    $failed++
                }
            } elseif ($item.action -eq "post") {
                $submoltName = if ($item.submolt_name) { $item.submolt_name } else { "aiagents" }
                $result = Submit-MoltbookPost -Title $item.title -Content $item.content -SubmoltName $submoltName

                if ($result.success) {
                    Write-Host "  POSTED: $($result.post.id)" -ForegroundColor Green
                    $posted++
                } else {
                    Write-Host "  FAILED: $($result.message)" -ForegroundColor Red
                    $failed++
                }
            } else {
                Write-Host "  UNKNOWN ACTION: $($item.action)" -ForegroundColor Red
                $failed++
            }
        }

        Write-MoltbookLog -Level INFO -Message "Batch complete: $posted posted, $skipped skipped, $failed failed"
        Write-Host "`n=== BATCH COMPLETE: $posted posted, $skipped skipped, $failed failed ===" -ForegroundColor $(if ($failed -eq 0) { "Green" } else { "Yellow" })
    }
}
finally {
    # ── Always release lock ───────────────────────────────────────────────────
    Exit-MoltbookLock
    Write-MoltbookLog -Level LOCK -Message "Lock released."
    Write-MoltbookLog -Level INFO -Message "Operator finished."
}
