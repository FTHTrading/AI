# Moltbook Comment Poster with auto-verification
# Usage: .\post_comment.ps1 -PostId "xxxx-..." -Content "Your comment"

param(
    [Parameter(Mandatory=$true)][string]$PostId,
    [Parameter(Mandatory=$true)][string]$Content,
    [string]$ApiKey = "moltbook_sk_fJpvzkSGYnqw6_3YaFnFHry-hgKr_aGI",
    [string]$BaseUrl = "https://www.moltbook.com/api/v1"
)

$ErrorActionPreference = "Stop"

# Post comment
$body = @{ content = $Content } | ConvertTo-Json -Compress
$tf = [System.IO.Path]::GetTempFileName()
[System.IO.File]::WriteAllText($tf, $body)

Write-Host "Posting comment to $($PostId.Substring(0,8))..." -ForegroundColor Yellow
$response = curl.exe -s -X POST "$BaseUrl/posts/$PostId/comments" -H "Authorization: Bearer $ApiKey" -H "Content-Type: application/json" -d "@$tf" 2>&1
Remove-Item $tf -ErrorAction SilentlyContinue

$result = $response | ConvertFrom-Json
if (-not $result.success) {
    Write-Host "FAILED: $($result.message)" -ForegroundColor Red
    if ($result.retry_after_minutes) { Write-Host "Rate limited. Retry after $($result.retry_after_minutes) min." -ForegroundColor Yellow }
    exit 1
}

$commentId = $result.comment.id
Write-Host "Comment posted: $commentId" -ForegroundColor Green

# Check for verification
$verification = $result.comment.verification
if ($verification -and $verification.verification_code) {
    Write-Host "Solving verification..." -ForegroundColor Yellow
    $challenge = $verification.challenge_text
    Write-Host "Challenge: $challenge" -ForegroundColor DarkGray
    
    # Parse math from challenge text
    # Moltbook challenges follow pattern: "X is N, operation M, what's result?"
    if ($challenge -match '(\d+)\s*(NeWtOnS|pounds|grams|meters|celsius).*?(\d+)') {
        $num1 = [double]$Matches[1]
        $num2 = [double]$Matches[3]
        if ($challenge -match 'InCrEaSeS?\s*By|AdDs?|PlUs|GaInS?') {
            $answer = $num1 + $num2
        } elseif ($challenge -match 'DeCrEaSeS?\s*By|LoSeS?|MiNuS|DrOpS?') {
            $answer = $num1 - $num2
        } elseif ($challenge -match 'TiMeS|MuLtIpLiE') {
            $answer = $num1 * $num2
        } else {
            $answer = $num1 + $num2  # default to addition
        }
        $answerStr = "{0:F2}" -f $answer
    } else {
        Write-Host "Could not parse challenge automatically. Manual answer needed." -ForegroundColor Red
        exit 1
    }
    
    Write-Host "Answer: $answerStr" -ForegroundColor Cyan
    $vBody = @{ verification_code = $verification.verification_code; answer = $answerStr } | ConvertTo-Json -Compress
    $vtf = [System.IO.Path]::GetTempFileName()
    [System.IO.File]::WriteAllText($vtf, $vBody)
    
    $vResult = curl.exe -s -X POST "$BaseUrl/verify" -H "Authorization: Bearer $ApiKey" -H "Content-Type: application/json" -d "@$vtf" 2>&1
    Remove-Item $vtf -ErrorAction SilentlyContinue
    
    $vParsed = $vResult | ConvertFrom-Json
    if ($vParsed.success -or $vParsed.message -eq "Already answered") {
        Write-Host "Verified!" -ForegroundColor Green
    } else {
        Write-Host "Verification response: $($vParsed.message)" -ForegroundColor Yellow
    }
}

Write-Host "Done." -ForegroundColor Green
