# Moltbook.Ledger.psm1 — Persistent reply ledger with atomic writes
# Tracks every comment posted. Prevents duplicate replies.
# Structure: { posts: { <post_id>: { <comment_id>: { response_hash, timestamp, content_preview } } } }

$script:LedgerPath = $null
$script:Ledger = $null

function Initialize-MoltbookLedger {
    param([Parameter(Mandatory)][string]$Path)
    $script:LedgerPath = $Path
    if (Test-Path $Path) {
        $raw = Get-Content $Path -Raw -ErrorAction SilentlyContinue
        if ($raw) {
            $script:Ledger = $raw | ConvertFrom-Json -ErrorAction SilentlyContinue
        }
    }
    if (-not $script:Ledger) {
        $script:Ledger = [PSCustomObject]@{ posts = [PSCustomObject]@{} }
    }
}

function Get-ResponseHash {
    <#
    .SYNOPSIS
        SHA256(comment_id + normalized_response_text) — deterministic fingerprint.
    #>
    param(
        [Parameter(Mandatory)][string]$CommentId,
        [Parameter(Mandatory)][string]$ResponseText
    )
    $normalized = ($CommentId + "|" + ($ResponseText -replace '\s+', ' ').Trim()).ToLowerInvariant()
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($normalized)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $hash = $sha.ComputeHash($bytes)
    $sha.Dispose()
    return [BitConverter]::ToString($hash).Replace("-", "").ToLowerInvariant()
}

function Test-AlreadyReplied {
    <#
    .SYNOPSIS
        Returns $true if we have already posted a reply to this comment on this post.
    #>
    param(
        [Parameter(Mandatory)][string]$PostId,
        [Parameter(Mandatory)][string]$CommentId
    )
    if (-not $script:Ledger.posts.PSObject.Properties[$PostId]) { return $false }
    $postEntry = $script:Ledger.posts.$PostId
    if (-not $postEntry.PSObject.Properties[$CommentId]) { return $false }
    return $true
}

function Test-ResponseHashExists {
    <#
    .SYNOPSIS
        Returns $true if this exact response hash exists for any comment on this post.
        Catches identical text posted to different comments on the same post.
    #>
    param(
        [Parameter(Mandatory)][string]$PostId,
        [Parameter(Mandatory)][string]$ResponseHash
    )
    if (-not $script:Ledger.posts.PSObject.Properties[$PostId]) { return $false }
    $postEntry = $script:Ledger.posts.$PostId
    foreach ($prop in $postEntry.PSObject.Properties) {
        if ($prop.Value.response_hash -eq $ResponseHash) { return $true }
    }
    return $false
}

function Add-LedgerEntry {
    <#
    .SYNOPSIS
        Records a successful reply in the ledger and writes atomically.
    #>
    param(
        [Parameter(Mandatory)][string]$PostId,
        [Parameter(Mandatory)][string]$CommentId,
        [Parameter(Mandatory)][string]$ResponseHash,
        [string]$ContentPreview = "",
        [string]$ResultCommentId = ""
    )
    # Ensure post entry exists
    if (-not $script:Ledger.posts.PSObject.Properties[$PostId]) {
        $script:Ledger.posts | Add-Member -NotePropertyName $PostId -NotePropertyValue ([PSCustomObject]@{})
    }
    $postEntry = $script:Ledger.posts.$PostId

    # Add comment entry
    $entry = [PSCustomObject]@{
        response_hash    = $ResponseHash
        timestamp        = (Get-Date).ToUniversalTime().ToString("o")
        content_preview  = if ($ContentPreview.Length -gt 80) { $ContentPreview.Substring(0, 80) + "..." } else { $ContentPreview }
        result_comment_id = $ResultCommentId
    }
    $postEntry | Add-Member -NotePropertyName $CommentId -NotePropertyValue $entry -Force

    # Atomic write: temp file → rename
    Save-Ledger
}

function Save-Ledger {
    <#
    .SYNOPSIS
        Atomic write of ledger to disk.
    #>
    if (-not $script:LedgerPath) { return }
    $json = $script:Ledger | ConvertTo-Json -Depth 10
    $tmpPath = $script:LedgerPath + ".tmp"
    [System.IO.File]::WriteAllText($tmpPath, $json)
    if (Test-Path $script:LedgerPath) {
        Remove-Item $script:LedgerPath -Force
    }
    Rename-Item $tmpPath $script:LedgerPath
}

function Get-LedgerStats {
    <#
    .SYNOPSIS
        Returns summary statistics from the ledger.
    #>
    $totalPosts = 0
    $totalReplies = 0
    foreach ($postProp in $script:Ledger.posts.PSObject.Properties) {
        $totalPosts++
        foreach ($commentProp in $postProp.Value.PSObject.Properties) {
            $totalReplies++
        }
    }
    return @{
        posts   = $totalPosts
        replies = $totalReplies
    }
}

Export-ModuleMember -Function Initialize-MoltbookLedger, Get-ResponseHash, Test-AlreadyReplied, Test-ResponseHashExists, Add-LedgerEntry, Save-Ledger, Get-LedgerStats
