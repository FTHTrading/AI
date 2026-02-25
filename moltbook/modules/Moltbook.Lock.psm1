# Moltbook.Lock.psm1 — File-based concurrency lock
# Prevents concurrent script execution. try/finally enforced.

$script:LockPath = $null

function Initialize-MoltbookLock {
    param([Parameter(Mandatory)][string]$Path)
    $script:LockPath = $Path
}

function Test-MoltbookLock {
    <#
    .SYNOPSIS
        Returns $true if lock is held by another process.
    #>
    if (-not $script:LockPath) { return $false }
    if (-not (Test-Path $script:LockPath)) { return $false }

    # Check if lock is stale (older than 10 minutes)
    $lockInfo = Get-Content $script:LockPath -Raw -ErrorAction SilentlyContinue | ConvertFrom-Json -ErrorAction SilentlyContinue
    if ($lockInfo -and $lockInfo.timestamp) {
        $lockTime = [DateTimeOffset]::Parse($lockInfo.timestamp).UtcDateTime
        $age = (Get-Date).ToUniversalTime() - $lockTime
        if ($age.TotalMinutes -gt 10) {
            # Stale lock — remove it
            Remove-Item $script:LockPath -Force -ErrorAction SilentlyContinue
            return $false
        }
    }
    return $true
}

function Enter-MoltbookLock {
    <#
    .SYNOPSIS
        Acquires the lock. Returns $true on success, $false if already held.
    #>
    if (Test-MoltbookLock) {
        return $false
    }
    $lockData = @{
        pid       = $PID
        timestamp = (Get-Date).ToUniversalTime().ToString("o")
        host      = $env:COMPUTERNAME
    } | ConvertTo-Json -Compress
    $parentDir = Split-Path $script:LockPath -Parent
    if (-not (Test-Path $parentDir)) {
        New-Item $parentDir -ItemType Directory -Force | Out-Null
    }
    Set-Content -Path $script:LockPath -Value $lockData -NoNewline -Force
    return $true
}

function Exit-MoltbookLock {
    <#
    .SYNOPSIS
        Releases the lock. Safe to call multiple times.
    #>
    if ($script:LockPath -and (Test-Path $script:LockPath)) {
        Remove-Item $script:LockPath -Force -ErrorAction SilentlyContinue
    }
}

Export-ModuleMember -Function Initialize-MoltbookLock, Test-MoltbookLock, Enter-MoltbookLock, Exit-MoltbookLock
