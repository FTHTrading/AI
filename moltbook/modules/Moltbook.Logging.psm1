# Moltbook.Logging.psm1 — Append-only operator log
# Every action is recorded. No silent operations.

$script:LogPath = $null

function Initialize-MoltbookLog {
    param([string]$Path)
    $script:LogPath = $Path
    if (-not (Test-Path $Path)) {
        $null = New-Item -Path $Path -ItemType File -Force
    }
}

function Write-MoltbookLog {
    param(
        [Parameter(Mandatory)][string]$Message,
        [ValidateSet("INFO","WARN","ERROR","SKIP","POST","VERIFY","LOCK","RATE")]
        [string]$Level = "INFO"
    )
    $ts = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
    $line = "[$ts] [$Level] $Message"
    if ($script:LogPath) {
        Add-Content -Path $script:LogPath -Value $line -Encoding utf8
    }
    $color = switch ($Level) {
        "INFO"   { "White" }
        "WARN"   { "Yellow" }
        "ERROR"  { "Red" }
        "SKIP"   { "DarkGray" }
        "POST"   { "Green" }
        "VERIFY" { "Cyan" }
        "LOCK"   { "Magenta" }
        "RATE"   { "Yellow" }
        default  { "White" }
    }
    Write-Host $line -ForegroundColor $color
}

Export-ModuleMember -Function Initialize-MoltbookLog, Write-MoltbookLog
