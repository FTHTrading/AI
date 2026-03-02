# Moltbook.Api.psm1 — API wrapper with rate-limit guard, word-number verification, idempotent posting
# All Moltbook HTTP calls go through this module. No other script should call the API directly.

$script:ApiKey = $null
$script:BaseUrl = $null
$script:LastCallTime = [datetime]::MinValue
$script:MinIntervalMs = 3000  # 3s minimum between API calls

# ── Word-number conversion table ──────────────────────────────────────────────
$script:WordNumbers = @{
    "zero"=0; "one"=1; "two"=2; "three"=3; "four"=4; "five"=5; "six"=6; "seven"=7; "eight"=8; "nine"=9
    "ten"=10; "eleven"=11; "twelve"=12; "thirteen"=13; "fourteen"=14; "fifteen"=15; "sixteen"=16
    "seventeen"=17; "eighteen"=18; "nineteen"=19; "twenty"=20; "thirty"=30; "forty"=40; "fifty"=50
    "sixty"=60; "seventy"=70; "eighty"=80; "ninety"=90; "hundred"=100; "thousand"=1000
    # Moltbook splits compound numbers: "four teen" = 4 + 10 = 14
    "teen"=10
}

function Initialize-MoltbookApi {
    param(
        [Parameter(Mandatory)][string]$ApiKey,
        [string]$BaseUrl = "https://www.moltbook.com/api/v1"
    )
    $script:ApiKey = $ApiKey
    $script:BaseUrl = $BaseUrl
}

# ── Rate Limit Guard ──────────────────────────────────────────────────────────
function Wait-RateLimit {
    $elapsed = ((Get-Date) - $script:LastCallTime).TotalMilliseconds
    if ($elapsed -lt $script:MinIntervalMs) {
        $waitMs = $script:MinIntervalMs - $elapsed
        Start-Sleep -Milliseconds ([int]$waitMs)
    }
}

function Update-CallTimestamp { $script:LastCallTime = Get-Date }

# ── Core HTTP caller ──────────────────────────────────────────────────────────
function Invoke-MoltbookApi {
    <#
    .SYNOPSIS
        Central API caller. All requests route through here.
        Handles rate limiting (429) with single retry.
    #>
    param(
        [Parameter(Mandatory)][string]$Method,
        [Parameter(Mandatory)][string]$Endpoint,
        [string]$Body = $null
    )
    Wait-RateLimit

    $url = "$($script:BaseUrl)$Endpoint"
    $headers = @(
        "-H", "Authorization: Bearer $($script:ApiKey)",
        "-H", "Content-Type: application/json"
    )

    $args = @("-s", "-X", $Method) + @($url) + $headers

    $tf = $null
    if ($Body) {
        $tf = [System.IO.Path]::GetTempFileName()
        [System.IO.File]::WriteAllText($tf, $Body)
        $args += @("-d", "@$tf")
    }

    try {
        $raw = & curl.exe @args 2>&1
        Update-CallTimestamp
    } finally {
        if ($tf) { Remove-Item $tf -ErrorAction SilentlyContinue }
    }

    $parsed = $null
    try { $parsed = $raw | ConvertFrom-Json } catch {
        return @{ success = $false; message = "JSON parse error"; raw = ($raw -join "`n") }
    }

    # Handle 429 rate limit — single retry
    if (-not $parsed.success -and $parsed.retry_after_minutes) {
        $waitSec = ([int]$parsed.retry_after_minutes * 60) + 5
        Write-MoltbookLog -Level RATE -Message "429 rate limited. Waiting ${waitSec}s before retry..."
        Start-Sleep -Seconds $waitSec

        $tf2 = $null
        if ($Body) {
            $tf2 = [System.IO.Path]::GetTempFileName()
            [System.IO.File]::WriteAllText($tf2, $Body)
            $args2 = @("-s", "-X", $Method) + @($url) + $headers + @("-d", "@$tf2")
        } else {
            $args2 = @("-s", "-X", $Method) + @($url) + $headers
        }

        try {
            $raw = & curl.exe @args2 2>&1
            Update-CallTimestamp
        } finally {
            if ($tf2) { Remove-Item $tf2 -ErrorAction SilentlyContinue }
        }

        try { $parsed = $raw | ConvertFrom-Json } catch {
            return @{ success = $false; message = "JSON parse error on retry"; raw = ($raw -join "`n") }
        }
    }

    return $parsed
}

# ── Word-to-Number Parser ────────────────────────────────────────────────────
function ConvertFrom-WordNumber {
    <#
    .SYNOPSIS
        Converts obfuscated word-numbers like "ThIrTy", "FiFtEeN" to numeric values.
        Also handles plain digits. Returns [double] or $null on failure.
    #>
    param([Parameter(Mandatory)][string]$Text)

    $clean = $Text.Trim().ToLowerInvariant()

    # Pure digits
    if ($clean -match '^\d+(\.\d+)?$') { return [double]$clean }

    # Compound like "twenty-five" or "twenty five"
    $parts = $clean -split '[-\s]+'
    $total = 0
    $current = 0

    foreach ($part in $parts) {
        if ($script:WordNumbers.ContainsKey($part)) {
            $val = $script:WordNumbers[$part]
            if ($val -eq 100) {
                $current = if ($current -eq 0) { 100 } else { $current * 100 }
            } elseif ($val -eq 1000) {
                $current = if ($current -eq 0) { 1000 } else { $current * 1000 }
                $total += $current
                $current = 0
            } else {
                $current += $val
            }
        } else {
            # Unknown word — fail gracefully
            return $null
        }
    }
    $total += $current
    return [double]$total
}

function Deobfuscate-ChallengeText {
    <#
    .SYNOPSIS
        Deobfuscates Moltbook challenge text.
        Handles two obfuscation styles:
          1. Doubled pairs within a token: "tWeNtY" -> "twenty"
          2. Characters split across space-separated micro-tokens: "tW eN tY" -> "twenty"
        Returns a hashtable with:
          .Spaced   - tokens joined with spaces (for word-boundary matching)
          .Concat   - tokens joined without spaces (for cross-token number words like "twentyeight")
    #>
    param([Parameter(Mandatory)][string]$Text)

    # Step 1: Strip all punctuation/special chars, keep letters, digits, spaces
    $cleaned = $Text -replace '[^a-zA-Z0-9\s]', ' '

    # Step 2: Per-token: collapse doubled case-pairs and lowercase
    $words = $cleaned -split '\s+' | Where-Object { $_ }
    $tokens = @()
    foreach ($word in $words) {
        $deduped = ""
        $i = 0
        while ($i -lt $word.Length) {
            $c = $word[$i]
            if ($i + 1 -lt $word.Length -and [char]::ToLowerInvariant($c) -eq [char]::ToLowerInvariant($word[$i + 1])) {
                $deduped += [char]::ToLowerInvariant($c)
                $i += 2
            } else {
                $deduped += [char]::ToLowerInvariant($c)
                $i++
            }
        }
        if ($deduped) { $tokens += $deduped }
    }

    return @{
        Spaced = ($tokens -join " ")
        Concat = ($tokens -join "")
    }
}

function Extract-ChallengeNumbers {
    <#
    .SYNOPSIS
        Extracts numbers from a Moltbook verification challenge.
        Handles digit-based, word-based, and doubled-character obfuscated numbers.
        Handles cross-token compound numbers like "tW eN tY eI gGhT" = 28 via Concat.
    #>
    param([Parameter(Mandatory)][string]$ChallengeText)

    $numbers = @()

    # Deobfuscate — returns .Spaced and .Concat
    $deob = Deobfuscate-ChallengeText -Text $ChallengeText

    # ---- Spaced/original text: use word-boundary patterns ----
    $compoundSpaced = '(?i)(twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety)[\s-]+(one|two|three|four|five|six|seven|eight|nine)'
    $simpleSpaced   = '(?i)\b(zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred|thousand)\b'

    foreach ($text in @($ChallengeText, $deob.Spaced)) {
        # Plain digits
        foreach ($m in ([regex]::Matches($text, '\b\d+(\.\d+)?\b'))) {
            $numbers += [double]$m.Value
        }
        # Compound word-numbers (e.g. "twenty five") — track covered positions
        $coveredSpaced = @()
        foreach ($m in ([regex]::Matches($text, $compoundSpaced))) {
            $v = ConvertFrom-WordNumber -Text "$($m.Groups[1].Value)-$($m.Groups[2].Value)"
            if ($null -ne $v) {
                $numbers += $v
                $coveredSpaced += @{ Start = $m.Index; End = $m.Index + $m.Length - 1 }
            }
        }
        # Simple word-numbers — skip those covered by compound matches
        foreach ($m in ([regex]::Matches($text, $simpleSpaced))) {
            $covered = $false
            foreach ($cr in $coveredSpaced) {
                if ($m.Index -ge $cr.Start -and $m.Index -le $cr.End) { $covered = $true; break }
            }
            if (-not $covered) {
                $v = ConvertFrom-WordNumber -Text $m.Value.Trim()
                if ($null -ne $v) { $numbers += $v }
            }
        }
    }

    # ---- Concatenated text: no word boundaries, for split tokens ----
    # Normalize near-miss number words that deobfuscation can mangle
    # e.g. "thre" (from "ThR ee") should correct to "three"
    $nearMissMap = @{
        'thre'     = 'three'
        'ninteen'  = 'nineteen'
        'elevin'   = 'eleven'
        'twelv'    = 'twelve'
        'forteen'  = 'fourteen'
        'sixtee'   = 'sixteen'
        'sevente'  = 'seventeen'
        'eightee'  = 'eighteen'
        'fiften'   = 'fifteen'
    }
    $concatText = $deob.Concat
    foreach ($near in $nearMissMap.GetEnumerator()) {
        $concatText = [regex]::Replace($concatText, "(?i)$([regex]::Escape($near.Key))", $near.Value)
    }

    # Process compound matches first, track covered positions, then skip simples that overlap
    $compoundConcat = '(?i)(twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety)(one|two|three|four|five|six|seven|eight|nine)'
    $simpleConcat   = '(?i)(nineteen|eighteen|seventeen|sixteen|fifteen|fourteen|thirteen|twelve|eleven|hundred|thousand|zero|one|two|three|four|five|six|seven|eight|nine|ten|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety)'

    $coveredRanges = @()
    foreach ($m in ([regex]::Matches($concatText, $compoundConcat))) {
        $v = ConvertFrom-WordNumber -Text "$($m.Groups[1].Value)-$($m.Groups[2].Value)"
        if ($null -ne $v) {
            $numbers += $v
            $coveredRanges += @{ Start = $m.Index; End = $m.Index + $m.Length - 1 }
        }
    }
    foreach ($m in ([regex]::Matches($concatText, $simpleConcat))) {
        # Skip if this position is within a compound match's range
        $isInCompound = $false
        foreach ($range in $coveredRanges) {
            if ($m.Index -ge $range.Start -and $m.Index -le $range.End) {
                $isInCompound = $true; break
            }
        }
        if (-not $isInCompound) {
            $v = ConvertFrom-WordNumber -Text $m.Value.Trim()
            if ($null -ne $v) { $numbers += $v }
        }
    }

    # Deduplicate by integer value, first-seen order
    $seen = @{}
    $unique = @()
    foreach ($n in $numbers) {
        $key = [string][int]$n
        if (-not $seen.ContainsKey($key)) {
            $seen[$key] = $true
            $unique += $n
        }
    }

    return $unique
}

function Solve-Challenge {
    <#
    .SYNOPSIS
        Parses a Moltbook verification challenge and returns the answer string.
    #>
    param([Parameter(Mandatory)][string]$ChallengeText)

    $numbers = Extract-ChallengeNumbers -ChallengeText $ChallengeText
    if ($numbers.Count -lt 2) {
        Write-MoltbookLog -Level WARN -Message "Challenge extraction found $($numbers.Count) numbers: '$ChallengeText'"
        return $null
    }

    $num1 = $numbers[0]
    $num2 = $numbers[1]

    # Use deobfuscated text for operation detection
    $deob = Deobfuscate-ChallengeText -Text $ChallengeText
    $lower = $deob.Spaced.ToLower()

    if ($lower -match 'times|multipl|product|doubl|exert|tripl') {
        $answer = $num1 * $num2
    } elseif ($lower -match 'divid|split|ratio|per|half') {
        if ($num2 -ne 0) { $answer = $num1 / $num2 } else { return $null }
    } elseif ($lower -match 'decreas|subtract|minus|lose|less|drop|reduc|remov|remain') {
        $answer = $num1 - $num2
    } else {
        # Default: addition
        $answer = $num1 + $num2
    }

    # Sanity check with 3-number challenges:
    # "has X, adds Y, to get Z" — answer is Z (appears after "to get" in concat form)
    if ($numbers.Count -ge 3 -and ($lower -match 'to get|equal')) {
        # Work on concat text since spaced has fragmented tokens
        $concatLower = ($deob.Concat).ToLower()
        $toGetMatch = [regex]::Match($concatLower, 'toget((?:twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety)(?:one|two|three|four|five|six|seven|eight|nine)?|zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|\d+)')
        if ($toGetMatch.Success) {
            $rawTg = $toGetMatch.Groups[1].Value.Trim()
            # ConvertFrom-WordNumber needs hyphen/space separation for compound words
            # Try inserting a dash between "twenty|thirty... + ones digit" if no separator
            $rawTg = [regex]::Replace($rawTg, '(?i)(twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety)(one|two|three|four|five|six|seven|eight|nine)$', '$1-$2')
            $tgVal = ConvertFrom-WordNumber -Text $rawTg
            if ($null -ne $tgVal) { $answer = $tgVal }
        }
    }

    return "{0:F2}" -f $answer
}

# ── Public API Functions ──────────────────────────────────────────────────────

function Invoke-MoltbookVerify {
    <#
    .SYNOPSIS
        Solves and submits a verification challenge.
    #>
    param(
        [Parameter(Mandatory)][string]$VerificationCode,
        [Parameter(Mandatory)][string]$ChallengeText
    )
    $answer = Solve-Challenge -ChallengeText $ChallengeText
    if (-not $answer) {
        Write-MoltbookLog -Level ERROR -Message "Could not solve challenge: $ChallengeText"
        return @{ success = $false; message = "parse_failure" }
    }

    Write-MoltbookLog -Level VERIFY -Message "Challenge answer: $answer"
    $body = @{ verification_code = $VerificationCode; answer = $answer } | ConvertTo-Json -Compress
    return Invoke-MoltbookApi -Method POST -Endpoint "/verify" -Body $body
}

function Get-MoltbookPostComments {
    <#
    .SYNOPSIS
        Fetches all comments on a post.
    #>
    param([Parameter(Mandatory)][string]$PostId)
    return Invoke-MoltbookApi -Method GET -Endpoint "/posts/$PostId/comments"
}

function Get-MoltbookAgentPosts {
    <#
    .SYNOPSIS
        Fetches posts by agent.
    #>
    param([string]$Agent = "genesisprotocol")
    return Invoke-MoltbookApi -Method GET -Endpoint "/agents/$Agent/posts"
}

function Submit-MoltbookComment {
    <#
    .SYNOPSIS
        Posts a comment with verification solving. Does NOT check idempotency.
        Use Submit-MoltbookIdempotentComment for dedup-guarded posting.
    #>
    param(
        [Parameter(Mandatory)][string]$PostId,
        [Parameter(Mandatory)][string]$Content
    )
    $body = @{ content = $Content } | ConvertTo-Json -Compress
    $result = Invoke-MoltbookApi -Method POST -Endpoint "/posts/$PostId/comments" -Body $body

    if (-not $result.success) {
        return $result
    }

    $commentId = $result.comment.id
    Write-MoltbookLog -Level POST -Message "Comment posted: $($commentId.Substring(0,8))"

    # Auto-verify if needed
    $v = $result.comment.verification
    if ($v -and $v.verification_code) {
        $vResult = Invoke-MoltbookVerify -VerificationCode $v.verification_code -ChallengeText $v.challenge_text
        if ($vResult.success -or $vResult.message -eq "Already answered") {
            Write-MoltbookLog -Level VERIFY -Message "Verified: $commentId"
        } else {
            Write-MoltbookLog -Level WARN -Message "Verification response: $($vResult.message)"
        }
    }

    return $result
}

function Submit-MoltbookIdempotentComment {
    <#
    .SYNOPSIS
        Idempotent comment posting:
        1. Check ledger — if already replied to this comment, SKIP
        2. Check response hash — if identical content hash exists on this post, SKIP  
        3. Fetch existing comments — if our agent already has a comment with identical text, SKIP
        4. Post the comment
        5. Record in ledger
    .DESCRIPTION
        This is the ONLY function external scripts should call for posting comments.
    #>
    param(
        [Parameter(Mandatory)][string]$PostId,
        [Parameter(Mandatory)][string]$ParentCommentId,
        [Parameter(Mandatory)][string]$Content
    )

    # Guard 1: Ledger check — already replied to this comment?
    if (Test-AlreadyReplied -PostId $PostId -CommentId $ParentCommentId) {
        Write-MoltbookLog -Level SKIP -Message "Ledger hit: already replied to $($ParentCommentId.Substring(0,8)) on $($PostId.Substring(0,8))"
        return @{ success = $true; skipped = $true; reason = "ledger_hit" }
    }

    # Guard 2: Response hash check — same content already posted to this post?
    $hash = Get-ResponseHash -CommentId $ParentCommentId -ResponseText $Content
    if (Test-ResponseHashExists -PostId $PostId -ResponseHash $hash) {
        Write-MoltbookLog -Level SKIP -Message "Hash collision: identical response already posted to $($PostId.Substring(0,8))"
        return @{ success = $true; skipped = $true; reason = "hash_collision" }
    }

    # Guard 3: Live API check — do we already have a reply on this post with this text?
    #   (Catches replies posted before the ledger existed)
    try {
        $existingComments = Get-MoltbookPostComments -PostId $PostId
        if ($existingComments.success -and $existingComments.comments) {
            $normalizedContent = ($Content -replace '\s+', ' ').Trim().ToLowerInvariant()
            foreach ($c in $existingComments.comments) {
                # Check if our agent already posted identical text
                if ($c.author -and $c.author.name -eq "genesisprotocol") {
                    $existingNorm = ($c.content -replace '\s+', ' ').Trim().ToLowerInvariant()
                    if ($existingNorm -eq $normalizedContent) {
                        Write-MoltbookLog -Level SKIP -Message "API dedup: identical comment already exists on $($PostId.Substring(0,8))"
                        # Backfill ledger
                        Add-LedgerEntry -PostId $PostId -CommentId $ParentCommentId -ResponseHash $hash -ContentPreview $Content -ResultCommentId $c.id
                        return @{ success = $true; skipped = $true; reason = "api_dedup" }
                    }
                }
            }
        }
    } catch {
        Write-MoltbookLog -Level WARN -Message "Could not fetch existing comments for dedup check: $_"
        # Continue — fail-open on read, never fail-open on write
    }

    # All guards passed — post the comment
    Write-MoltbookLog -Level POST -Message "Posting reply to $($ParentCommentId.Substring(0,8)) on $($PostId.Substring(0,8))"
    $result = Submit-MoltbookComment -PostId $PostId -Content $Content

    if ($result.success) {
        $resultId = if ($result.comment) { $result.comment.id } else { "" }
        Add-LedgerEntry -PostId $PostId -CommentId $ParentCommentId -ResponseHash $hash -ContentPreview $Content -ResultCommentId $resultId
        Write-MoltbookLog -Level POST -Message "Ledger recorded: $($ParentCommentId.Substring(0,8)) -> $($resultId.Substring(0,8))"
    } else {
        Write-MoltbookLog -Level ERROR -Message "Post failed: $($result.message)"
    }

    return $result
}

function Submit-MoltbookPost {
    <#
    .SYNOPSIS
        Creates a new post on a submolt with verification solving.
    #>
    param(
        [Parameter(Mandatory)][string]$Title,
        [Parameter(Mandatory)][string]$Content,
        [string]$SubmoltName = "aiagents"
    )
    $body = @{
        title         = $Title
        content       = $Content
        submolt_name  = $SubmoltName
    } | ConvertTo-Json -Compress

    $result = Invoke-MoltbookApi -Method POST -Endpoint "/posts" -Body $body

    if ($result.success -and $result.post) {
        Write-MoltbookLog -Level POST -Message "Post created: $($result.post.id.Substring(0,8)) '$Title'"

        $v = $result.post.verification
        if ($v -and $v.verification_code) {
            $vResult = Invoke-MoltbookVerify -VerificationCode $v.verification_code -ChallengeText $v.challenge_text
            if ($vResult.success -or $vResult.message -eq "Already answered") {
                Write-MoltbookLog -Level VERIFY -Message "Post verified: $($result.post.id.Substring(0,8))"
            } else {
                Write-MoltbookLog -Level WARN -Message "Post verification: $($vResult.message)"
            }
        }
    } elseif (-not $result.success) {
        Write-MoltbookLog -Level ERROR -Message "Post creation failed: $($result.message)"
    }

    return $result
}

Export-ModuleMember -Function Initialize-MoltbookApi, Invoke-MoltbookApi, Invoke-MoltbookVerify, `
    Get-MoltbookPostComments, Get-MoltbookAgentPosts, Submit-MoltbookComment, `
    Submit-MoltbookIdempotentComment, Submit-MoltbookPost, Solve-Challenge, `
    Extract-ChallengeNumbers, ConvertFrom-WordNumber, Deobfuscate-ChallengeText
