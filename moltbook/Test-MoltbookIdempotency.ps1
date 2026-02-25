# Test-MoltbookIdempotency.ps1 — Idempotency test suite for the Moltbook operator runtime
# Validates: dedup, ledger, locking, hash determinism, word-number parsing
# Run: .\Test-MoltbookIdempotency.ps1

$ErrorActionPreference = "Stop"
$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

# ── Import Modules ────────────────────────────────────────────────────────────
$modulesDir = Join-Path $ScriptRoot "modules"
Import-Module (Join-Path $modulesDir "Moltbook.Logging.psm1") -Force
Import-Module (Join-Path $modulesDir "Moltbook.Lock.psm1") -Force
Import-Module (Join-Path $modulesDir "Moltbook.Ledger.psm1") -Force
Import-Module (Join-Path $modulesDir "Moltbook.Api.psm1") -Force

# ── Test Helpers ──────────────────────────────────────────────────────────────
$script:TestCount = 0
$script:PassCount = 0
$script:FailCount = 0

function Assert-Equal {
    param($Expected, $Actual, [string]$TestName)
    $script:TestCount++
    if ($Expected -eq $Actual) {
        $script:PassCount++
        Write-Host "  PASS  $TestName" -ForegroundColor Green
    } else {
        $script:FailCount++
        Write-Host "  FAIL  $TestName" -ForegroundColor Red
        Write-Host "         Expected: $Expected" -ForegroundColor DarkGray
        Write-Host "         Actual:   $Actual" -ForegroundColor DarkGray
    }
}

function Assert-True {
    param([bool]$Value, [string]$TestName)
    Assert-Equal -Expected $true -Actual $Value -TestName $TestName
}

function Assert-False {
    param([bool]$Value, [string]$TestName)
    Assert-Equal -Expected $false -Actual $Value -TestName $TestName
}

function Assert-NotNull {
    param($Value, [string]$TestName)
    $script:TestCount++
    if ($null -ne $Value) {
        $script:PassCount++
        Write-Host "  PASS  $TestName" -ForegroundColor Green
    } else {
        $script:FailCount++
        Write-Host "  FAIL  $TestName (was null)" -ForegroundColor Red
    }
}

# ── Setup ─────────────────────────────────────────────────────────────────────
Write-Host "`n=== Moltbook Idempotency Test Suite ===" -ForegroundColor Cyan
Write-Host ""

# Use temp paths so tests don't pollute the real state
$testDir = Join-Path ([System.IO.Path]::GetTempPath()) "moltbook_test_$(Get-Random)"
New-Item $testDir -ItemType Directory -Force | Out-Null

$testLogPath = Join-Path $testDir "test.log"
$testLockPath = Join-Path $testDir ".test_lock"
$testLedgerPath = Join-Path $testDir "test_ledger.json"

Initialize-MoltbookLog -Path $testLogPath
Initialize-MoltbookLock -Path $testLockPath
Initialize-MoltbookLedger -Path $testLedgerPath
Initialize-MoltbookApi -ApiKey "test_key_not_real" -BaseUrl "https://localhost:0"

# ══════════════════════════════════════════════════════════════════════════════
# TEST GROUP 1: Hash Determinism
# ══════════════════════════════════════════════════════════════════════════════
Write-Host "[1] Hash Determinism" -ForegroundColor Yellow

$hash1 = Get-ResponseHash -CommentId "abc-123" -ResponseText "Hello World"
$hash2 = Get-ResponseHash -CommentId "abc-123" -ResponseText "Hello World"
Assert-Equal -Expected $hash1 -Actual $hash2 -TestName "Same inputs produce same hash"

$hash3 = Get-ResponseHash -CommentId "abc-123" -ResponseText "Hello  World"
Assert-Equal -Expected $hash1 -Actual $hash3 -TestName "Whitespace normalization: extra spaces"

$hash4 = Get-ResponseHash -CommentId "abc-123" -ResponseText "hello world"
Assert-Equal -Expected $hash1 -Actual $hash4 -TestName "Case normalization: lowercase"

$hash5 = Get-ResponseHash -CommentId "def-456" -ResponseText "Hello World"
Assert-True -Value ($hash1 -ne $hash5) -TestName "Different comment IDs produce different hashes"

$hash6 = Get-ResponseHash -CommentId "abc-123" -ResponseText "Goodbye World"
Assert-True -Value ($hash1 -ne $hash6) -TestName "Different content produces different hashes"

# ══════════════════════════════════════════════════════════════════════════════
# TEST GROUP 2: Ledger Operations
# ══════════════════════════════════════════════════════════════════════════════
Write-Host "`n[2] Ledger Operations" -ForegroundColor Yellow

Assert-False -Value (Test-AlreadyReplied -PostId "post-1" -CommentId "comment-1") -TestName "Empty ledger: not replied"

Add-LedgerEntry -PostId "post-1" -CommentId "comment-1" -ResponseHash "hash_abc" -ContentPreview "Test reply"

Assert-True -Value (Test-AlreadyReplied -PostId "post-1" -CommentId "comment-1") -TestName "After add: already replied"
Assert-False -Value (Test-AlreadyReplied -PostId "post-1" -CommentId "comment-2") -TestName "Different comment: not replied"
Assert-False -Value (Test-AlreadyReplied -PostId "post-2" -CommentId "comment-1") -TestName "Different post: not replied"

Assert-True -Value (Test-ResponseHashExists -PostId "post-1" -ResponseHash "hash_abc") -TestName "Hash exists on post"
Assert-False -Value (Test-ResponseHashExists -PostId "post-1" -ResponseHash "hash_xyz") -TestName "Different hash: not found"
Assert-False -Value (Test-ResponseHashExists -PostId "post-2" -ResponseHash "hash_abc") -TestName "Different post: hash not found"

# Test persistence: reload ledger from disk
Initialize-MoltbookLedger -Path $testLedgerPath
Assert-True -Value (Test-AlreadyReplied -PostId "post-1" -CommentId "comment-1") -TestName "Ledger persists across reload"

$stats = Get-LedgerStats
Assert-Equal -Expected 1 -Actual $stats.posts -TestName "Stats: 1 post"
Assert-Equal -Expected 1 -Actual $stats.replies -TestName "Stats: 1 reply"

# Add more entries
Add-LedgerEntry -PostId "post-1" -CommentId "comment-2" -ResponseHash "hash_def" -ContentPreview "Second reply"
Add-LedgerEntry -PostId "post-2" -CommentId "comment-3" -ResponseHash "hash_ghi" -ContentPreview "Third reply"

$stats2 = Get-LedgerStats
Assert-Equal -Expected 2 -Actual $stats2.posts -TestName "Stats: 2 posts after adds"
Assert-Equal -Expected 3 -Actual $stats2.replies -TestName "Stats: 3 replies after adds"

# ══════════════════════════════════════════════════════════════════════════════
# TEST GROUP 3: Lock Behavior
# ══════════════════════════════════════════════════════════════════════════════
Write-Host "`n[3] Lock Behavior" -ForegroundColor Yellow

Assert-False -Value (Test-MoltbookLock) -TestName "No lock initially"

$null = Enter-MoltbookLock
Assert-True -Value (Test-MoltbookLock) -TestName "Lock acquired"
Assert-True -Value (Test-Path $testLockPath) -TestName "Lock file exists on disk"

# Read lock content
$lockContent = Get-Content $testLockPath -Raw | ConvertFrom-Json
Assert-Equal -Expected $PID -Actual $lockContent.pid -TestName "Lock contains correct PID"

Exit-MoltbookLock
Assert-False -Value (Test-MoltbookLock) -TestName "Lock released"
Assert-False -Value (Test-Path $testLockPath) -TestName "Lock file removed"

# Stale lock detection (fake an old lock)
$staleLock = @{
    pid       = 99999
    timestamp = (Get-Date).AddMinutes(-15).ToUniversalTime().ToString("o")
    host      = $env:COMPUTERNAME
} | ConvertTo-Json
[System.IO.File]::WriteAllText($testLockPath, $staleLock)

Assert-False -Value (Test-MoltbookLock) -TestName "Stale lock (15min old) detected and cleared"

# ══════════════════════════════════════════════════════════════════════════════
# TEST GROUP 4: Word-Number Parsing
# ══════════════════════════════════════════════════════════════════════════════
Write-Host "`n[4] Word-Number Parsing" -ForegroundColor Yellow

Assert-Equal -Expected 30 -Actual (ConvertFrom-WordNumber "ThIrTy") -TestName "ThIrTy -> 30"
Assert-Equal -Expected 15 -Actual (ConvertFrom-WordNumber "FiFtEeN") -TestName "FiFtEeN -> 15"
Assert-Equal -Expected 42 -Actual (ConvertFrom-WordNumber "forty-two") -TestName "forty-two -> 42"
Assert-Equal -Expected 100 -Actual (ConvertFrom-WordNumber "one hundred") -TestName "one hundred -> 100"
Assert-Equal -Expected 7 -Actual (ConvertFrom-WordNumber "seven") -TestName "seven -> 7"
Assert-Equal -Expected 99 -Actual (ConvertFrom-WordNumber "ninety-nine") -TestName "ninety-nine -> 99"
Assert-Equal -Expected 0 -Actual (ConvertFrom-WordNumber "zero") -TestName "zero -> 0"
Assert-Equal -Expected 42 -Actual (ConvertFrom-WordNumber "42") -TestName "42 (digits) -> 42"

# ══════════════════════════════════════════════════════════════════════════════
# TEST GROUP 5: Challenge Solving
# ══════════════════════════════════════════════════════════════════════════════
Write-Host "`n[5] Challenge Solving" -ForegroundColor Yellow

# Digit-based
$a1 = Solve-Challenge "A force of 30 NeWtOnS increases by 15 NeWtOnS. What is the new value?"
Assert-Equal -Expected "45.00" -Actual $a1 -TestName "30 + 15 (digit) = 45.00"

$a2 = Solve-Challenge "A weight of 80 pounds decreases by 20 pounds."
Assert-Equal -Expected "60.00" -Actual $a2 -TestName "80 - 20 (digit) = 60.00"

# Word-based
$a3 = Solve-Challenge "A force of ThIrTy NeWtOnS increases by FiFtEeN NeWtOnS. What is the total?"
Assert-Equal -Expected "45.00" -Actual $a3 -TestName "ThIrTy + FiFtEeN (word) = 45.00"

# Mixed
$a4 = Solve-Challenge "A length of fifty meters gains 10 meters."
Assert-Equal -Expected "60.00" -Actual $a4 -TestName "fifty + 10 (mixed) = 60.00"

# Multiplication
$a5 = Solve-Challenge "A force of 5 NeWtOnS times 3 NeWtOnS."
Assert-Equal -Expected "15.00" -Actual $a5 -TestName "5 * 3 = 15.00"

# Number extraction
$nums = Extract-ChallengeNumbers "A force of ThIrTy NeWtOnS increases by 15 grams"
Assert-True -Value ($nums.Count -ge 2) -TestName "Extract: found at least 2 numbers from mixed challenge"

# ══════════════════════════════════════════════════════════════════════════════
# TEST GROUP 5b: Deobfuscation (doubled-character patterns)
# ══════════════════════════════════════════════════════════════════════════════
Write-Host "`n[5b] Deobfuscation" -ForegroundColor Yellow

$deob1 = Deobfuscate-ChallengeText "tWwEeNnTtYy nIiNnEe"
Assert-True -Value ($deob1.ToLower() -match "twenty") -TestName "Deobfuscate: tWwEeNnTtYy -> twenty"
Assert-True -Value ($deob1.ToLower() -match "nine") -TestName "Deobfuscate: nIiNnEe -> nine"

$deob2 = Deobfuscate-ChallengeText "fFoOuUrR tEeEnN"
Assert-True -Value ($deob2.ToLower() -match "four") -TestName "Deobfuscate: fFoOuUrR -> four"

# Full doubled-character challenge
$a6 = Solve-Challenge "A] lOoOobBbSsTtEeR] cLl.aaWwS /eXxErRccIiSsEe] fOoRrCcEe] aPpPpLlIiIeSs] tWwEeNnTtYy] nIiNnEe] noOtToOnNs~, aNd] tThHeE] oOtThHeErR] cLl.aAwW] aPpPpLlIiIeSs] fFoOuUrR tEeEnN] noOtToOnNs<, wWhHaT/] iSs] tThHeE] tOoTtAaLl] fFoOrRcEe>?"
Assert-Equal -Expected "43.00" -Actual $a6 -TestName "Doubled-char challenge: 29 + 14 = 43.00"

# ══════════════════════════════════════════════════════════════════════════════
# TEST GROUP 6: Idempotent Comment (Unit — No Network)
# ══════════════════════════════════════════════════════════════════════════════
Write-Host "`n[6] Idempotent Comment Guard Logic" -ForegroundColor Yellow

# Re-init ledger clean for this test
$testLedger2 = Join-Path $testDir "test_ledger2.json"
Initialize-MoltbookLedger -Path $testLedger2

# Simulate: first reply should NOT be in ledger
Assert-False -Value (Test-AlreadyReplied -PostId "p1" -CommentId "c1") -TestName "First reply: not in ledger"

# Record it as if we posted
$h = Get-ResponseHash -CommentId "c1" -ResponseText "Reply content"
Add-LedgerEntry -PostId "p1" -CommentId "c1" -ResponseHash $h -ContentPreview "Reply content"

# Second attempt: ledger should block
Assert-True -Value (Test-AlreadyReplied -PostId "p1" -CommentId "c1") -TestName "Second reply: ledger blocks"

# Same content different comment: hash exists check
Assert-True -Value (Test-ResponseHashExists -PostId "p1" -ResponseHash $h) -TestName "Same hash on same post: blocked"

# Different content same comment: should still be blocked by comment-level check
Assert-True -Value (Test-AlreadyReplied -PostId "p1" -CommentId "c1") -TestName "Different content same comment: still blocked"

# ══════════════════════════════════════════════════════════════════════════════
# TEST GROUP 7: Logging
# ══════════════════════════════════════════════════════════════════════════════
Write-Host "`n[7] Logging" -ForegroundColor Yellow

Write-MoltbookLog -Level INFO -Message "Test log entry"
$logContent = Get-Content $testLogPath -Raw
Assert-True -Value ($logContent -match "Test log entry") -TestName "Log entry written to file"
Assert-True -Value ($logContent -match "INFO") -TestName "Log entry has level tag"

# ══════════════════════════════════════════════════════════════════════════════
# RESULTS
# ══════════════════════════════════════════════════════════════════════════════
Write-Host "`n=== RESULTS ===" -ForegroundColor Cyan
Write-Host "  Total:  $($script:TestCount)" -ForegroundColor White
Write-Host "  Passed: $($script:PassCount)" -ForegroundColor Green
Write-Host "  Failed: $($script:FailCount)" -ForegroundColor $(if ($script:FailCount -eq 0) { "Green" } else { "Red" })

# ── Cleanup ───────────────────────────────────────────────────────────────────
Remove-Item $testDir -Recurse -Force -ErrorAction SilentlyContinue

if ($script:FailCount -gt 0) {
    Write-Host "`nIDEMPOTENCY TESTS FAILED" -ForegroundColor Red
    exit 1
} else {
    Write-Host "`nALL IDEMPOTENCY TESTS PASSED" -ForegroundColor Green
    exit 0
}
